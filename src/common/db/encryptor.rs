use crate::settings::types::Settings;
use aes_gcm::{
    aead::{
        consts::{B0, B1},
        Aead,
    },
    aes::{
        cipher::typenum::{UInt, UTerm},
        Aes256,
    },
    Aes256Gcm, AesGcm, KeyInit,
};
use base64::{engine::general_purpose, Engine};

fn get_cipher_and_nonce(
    settings: &Settings,
) -> Result<
    (
        AesGcm<Aes256, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>,
        Vec<u8>,
    ),
    String,
> {
    let key = general_purpose::STANDARD
        .decode(&settings.database.encryption_key)
        .map_err(|e| format!("Error base64-decoding encryption_key: {}", e))?;
    let nonce = general_purpose::STANDARD
        .decode(&settings.database.encryption_nonce)
        .map_err(|e| format!("Error base64-decoding encryption_key: {}", e))?;
    let cipher = Aes256Gcm::new(key.as_slice().into());
    Ok((cipher, nonce))
}

pub fn encrypt_and_encode(text: String, settings: &Settings) -> Result<String, String> {
    let (cipher, nonce) = get_cipher_and_nonce(settings)?;
    let cipher_text = cipher
        .encrypt(nonce.as_slice().into(), text.as_bytes())
        .map_err(|e| format!("Error encrypting text: {}", e))?;
    Ok(general_purpose::STANDARD.encode(cipher_text))
}

pub fn decode_and_decrypt(cipher_text: String, settings: &Settings) -> Result<String, String> {
    let (cipher, nonce) = get_cipher_and_nonce(settings)?;
    let decoded_cipher_text = general_purpose::STANDARD
        .decode(cipher_text)
        .map_err(|e| format!("Error base64-decoding cipher_text: {}", e))?;
    let plain_text = cipher
        .decrypt(nonce.as_slice().into(), decoded_cipher_text.as_ref())
        .map_err(|e| format!("Error decrypting cipher_text: {}", e))?;
    String::from_utf8(plain_text)
        .map_err(|e| format!("Error utf8-encoding decrypted text: {}", e.utf8_error()))
}
