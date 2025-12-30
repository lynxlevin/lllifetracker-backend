use aes_gcm::{
    aead::{
        consts as aes_gcm_consts, generic_array, rand_core::RngCore, AeadMutInPlace, Buffer,
        Error as AEADError, OsRng,
    },
    Aes128Gcm, Key, KeyInit, Nonce,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, DecodeError, Engine};
use hkdf::Hkdf;
use p256::elliptic_curve::{sec1::ToEncodedPoint, Error as P256Error};
use sha2::Sha256;
use thiserror::Error;

use common::{db::decode_and_decrypt, settings::types::Settings};
use entities::web_push_subscription;

#[derive(Debug, Error)]
pub enum MessageEncryptorError {
    #[error("Base64DecodeError: {0}")]
    Base64DecodeError(DecodeError),
    #[error("Aes128GcmEncryptError: {0}")]
    Aes128GcmEncryptError(AEADError),
    #[error("InternalError: {0}")]
    InternalError(String),
    #[error("InvalidP256DHKey: {0}")]
    InvalidP256DHKey(P256Error),
    #[error("Generic error message: {0}")]
    Error(String),
}

pub struct MessageEncryptor {
    p256dh_key: Vec<u8>,
    auth_key: Vec<u8>,
}

impl MessageEncryptor {
    pub fn new(
        subscription: &web_push_subscription::Model,
        settings: &Settings,
    ) -> Result<Self, MessageEncryptorError> {
        Ok(Self {
            p256dh_key: BASE64_URL_SAFE_NO_PAD
                .decode(
                    decode_and_decrypt(subscription.p256dh_key.clone(), &settings)
                        .map_err(|e| MessageEncryptorError::Error(e))?,
                )
                .map_err(|e| MessageEncryptorError::Base64DecodeError(e))?,
            auth_key: BASE64_URL_SAFE_NO_PAD
                .decode(
                    decode_and_decrypt(subscription.auth_key.clone(), &settings)
                        .map_err(|e| MessageEncryptorError::Error(e))?,
                )
                .map_err(|e| MessageEncryptorError::Base64DecodeError(e))?,
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
    ) -> Result<B, MessageEncryptorError> {
        let plain_record_size: u32 = record.len().try_into().map_err(|e| {
            MessageEncryptorError::InternalError(format!("Invalid record length: {:?}", e))
        })?;
        if plain_record_size >= encrypted_record_size - 16 {
            return Err(MessageEncryptorError::InternalError(
                "Invalid record length, plain_record longer than encrypted_record".to_string(),
            ));
        }

        record.extend_from_slice(b"\x02").map_err(|e| {
            MessageEncryptorError::InternalError(format!("Record extend error: {:?}", e))
        })?;

        Aes128Gcm::new(key)
            .encrypt_in_place(nonce, b"", &mut record)
            .map_err(|e| MessageEncryptorError::Aes128GcmEncryptError(e))?;
        Ok(record)
    }

    pub fn encrypt(&self, message: String) -> Result<Vec<u8>, MessageEncryptorError> {
        let message: Vec<u8> = message.as_bytes().into();
        let p256_public_key = p256::PublicKey::from_sec1_bytes(&self.p256dh_key)
            .map_err(|e| MessageEncryptorError::InvalidP256DHKey(e))?;
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
            .map_err(|e| MessageEncryptorError::InternalError(e.to_string()))?;

        let key_id = as_public.as_affine().to_encoded_point(false);
        let encrypted_record_length: u32 = (message.len() + 17).try_into().map_err(|e| {
            MessageEncryptorError::InternalError(format!("Invalid message length: {:?}", e))
        })?;

        let mut seq = [0u8; 12];
        seq[4..].copy_from_slice(&0_usize.to_be_bytes());
        let key = self.derive_key(salt, ikm.as_ref());
        let nonce = self.derive_nonce(salt, ikm.as_ref(), seq);

        let mut output = vec![];
        output.extend_from_slice(&salt);
        output.extend_from_slice(&encrypted_record_length.to_be_bytes());
        output.push(key_id.len().try_into().map_err(|e| {
            MessageEncryptorError::InternalError(format!("Invalid key_id length: {:?}", e))
        })?);
        output.extend_from_slice(key_id.as_ref());

        let record = self.encrypt_record(&key, &nonce, message, encrypted_record_length)?;
        output.extend_from_slice(&record);

        Ok(output)
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
    fn test_message_encryptor() {
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
