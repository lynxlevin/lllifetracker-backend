use std::{collections::BTreeMap, fmt::Debug, sync::Arc};

use actix_tls::connect::rustls_0_23;
use aes_gcm::{
    aead::{
        consts as aes_gcm_consts, generic_array, rand_core::RngCore, AeadMutInPlace, Buffer, OsRng,
    },
    Aes128Gcm, Key, KeyInit, Nonce,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use common::{db::decode_and_decrypt, settings::types::Settings};
use db_adapters::{
    ambition_adapter::{AmbitionAdapter, AmbitionFilter, AmbitionQuery},
    desired_state_adapter::{DesiredStateAdapter, DesiredStateFilter, DesiredStateQuery},
    web_push_subscription_adapter::{WebPushSubscriptionAdapter, WebPushSubscriptionQuery},
};
use entities::{user as user_entity, web_push_subscription};
use hkdf::Hkdf;
use http::Uri;
use jwt_simple::{
    prelude::{Claims, Duration, ECDSAP256KeyPairLike, ECDSAP256PublicKeyLike, ES256KeyPair},
    reexports::rand::{seq::SliceRandom, thread_rng},
};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use serde::Serialize;
use serde_json::json;
use sha2::Sha256;

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
                .map_err(error_500)?
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
                .map_err(error_500)?
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
        .map_err(error_500)?
        .ok_or(UseCaseError::NotFound(
            "You don't have active web push subscription.".to_string(),
        ))?;
    let endpoint =
        decode_and_decrypt(subscription.endpoint.clone(), &settings).map_err(error_500)?;

    let encrypted_message = encrypt_message(json!(message).to_string(), &subscription, &settings)?;

    let authorization_header = get_authorization_header(&endpoint, &settings)?;

    send_push_request(encrypted_message, &endpoint, &authorization_header).await?;

    Ok(())
}

fn error_500(e: impl Debug) -> UseCaseError {
    UseCaseError::InternalServerError(format!("{:?}", e))
}

fn derive_key<IKM: AsRef<[u8]>>(salt: [u8; 16], ikm: IKM) -> Key<Aes128Gcm> {
    let mut okm = [0u8; 16];
    let hk = Hkdf::<Sha256>::new(Some(&salt), ikm.as_ref());
    hk.expand(b"Content-Encoding: aes128gcm\0", &mut okm)
        .expect("okm length is always 16, impossible for it to be too large");
    Key::<Aes128Gcm>::from(okm)
}

fn derive_nonce<IKM: AsRef<[u8]>>(
    salt: [u8; 16],
    ikm: IKM,
    seq: [u8; 12],
) -> Nonce<aes_gcm_consts::U12> {
    let mut okm = [0u8; 12];
    let hk = Hkdf::<Sha256>::new(Some(&salt), ikm.as_ref());
    hk.expand(b"Content-Encoding: nonce\0", &mut okm)
        .expect("okm length is always 16, impossible for it to be too large");
    for i in 0..12 {
        okm[i] ^= seq[i]
    }
    Nonce::from(okm)
}

fn encrypt_record<B: Buffer>(
    key: &Key<Aes128Gcm>,
    nonce: &Nonce<aes_gcm_consts::U12>,
    mut record: B,
    encrypted_record_size: u32,
    is_last: bool,
) -> Result<B, UseCaseError> {
    let plain_record_size: u32 = record.len().try_into().map_err(error_500)?;
    if plain_record_size >= encrypted_record_size - 16 {
        return Err(error_500("RecordLengthInvalid".to_string()));
    }

    if is_last {
        record.extend_from_slice(b"\x02").map_err(error_500)?;
    } else {
        let pad_len = encrypted_record_size - plain_record_size - 16;
        record.extend_from_slice(b"\x01").map_err(error_500)?;
        record
            .extend_from_slice(
                &b"\x00".repeat(
                    (pad_len - 1).try_into().expect(
                        "padding length is between 0 and 15 which wil always fit into usize",
                    ),
                ),
            )
            .map_err(error_500)?;
    }
    Aes128Gcm::new(key)
        .encrypt_in_place(nonce, b"", &mut record)
        .map_err(error_500)?;
    Ok(record)
}

