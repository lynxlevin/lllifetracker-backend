use http::{
    header::{InvalidHeaderValue, AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE},
    HeaderMap, HeaderValue, StatusCode,
};
use reqwest::Error as ReqwestError;
use serde::Serialize;
use serde_json::json;
use thiserror::Error;

use common::{db::decode_and_decrypt, settings::types::Settings};
use entities::web_push_subscription;

use crate::notification::utils::web_push_messenger::{
    web_push_message_encryptor::{MessageEncryptor, MessageEncryptorError},
    web_push_vapid_signature_builder::{VapidSignatureBuilder, VapidSignatureBuilderError},
};

const TTL_SECONDS: u64 = 60 * 60 * 23;

#[derive(Serialize, Debug)]
pub struct Message {
    pub title: Option<String>,
    pub body: String,
    pub path: Option<String>,
}

pub struct WebPushMessenger {
    endpoint: String,
    message_encryptor: MessageEncryptor,
    vapid_signature_builder: VapidSignatureBuilder,
}

pub enum WebPushMessengerResult {
    OK,
    InvalidSubscription,
}

#[derive(Debug, Error)]
pub enum WebPushMessengerError {
    #[error("WebPushMessengerError:InitializationError:{0}")]
    InitializationError(String),
    #[error("WebPushMessengerError:MessageEncryptorError:{0}")]
    MessageEncryptorError(MessageEncryptorError),
    #[error("WebPushMessengerError:VapidSignatureBuilderError:{0}")]
    VapidSignatureBuilderError(VapidSignatureBuilderError),
    #[error("WebPushMessengerError:InvalidHeaderValue:{0}")]
    InvalidHeaderValue(InvalidHeaderValue),
    #[error("WebPushMessengerError:RequestError:{0}")]
    RequestError(ReqwestError),
}

impl WebPushMessenger {
    pub fn new(
        subscription: &web_push_subscription::Model,
        settings: &Settings,
    ) -> Result<Self, WebPushMessengerError> {
        Ok(Self {
            endpoint: decode_and_decrypt(subscription.endpoint.clone(), &settings)
                .map_err(|e| WebPushMessengerError::InitializationError(e.to_string()))?,
            message_encryptor: MessageEncryptor::new(subscription, settings)
                .map_err(|e| WebPushMessengerError::MessageEncryptorError(e))?,
            vapid_signature_builder: VapidSignatureBuilder::new(settings)
                .map_err(|e| WebPushMessengerError::VapidSignatureBuilderError(e))?,
        })
    }

    pub async fn send_message(
        &self,
        message: Message,
    ) -> Result<WebPushMessengerResult, WebPushMessengerError> {
        let encrypted_message = self.encrypt_message(message)?;

        // NOTE: Http client cannot be awc because it causes "future is not Send" error in run_cron_processes.
        // This is inevitable though not ideal because reqwest is hyper-based whereas actix-web is tokio-based.
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/octet-stream")
                .map_err(|e| WebPushMessengerError::InvalidHeaderValue(e))?,
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(
                &self
                    .vapid_signature_builder
                    .build(&self.endpoint, TTL_SECONDS)
                    .map_err(|e| WebPushMessengerError::VapidSignatureBuilderError(e))?,
            )
            .map_err(|e| WebPushMessengerError::InvalidHeaderValue(e))?,
        );
        headers.insert(
            CONTENT_ENCODING,
            HeaderValue::from_str("aes128gcm")
                .map_err(|e| WebPushMessengerError::InvalidHeaderValue(e))?,
        );
        headers.insert("TTL", HeaderValue::from(TTL_SECONDS));
        headers.insert(
            "Urgency",
            HeaderValue::from_str("normal")
                .map_err(|e| WebPushMessengerError::InvalidHeaderValue(e))?,
        );
        client
            .post(&self.endpoint)
            .headers(headers)
            .body(encrypted_message)
            .send()
            .await
            .map(|res| match res.status() {
                StatusCode::NOT_FOUND | StatusCode::GONE => {
                    WebPushMessengerResult::InvalidSubscription
                }
                _ => WebPushMessengerResult::OK,
            })
            .map_err(|e| WebPushMessengerError::RequestError(e))
    }

    pub fn encrypt_message(&self, message: Message) -> Result<Vec<u8>, WebPushMessengerError> {
        let message = json!(message).to_string();
        self.message_encryptor
            .encrypt(message)
            .map_err(|e| WebPushMessengerError::MessageEncryptorError(e))
    }
}
