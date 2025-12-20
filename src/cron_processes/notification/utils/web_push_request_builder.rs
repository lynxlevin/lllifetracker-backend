use actix_tls::connect::rustls_0_23;
use aes_gcm::{
    aead::{
        consts as aes_gcm_consts, generic_array, rand_core::RngCore, AeadMutInPlace, Buffer, OsRng,
    },
    Aes128Gcm, Key, KeyInit, Nonce,
};
use awc::ClientRequest;
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use hkdf::Hkdf;
use http::Uri;
use jwt_simple::prelude::{
    Claims, Duration, ECDSAP256KeyPairLike, ECDSAP256PublicKeyLike, ES256KeyPair,
};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use serde::Serialize;
use serde_json::json;
use sha2::Sha256;
use std::{collections::BTreeMap, error::Error, fmt::Display, sync::Arc};

use common::{db::decode_and_decrypt, settings::types::Settings};
use entities::web_push_subscription;

const TTL_SECONDS: u64 = 60 * 60 * 23;

#[derive(Serialize)]
struct Message {
    body: String,
}

pub struct WebPushRequestBuilder {
    endpoint: String,
    message_encryptor: MessageEncryptor,
    vapid_signature_builder: VapidSignatureBuilder,
}

#[derive(Debug)]
pub struct WebPushRequestBuilderError {
    method_detail: String,
    error: String,
}
impl WebPushRequestBuilderError {
    fn new(method_detail: impl ToString, error: impl ToString) -> Self {
        Self {
            method_detail: method_detail.to_string(),
            error: error.to_string(),
        }
    }
}
impl Display for WebPushRequestBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.method_detail, &self.error)
    }
}
impl Error for WebPushRequestBuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl WebPushRequestBuilder {
    pub fn new(
        subscription: &web_push_subscription::Model,
        settings: &Settings,
    ) -> Result<Self, WebPushRequestBuilderError> {
        Ok(Self {
            endpoint: decode_and_decrypt(subscription.endpoint.clone(), &settings)
                .map_err(|e| WebPushRequestBuilderError::new("WebPushRequestBuilder::new", e))?,
            message_encryptor: MessageEncryptor::new(subscription, settings)?,
            vapid_signature_builder: VapidSignatureBuilder::new(settings)?,
        })
    }

    pub fn encrypt_message(
        &self,
        message: impl ToString,
    ) -> Result<Vec<u8>, WebPushRequestBuilderError> {
        let message = json!(Message {
            body: message.to_string()
        })
        .to_string();
        self.message_encryptor.encrypt(message)
    }

    pub fn get_awc_client(
        &self,
        ttl_seconds: Option<u64>,
    ) -> Result<ClientRequest, WebPushRequestBuilderError> {
        let config = rustls_0_23::reexports::ClientConfig::builder()
            .with_root_certificates(rustls_0_23::webpki_roots_cert_store())
            .with_no_client_auth();
        let client = awc::Client::builder()
            .connector(awc::Connector::new().rustls_0_23(Arc::new(config)))
            .finish();
        Ok(client
            .post(&self.endpoint)
            .content_type("application/octet-stream")
            .insert_header((
                "Authorization",
                self.vapid_signature_builder.build(&self.endpoint)?,
            ))
            .insert_header(("Content-Encoding", "aes128gcm"))
            .insert_header(("TTL", ttl_seconds.unwrap_or(TTL_SECONDS)))
            .insert_header(("Urgency", "normal")))
    }
}

struct MessageEncryptor {
    p256dh_key: Vec<u8>,
    auth_key: Vec<u8>,
}

impl MessageEncryptor {
    fn new(
        subscription: &web_push_subscription::Model,
        settings: &Settings,
    ) -> Result<Self, WebPushRequestBuilderError> {
        Ok(Self {
            p256dh_key: BASE64_URL_SAFE_NO_PAD
                .decode(
                    decode_and_decrypt(subscription.p256dh_key.clone(), &settings)
                        .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::new", e))?,
                )
                .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::new", e))?,
            auth_key: BASE64_URL_SAFE_NO_PAD
                .decode(
                    decode_and_decrypt(subscription.auth_key.clone(), &settings)
                        .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::new", e))?,
                )
                .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::new", e))?,
        })
    }

    fn derive_key<IKM: AsRef<[u8]>>(&self, salt: [u8; 16], ikm: IKM) -> Key<Aes128Gcm> {
        let mut okm = [0u8; 16];
        let hk = Hkdf::<Sha256>::new(Some(&salt), ikm.as_ref());
        hk.expand(b"Content-Encoding: aes128gcm\0", &mut okm)
            .expect("okm length is always 16, impossible for it to be too large");
        Key::<Aes128Gcm>::from(okm)
    }

    fn derive_nonce<IKM: AsRef<[u8]>>(
        &self,
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
        &self,
        key: &Key<Aes128Gcm>,
        nonce: &Nonce<aes_gcm_consts::U12>,
        mut record: B,
        encrypted_record_size: u32,
    ) -> Result<B, WebPushRequestBuilderError> {
        let plain_record_size: u32 = record
            .len()
            .try_into()
            .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::encrypt_record", e))?;
        if plain_record_size >= encrypted_record_size - 16 {
            return Err(WebPushRequestBuilderError::new(
                "MessageEncryptor::encrypt_record",
                "RecordLengthInvalid",
            ));
        }

        record
            .extend_from_slice(b"\x02")
            .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::encrypt_record", e))?;

        Aes128Gcm::new(key)
            .encrypt_in_place(nonce, b"", &mut record)
            .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::encrypt_record", e))?;
        Ok(record)
    }

