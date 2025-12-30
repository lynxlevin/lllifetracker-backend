use http::{
    header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_TYPE},
    HeaderMap, HeaderValue, StatusCode,
};
use serde::Serialize;
use serde_json::json;
use std::{error::Error, fmt::Display};

use common::{db::decode_and_decrypt, settings::types::Settings};
use entities::web_push_subscription;

use crate::notification::utils::web_push_messenger::{
    web_push_message_encryptor::MessageEncryptor,
    web_push_vapid_signature_builder::VapidSignatureBuilder,
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

#[derive(Debug)]
// MYMEMO: This should be changed to enum error
pub struct WebPushMessengerError {
    method_detail: String,
    error: String,
}
impl WebPushMessengerError {
    pub fn new(method_detail: impl ToString, error: impl ToString) -> Self {
        Self {
            method_detail: method_detail.to_string(),
            error: error.to_string(),
        }
    }
}
impl Display for WebPushMessengerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", &self.method_detail, &self.error)
    }
}
impl Error for WebPushMessengerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl WebPushMessenger {
    pub fn new(
        subscription: &web_push_subscription::Model,
        settings: &Settings,
    ) -> Result<Self, WebPushMessengerError> {
        Ok(Self {
            endpoint: decode_and_decrypt(subscription.endpoint.clone(), &settings)
                .map_err(|e| WebPushMessengerError::new("WebPushMessenger::new", e))?,
            message_encryptor: MessageEncryptor::new(subscription, settings)?,
            vapid_signature_builder: VapidSignatureBuilder::new(settings)?,
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
                .map_err(|e| WebPushMessengerError::new("WebPushMessenger::send_message", e))?,
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(
                &self
                    .vapid_signature_builder
                    .build(&self.endpoint, TTL_SECONDS)?,
            )
            .map_err(|e| WebPushMessengerError::new("WebPushMessenger::send_message", e))?,
        );
        headers.insert(
            CONTENT_ENCODING,
            HeaderValue::from_str("aes128gcm")
                .map_err(|e| WebPushMessengerError::new("WebPushMessenger::send_message", e))?,
        );
        headers.insert("TTL", HeaderValue::from(TTL_SECONDS));
        headers.insert(
            "Urgency",
            HeaderValue::from_str("normal")
                .map_err(|e| WebPushMessengerError::new("WebPushMessenger::send_message", e))?,
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
            .map_err(|e| WebPushMessengerError::new("WebPushMessenger::send_message", e))
    }

    pub fn encrypt_message(&self, message: Message) -> Result<Vec<u8>, WebPushMessengerError> {
        let message = json!(message).to_string();
        self.message_encryptor.encrypt(message)
    }
}
