use aes_gcm::{
    aead::{
        consts as aes_gcm_consts, generic_array, rand_core::RngCore, AeadMutInPlace, Buffer, OsRng,
    },
    Aes128Gcm, Key, KeyInit, Nonce,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use common::{db::decode_and_decrypt, settings::types::Settings};
use entities::web_push_subscription;
use hkdf::Hkdf;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use sha2::Sha256;

use crate::{error_500, UseCaseError};

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
pub fn encrypt_message(
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
    fn test_encryption() {
        let (key_pair, auth) = ece::generate_keypair_and_auth_secret().unwrap();
        let p256dh_key = key_pair.pub_as_raw().unwrap();

        let settings = get_test_settings();
        let subscription = factory::web_push_subscription(Uuid::now_v7(), &settings)
            .encrypt_and_save_p256dh_key(BASE64_URL_SAFE_NO_PAD.encode(p256dh_key), &settings)
            .encrypt_and_save_auth_key(BASE64_URL_SAFE_NO_PAD.encode(auth), &settings)
            .get_model();
        let message = "Encrypt this message, please.";

        let encrypted = encrypt_message(message.to_string(), &subscription, &settings).unwrap();

        assert_eq!(
            message.as_bytes(),
            ece::decrypt(&key_pair.raw_components().unwrap(), &auth, &encrypted).unwrap()
        );
    }
}
