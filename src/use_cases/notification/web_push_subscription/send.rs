use std::{collections::BTreeMap, sync::Arc};

use actix_tls::connect::rustls_0_23;
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use common::{db::decode_and_decrypt, settings::types::Settings};
use db_adapters::{
    ambition_adapter::{AmbitionAdapter, AmbitionFilter, AmbitionQuery},
    desired_state_adapter::{DesiredStateAdapter, DesiredStateFilter, DesiredStateQuery},
    web_push_subscription_adapter::{WebPushSubscriptionAdapter, WebPushSubscriptionQuery},
};
use ece::encrypt;
use entities::{user as user_entity, web_push_subscription};
use http::Uri;
use jwt_simple::{
    prelude::{Claims, Duration, ECDSAP256KeyPairLike, ECDSAP256PublicKeyLike, ES256KeyPair},
    reexports::rand::{seq::SliceRandom, thread_rng},
};
use serde::Serialize;
use serde_json::json;

use crate::UseCaseError;

const TTL_SECONDS: u64 = 60 * 60 * 23;

enum NotificationChoice {
    Ambition,
    DesiredState,
}

#[derive(Serialize)]
struct Message {
    body: String,
}

pub async fn send_web_push<'a>(
    user: user_entity::Model,
    settings: &Settings,
    web_push_subscription_adapter: WebPushSubscriptionAdapter<'a>,
    ambition_adapter: AmbitionAdapter<'a>,
    desired_state_adapter: DesiredStateAdapter<'a>,
) -> Result<(), UseCaseError> {
    let choices = [
        NotificationChoice::Ambition,
        NotificationChoice::DesiredState,
    ];
    let choice = choices.choose(&mut thread_rng()).unwrap();
    let message = match choice {
        NotificationChoice::Ambition => {
            let ambition = ambition_adapter
                .filter_eq_user(&user)
                .get_random()
                .await
                .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
                .ok_or(UseCaseError::NotFound(
                    "You don't have any ambition.".to_string(),
                ))?;
            Message {
                body: match ambition.description {
                    Some(description) => format!("{}:\n{}", ambition.name, description),
                    None => ambition.name,
                },
            }
        }
        NotificationChoice::DesiredState => {
            let desired_state = desired_state_adapter
                .filter_eq_user(&user)
                .get_random()
                .await
                .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
                .ok_or(UseCaseError::NotFound(
                    "You don't have any desired_state.".to_string(),
                ))?;
            Message {
                body: match desired_state.description {
                    Some(description) => format!("{}:\n{}", desired_state.name, description),
                    None => desired_state.name,
                },
            }
        }
    };

    let subscription = web_push_subscription_adapter
        .get_by_user(&user)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        .ok_or(UseCaseError::NotFound(
            "You don't have active web push subscription.".to_string(),
        ))?;
    let endpoint = decode_and_decrypt(subscription.endpoint.clone(), &settings)
        .map_err(|e| UseCaseError::InternalServerError(e))?;

    let encrypted_message = encrypt_message(json!(message).to_string(), &subscription, &settings)?;

    let authorization_header = get_authorization_header(&endpoint, &settings)?;

    send_push_request(encrypted_message, &endpoint, &authorization_header).await?;

    dbg!(json!(message).to_string());

    Ok(())
}

fn encrypt_message(
    message: String,
    subscription: &web_push_subscription::Model,
    settings: &Settings,
) -> Result<Vec<u8>, UseCaseError> {
    let p256dh_key = BASE64_URL_SAFE_NO_PAD
        .decode(
            decode_and_decrypt(subscription.p256dh_key.clone(), &settings)
                .map_err(|e| UseCaseError::InternalServerError(e))?,
        )
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
    let auth_key = BASE64_URL_SAFE_NO_PAD
        .decode(
            decode_and_decrypt(subscription.auth_key.clone(), &settings)
                .map_err(|e| UseCaseError::InternalServerError(e))?,
        )
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
    encrypt(&p256dh_key, &auth_key, message.as_bytes())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

fn get_jwt_token(
    vapid_key: &ES256KeyPair,
    endpoint: Uri,
    settings: &Settings,
) -> Result<String, UseCaseError> {
    let mut jwt_token_claims = Claims::with_custom_claims(
        BTreeMap::<String, String>::new(),
        Duration::from_secs(TTL_SECONDS),
    );
    jwt_token_claims.custom.insert(
        "aud".to_string(),
        format!(
            "{}://{}",
            endpoint.scheme_str().unwrap(),
            endpoint.host().unwrap()
        )
        .into(),
    );
    jwt_token_claims.custom.insert(
        "sub".to_string(),
        format!("mailto:{}", settings.application.app_owner_email.clone()),
    );
    vapid_key
        .sign(jwt_token_claims)
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

fn get_authorization_header(endpoint: &str, settings: &Settings) -> Result<String, UseCaseError> {
    let vapid_key = ES256KeyPair::from_bytes(
        &BASE64_URL_SAFE_NO_PAD
            .decode(settings.application.vapid_private_key.clone())
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?,
    )
    .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let jwt_token = get_jwt_token(
        &vapid_key,
        endpoint
            .try_into()
            .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?,
        settings,
    )?;

    Ok(format!(
        "vapid t={}, k={}",
        jwt_token,
        BASE64_URL_SAFE_NO_PAD.encode(&vapid_key.public_key().public_key().to_bytes_uncompressed()),
    ))
}

async fn send_push_request(
    message: Vec<u8>,
    endpoint: &str,
    authorization_header: &str,
) -> Result<(), UseCaseError> {
    let config = rustls_0_23::reexports::ClientConfig::builder()
        .with_root_certificates(rustls_0_23::webpki_roots_cert_store())
        .with_no_client_auth();
    let client = awc::Client::builder()
        .connector(awc::Connector::new().rustls_0_23(Arc::new(config)))
        .finish();
    client
        .post(endpoint)
        .content_type("application/octet-stream")
        .insert_header(("Authorization", authorization_header))
        .insert_header(("Content-Encoding", "aes128gcm"))
        .insert_header(("TTL", TTL_SECONDS))
        .insert_header(("Urgency", "normal"))
        .send_body(message)
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;
    Ok(())
}
