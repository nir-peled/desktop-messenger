use std::{fmt, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{message::Message, task_queue::TaskQueue};

#[derive(Debug)]
pub enum MessageReceiverError {
	ConnectionError(String),
}

#[async_trait]
pub trait OpenConnection {
	async fn add_channel(&mut self, channel: &str);
	async fn remove_channel(&mut self, channel: &str);
	fn channels(&self) -> Vec<Box<str>>;
	async fn receive_message(&mut self, message: Message);
}

pub type OpenConnectionHolder = Arc<Mutex<dyn OpenConnection>>;

pub trait MessageReceiver {
	#[must_use]
	async fn listen(
		&self,
		task_queue: TaskQueue,
	) -> Result<OpenConnectionHolder, MessageReceiverError>;
}

impl std::error::Error for MessageReceiverError {}

impl fmt::Display for MessageReceiverError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::ConnectionError(e) => write!(f, "Connection Error: {}", e),
		}
	}
}

pub mod appsync_message_receiver;
pub mod dummy;
