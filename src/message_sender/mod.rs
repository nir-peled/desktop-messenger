use std::fmt;

use async_trait::async_trait;

use crate::message::Message;

#[derive(Debug)]
pub enum MessageSendError {
	HTTPError(String),
	SendFailed(String),
}

#[async_trait]
pub trait MessageSender {
	async fn send_text_message(&mut self, message: Message) -> Result<(), MessageSendError>;
}

impl std::error::Error for MessageSendError {}

impl fmt::Display for MessageSendError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::HTTPError(e) => write!(f, "HTTP Error: {}", e),
			Self::SendFailed(e) => write!(f, "Message Send Failed: {}", e),
		}
	}
}

pub mod appsync_message_sender;
pub mod dummy;
