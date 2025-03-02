use hyper::body::HttpBody;
use serde_json::json;
use std::sync::Arc;

use async_trait::async_trait;

use super::{MessageSendError, MessageSender};
use crate::{authenticator::Authenticator, message::Message};

type Auth = dyn Authenticator + Send + Sync;

struct AppSyncMessageSender {
	uri: String,
	auth: Arc<Auth>,
	client: hyper::Client<hyper::client::connect::HttpConnector>,
}

impl AppSyncMessageSender {
	pub fn new(uri: String, auth: Arc<Auth>) -> Self {
		let client = hyper::Client::new();
		Self { uri, auth, client }
	}

	fn message_to_body(message: Message) -> hyper::Body {
		let Message {
			sender,
			channel,
			contents,
		} = message;
		let event = json!({
			"sender": sender,
			"contents":  contents,
		});

		let body_raw = json!({
			"channel": channel,
			"events": [event.to_string()],
		})
		.to_string();

		hyper::Body::from(body_raw)
	}

	async fn response_body(
		response: hyper::Response<hyper::Body>,
	) -> Result<String, MessageSendError> {
		let body_bytes = hyper::body::to_bytes(response.into_body()).await?;
		Ok(String::from_utf8(body_bytes.to_vec())?)
	}
}

#[async_trait]
impl MessageSender for AppSyncMessageSender {
	async fn send_text_message(&mut self, message: Message) -> Result<(), MessageSendError> {
		let body = Self::message_to_body(message);
		let mut request_builder = hyper::Request::builder()
			.method(hyper::Method::POST)
			.uri(self.uri.clone())
			.header("content_type", "application/json");
		for (key, value) in self.auth.publish_auth_headers() {
			request_builder = request_builder.header(key, value);
		}
		let request = request_builder.body(body)?;

		let response = self.client.request(request).await?;
		if !response.status().is_success() {
			return Err(MessageSendError::SendFailed(
				Self::response_body(response).await?,
			));
		}

		Ok(())
	}
}

impl From<hyper::http::Error> for MessageSendError {
	fn from(error: hyper::http::Error) -> Self {
		Self::HTTPError(error.to_string())
	}
}

impl From<hyper::Error> for MessageSendError {
	fn from(error: hyper::Error) -> Self {
		Self::HTTPError(error.to_string())
	}
}

impl From<std::string::FromUtf8Error> for MessageSendError {
	fn from(error: std::string::FromUtf8Error) -> Self {
		Self::HTTPError(error.to_string())
	}
}