// MYMEMO: Refactor these into a builder
fn encrypt_message(
    message: String,
    subscription: &web_push_subscription::Model,
    settings: &Settings,
) -> Result<Vec<u8>, UseCaseError> {
    let p256dh_key = BASE64_URL_SAFE_NO_PAD
        .decode(decode_and_decrypt(subscription.p256dh_key.clone(), &settings).map_err(error_500)?)
        .map_err(error_500)?;
    let p256_public_key = p256::PublicKey::from_sec1_bytes(&p256dh_key).map_err(error_500)?;

    let auth_key = BASE64_URL_SAFE_NO_PAD
        .decode(decode_and_decrypt(subscription.auth_key.clone(), &settings).map_err(error_500)?)
        .map_err(error_500)?;
    let auth =
        generic_array::GenericArray::<u8, generic_array::typenum::U16>::clone_from_slice(&auth_key);

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let as_secret = p256::SecretKey::random(&mut OsRng);
    let as_public = as_secret.public_key();
    let shared =
        p256::ecdh::diffie_hellman(as_secret.to_nonzero_scalar(), p256_public_key.as_affine());
    let mut info = vec![];
    info.extend_from_slice(&b"WebPush: info"[..]);
    info.push(0u8);
    info.extend_from_slice(
        &p256_public_key
            .as_affine()
            .to_encoded_point(false)
            .as_bytes(),
    );
    info.extend_from_slice(&as_public.as_affine().to_encoded_point(false).as_bytes());
    let mut ikm = [0u8; 32];
    let hk = Hkdf::<Sha256>::new(Some(&auth), &shared.raw_secret_bytes().as_ref());
    hk.expand(&info, &mut ikm).map_err(error_500)?;
    let key_id = as_public.as_affine().to_encoded_point(false);
    let encrypted_record_length: u32 = (message.len() + 17).try_into().map_err(error_500)?;

    let message: Vec<u8> = message.as_bytes().into();
    let records = Some(message).into_iter().enumerate().map(|(n, record)| {
        let mut seq = [0u8; 12];
        seq[4..].copy_from_slice(&n.to_be_bytes());
        let key = derive_key(salt, ikm.as_ref());
        let nonce = derive_nonce(salt, ikm.as_ref(), seq);
        (key, nonce, record)
    });
    let mut output = vec![];
    let mut header = vec![];
    header.extend_from_slice(&salt[..]);
    header.extend_from_slice(&encrypted_record_length.to_be_bytes());
    header.push(key_id.as_ref().len().try_into().map_err(error_500)?);
    header.extend_from_slice(key_id.as_ref());
    output.extend_from_slice(&header);
    let mut peekable = records.peekable();
    while let Some((key, nonce, record)) = peekable.next() {
        let is_last_record = peekable.peek().is_none();
        let record = encrypt_record(
            &key,
            &nonce,
            record,
            encrypted_record_length,
            is_last_record,
        )
        .map_err(error_500)?;
        output.extend_from_slice(&record);
    }

    Ok(output)
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
    vapid_key.sign(jwt_token_claims).map_err(error_500)
}

fn get_authorization_header(endpoint: &str, settings: &Settings) -> Result<String, UseCaseError> {
    let vapid_key = ES256KeyPair::from_bytes(
        &BASE64_URL_SAFE_NO_PAD
            .decode(settings.application.vapid_private_key.clone())
            .map_err(error_500)?,
    )
    .map_err(error_500)?;

    let jwt_token = get_jwt_token(
        &vapid_key,
        endpoint.try_into().map_err(error_500)?,
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
        .map_err(error_500)?;
    Ok(())
}
