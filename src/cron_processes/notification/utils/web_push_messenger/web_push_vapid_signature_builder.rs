use base64::{prelude::BASE64_URL_SAFE_NO_PAD, DecodeError, Engine};
use http::{uri::InvalidUri, Uri};
use jwt_simple::{
    prelude::{Claims, Duration, ECDSAP256KeyPairLike, ECDSAP256PublicKeyLike, ES256KeyPair},
    Error as JWTGenericError,
};
use std::collections::BTreeMap;
use thiserror::Error;

use common::settings::types::Settings;

#[derive(Debug, Error)]
pub enum VapidSignatureBuilderError {
    #[error("Base64DecodeError: {0}")]
    Base64DecodeError(DecodeError),
    #[error("EndpointParseError: {0}")]
    EndpointParseError(InvalidUri),
    #[error("Error parsing vapid_private_key: {0}")]
    VapidKeyParseError(JWTGenericError),
    #[error("Error generating vapid signature: {0}")]
    SigningError(JWTGenericError),
}

pub struct VapidSignatureBuilder {
    vapid_private_key: Vec<u8>,
    app_owner_email: String,
}

impl VapidSignatureBuilder {
    pub fn new(settings: &Settings) -> Result<Self, VapidSignatureBuilderError> {
        Ok(Self {
            vapid_private_key: BASE64_URL_SAFE_NO_PAD
                .decode(settings.application.vapid_private_key.clone())
                .map_err(|e| VapidSignatureBuilderError::Base64DecodeError(e))?,
            app_owner_email: settings.application.app_owner_email.clone(),
        })
    }

    fn build_jwt(
        &self,
        endpoint: Uri,
        ttl_seconds: u64,
    ) -> Result<String, VapidSignatureBuilderError> {
        let vapid_key = ES256KeyPair::from_bytes(&self.vapid_private_key)
            .map_err(|e| VapidSignatureBuilderError::VapidKeyParseError(e))?;

        let mut jwt_claims = Claims::with_custom_claims(
            BTreeMap::<String, String>::new(),
            Duration::from_secs(ttl_seconds),
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
            .map_err(|e| VapidSignatureBuilderError::SigningError(e))
    }

    pub fn build(
        &self,
        endpoint: &str,
        ttl_seconds: u64,
    ) -> Result<String, VapidSignatureBuilderError> {
        let vapid_key = ES256KeyPair::from_bytes(&self.vapid_private_key)
            .map_err(|e| VapidSignatureBuilderError::VapidKeyParseError(e))?;
        let jwt = self.build_jwt(
            endpoint
                .try_into()
                .map_err(|e| VapidSignatureBuilderError::EndpointParseError(e))?,
            ttl_seconds,
        )?;

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
    use common::settings::get_test_settings;
    use jwt_simple::claims::{Audiences, NoCustomClaims};

    #[test]
    fn test_vapid_signature_builder() {
        let settings = get_test_settings();
        let builder = VapidSignatureBuilder::new(&settings).unwrap();
        let endpoint = "https://dummy-endpoint.com";

        let signature = builder.build(endpoint, 100).unwrap();
        let mut signature_parts = signature.split(' ');

        let signature_start = signature_parts.next().unwrap();
        let signature_t = signature_parts.next().unwrap();
        let signature_k = signature_parts.next().unwrap();
        assert_eq!(signature_parts.next(), None);

        assert_eq!(signature_start, "vapid");
        assert_eq!(
            signature_k,
            "k=BFOdTFXaneR5amG3nD6S5lqvZUK0melyCApPy7vGGpbsagm65TfZHWHUtHIHYvP5pa_PDaKn0_364U0gTVecPQw"
        );

        // signature_t assertion
        let vapid_private_key = BASE64_URL_SAFE_NO_PAD
            .decode(settings.application.vapid_private_key.clone())
            .unwrap();
        let vapid_key = ES256KeyPair::from_bytes(&vapid_private_key)
            .unwrap()
            .public_key();
        let token = &signature_t[2..284];
        let claims = vapid_key
            .verify_token::<NoCustomClaims>(token, None)
            .unwrap();
        assert_eq!(
            claims.subject,
            Some(format!("mailto:{}", settings.application.app_owner_email))
        );
        assert_eq!(
            claims.audiences,
            Some(Audiences::AsString(endpoint.to_string()))
        );
    }
}
