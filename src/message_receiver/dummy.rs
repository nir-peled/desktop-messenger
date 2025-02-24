use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use super::{MessageReceiver, OpenConnection};
use crate::message::Message;
use crate::task_queue::{Task, TaskQueue};

pub struct DummyOpenConnection {
	task_queue: TaskQueue,
	loop_handle: JoinHandle<()>,
	channels: Vec<String>,
}

pub struct DummyMessageReceiver {}

impl DummyOpenConnection {
	pub async fn new(task_queue: TaskQueue) -> Arc<Mutex<Self>> {
		println!("creating DummyOpenConnection");
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
				sender: "dummy".to_string(),
				channel: "dummy".to_string(),
				contents: "Hello, Dummy!".to_string(),
			};

			loop {
				interval.tick().await;
				match weak_copy.upgrade() {
					Some(connection) => connection.lock().await.send_message(message.clone()).await,
					None => return,
				};
			}
		});

		result
	}
}

impl Drop for DummyOpenConnection {
	fn drop(&mut self) {
		println!("dropping DummyOpenConnection");
		self.loop_handle.abort();
	}
}

#[async_trait]
impl OpenConnection for DummyOpenConnection {
	fn add_channel(&mut self, channel: &str) {
		self.channels.push(channel.to_string());
	}

	fn channels(&self) -> Vec<String> {
		self.channels.clone()
	}

	async fn send_message(&mut self, message: Message) {
		self.task_queue.push(Task::receive_message(message)).await
	}
}

impl DummyMessageReceiver {
	pub fn new() -> Self {
		Self {}
	}
}

impl MessageReceiver for DummyMessageReceiver {
	async fn listen(&mut self, task_queue: TaskQueue) -> Arc<Mutex<dyn OpenConnection>> {
		DummyOpenConnection::new(task_queue).await
	}
}
