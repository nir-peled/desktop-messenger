use reqwest::{Client, RequestBuilder, Response};
use serde_json::json;
use std::sync::Arc;

use async_trait::async_trait;

use super::{MessageSendError, MessageSender};
use crate::{authenticator::Authenticator, message::Message};

type Auth = dyn Authenticator + Sync + Send;

pub struct AppSyncMessageSender {
	uri: String,
	auth: Arc<Auth>,
	client: Client,
}

impl AppSyncMessageSender {
	pub fn new(uri: String, auth: Arc<Auth>) -> Self {
		let client = Client::new();
		Self { uri, auth, client }
	}

	fn message_to_body(message: Message) -> serde_json::Value {
		let event = serde_json::to_string(&message).unwrap();

		json!({
			"channel": message.channel,
			"events": [event],
		})
	}

	fn build_message(&self, message: Message) -> RequestBuilder {
		let body = Self::message_to_body(message);

		let mut request_builder = self
			.client
			.post(&self.uri)
			.header("content_type", "application/json");

		for (key, value) in self.auth.publish_auth_headers() {
			request_builder = request_builder.header(key, value);
		}

		request_builder.json(&body)
	}
}

#[async_trait]
impl MessageSender for AppSyncMessageSender {
	async fn send_text_message(&self, message: Message) -> Result<(), MessageSendError> {
		let request = self.build_message(message);

		let response = request.send().await?;

		if !response.status().is_success() {
			return Err(MessageSendError::SendFailed(response.text().await?));
		}

		Ok(())
	}
}

impl From<reqwest::Error> for MessageSendError {
	fn from(error: reqwest::Error) -> Self {
		Self::HTTPError(error.to_string())
	}
}