    fn encrypt(&self, message: String) -> Result<Vec<u8>, WebPushRequestBuilderError> {
        let message: Vec<u8> = message.as_bytes().into();
        let p256_public_key = p256::PublicKey::from_sec1_bytes(&self.p256dh_key)
            .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::encrypt", e))?;
        let auth = generic_array::GenericArray::<u8, generic_array::typenum::U16>::clone_from_slice(
            &self.auth_key,
        );

        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        let as_secret = p256::SecretKey::random(&mut OsRng);
        let as_public = as_secret.public_key();

        let mut info = vec![];
        info.extend_from_slice(b"WebPush: info");
        info.push(0u8);
        info.extend_from_slice(
            p256_public_key
                .as_affine()
                .to_encoded_point(false)
                .as_bytes(),
        );
        info.extend_from_slice(as_public.as_affine().to_encoded_point(false).as_bytes());

        let mut ikm = [0u8; 32];
        let shared =
            p256::ecdh::diffie_hellman(as_secret.to_nonzero_scalar(), p256_public_key.as_affine());
        let hk = Hkdf::<Sha256>::new(Some(&auth), &shared.raw_secret_bytes().as_ref());
        hk.expand(&info, &mut ikm)
            .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::encrypt", e))?;

        let key_id = as_public.as_affine().to_encoded_point(false);
        let encrypted_record_length: u32 = (message.len() + 17)
            .try_into()
            .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::encrypt", e))?;

        let mut seq = [0u8; 12];
        seq[4..].copy_from_slice(&0_usize.to_be_bytes());
        let key = self.derive_key(salt, ikm.as_ref());
        let nonce = self.derive_nonce(salt, ikm.as_ref(), seq);

        let mut output = vec![];
        output.extend_from_slice(&salt);
        output.extend_from_slice(&encrypted_record_length.to_be_bytes());
        output.push(
            key_id
                .len()
                .try_into()
                .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::encrypt", e))?,
        );
        output.extend_from_slice(key_id.as_ref());

        let record = self
            .encrypt_record(&key, &nonce, message, encrypted_record_length)
            .map_err(|e| WebPushRequestBuilderError::new("MessageEncryptor::encrypt", e))?;
        output.extend_from_slice(&record);

        Ok(output)
    }
}

struct VapidSignatureBuilder {
    vapid_private_key: Vec<u8>,
    app_owner_email: String,
}

impl VapidSignatureBuilder {
    fn new(settings: &Settings) -> Result<Self, WebPushRequestBuilderError> {
        Ok(Self {
            vapid_private_key: BASE64_URL_SAFE_NO_PAD
                .decode(settings.application.vapid_private_key.clone())
                .map_err(|e| WebPushRequestBuilderError::new("VapidSignatureBuilder::new", e))?,
            app_owner_email: settings.application.app_owner_email.clone(),
        })
    }

    fn build_jwt(&self, endpoint: Uri) -> Result<String, WebPushRequestBuilderError> {
        let vapid_key = ES256KeyPair::from_bytes(&self.vapid_private_key)
            .map_err(|e| WebPushRequestBuilderError::new("VapidSignatureBuilder::build_jwt", e))?;

        let mut jwt_claims = Claims::with_custom_claims(
            BTreeMap::<String, String>::new(),
            Duration::from_secs(TTL_SECONDS),
        );
        jwt_claims.custom.insert(
            "aud".to_string(),
            format!(
                "{}://{}",
                endpoint.scheme_str().unwrap(),
                endpoint.host().unwrap()
            )
            .into(),
        );
        jwt_claims.custom.insert(
            "sub".to_string(),
            format!("mailto:{}", &self.app_owner_email),
        );
        vapid_key
            .sign(jwt_claims)
            .map_err(|e| WebPushRequestBuilderError::new("VapidSignatureBuilder::build_jwt", e))
    }

    fn build(&self, endpoint: &str) -> Result<String, WebPushRequestBuilderError> {
        let vapid_key = ES256KeyPair::from_bytes(&self.vapid_private_key)
            .map_err(|e| WebPushRequestBuilderError::new("VapidSignatureBuilder::build", e))?;
        let jwt =
            self.build_jwt(endpoint.try_into().map_err(|e| {
                WebPushRequestBuilderError::new("VapidSignatureBuilder::build", e)
            })?)?;

        Ok(format!(
            "vapid t={}, k={}",
            jwt,
            BASE64_URL_SAFE_NO_PAD
                .encode(&vapid_key.public_key().public_key().to_bytes_uncompressed()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::prelude::BASE64_URL_SAFE_NO_PAD;
    use common::{
        factory::{self, *},
        settings::get_test_settings,
    };
    use uuid::Uuid;

    #[test]
    fn test_encrypt_message() {
        let (key_pair, auth) = ece::generate_keypair_and_auth_secret().unwrap();
        let p256dh_key = key_pair.pub_as_raw().unwrap();

        let settings = get_test_settings();
        let subscription = factory::web_push_subscription(Uuid::now_v7(), &settings)
            .encrypt_and_save_p256dh_key(BASE64_URL_SAFE_NO_PAD.encode(p256dh_key), &settings)
            .encrypt_and_save_auth_key(BASE64_URL_SAFE_NO_PAD.encode(auth), &settings)
            .get_model();
        let message = "Encrypt this message, please.";

        let encryptor = MessageEncryptor::new(&subscription, &settings).unwrap();
        let encrypted = encryptor.encrypt(message.to_string()).unwrap();

        assert_eq!(
            message.as_bytes(),
            ece::decrypt(&key_pair.raw_components().unwrap(), &auth, &encrypted).unwrap()
        );
    }
}
