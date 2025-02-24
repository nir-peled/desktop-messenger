use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::{message::Message, task_queue::TaskQueue};

#[async_trait]
pub trait OpenConnection {
	fn add_channel(&mut self, channel: &str);
	fn channels(&self) -> Vec<String>;
	async fn send_message(&mut self, message: Message);
}

pub trait MessageReceiver {
	#[must_use]
	async fn listen(&mut self, task_queue: TaskQueue) -> Arc<Mutex<dyn OpenConnection>>;
}

pub mod dummy;
