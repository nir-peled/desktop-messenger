use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::{MessageReceiver, MessageReceiverError, OpenConnection, OpenConnectionHolder};
use crate::message::Message;
use crate::task_queue::{TaskData, TaskQueue};

pub struct DummyOpenConnection {
	task_queue: TaskQueue,
	loop_handle: JoinHandle<()>,
	channels: Vec<Box<str>>,
}

pub struct DummyMessageReceiver {}

impl DummyOpenConnection {
	pub async fn new(task_queue: TaskQueue) -> OpenConnectionHolder {
		let result = Arc::new(Mutex::new(Self {
			task_queue: task_queue,
			loop_handle: tokio::task::spawn(async {}),
			channels: Vec::new(),
		}));
		let weak_copy = Arc::downgrade(&result);

		result.lock().await.loop_handle = tokio::task::spawn(async move {
			let duration = Duration::from_secs(3);
			let mut interval = tokio::time::interval(duration);
			let message = Message {
				sender: "dummy".into(),
				channel: "dummy".into(),
				contents: "Hello, Dummy!".into(),
			};

			loop {
				interval.tick().await;
				match weak_copy.upgrade() {
					Some(connection) => {
						connection
							.lock()
							.await
							.receive_message(message.clone())
							.await
					}
					None => return,
				};
			}
		});

		result
	}
}

impl Drop for DummyOpenConnection {
	fn drop(&mut self) {
		self.loop_handle.abort();
	}
}

#[async_trait]
impl OpenConnection for DummyOpenConnection {
	async fn add_channel(&mut self, channel: &str) {
		self.channels.push(channel.into());
	}

	async fn remove_channel(&mut self, channel: &str) {
		self.channels.retain(|c| **c != *channel);
	}

	fn channels(&self) -> Vec<Box<str>> {
		self.channels.clone()
	}

	async fn receive_message(&mut self, message: Message) {
		self.task_queue
			.push(TaskData::ReceiveMessage(message))
			.await
	}
}

impl DummyMessageReceiver {
	pub fn new() -> Self {
		Self {}
	}
}

impl MessageReceiver for DummyMessageReceiver {
	async fn listen(
		&self,
		task_queue: TaskQueue,
	) -> Result<OpenConnectionHolder, MessageReceiverError> {
		Ok(DummyOpenConnection::new(task_queue).await)
	}
}
