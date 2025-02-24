use std::{
	collections::VecDeque,
	sync::{
		atomic::{AtomicU32, Ordering},
		Arc,
	},
};
use tokio::sync::{watch::Sender, Mutex};

use crate::message::Message;

pub struct Task {
	pub kind: String,
	pub message: Option<Message>,
}

#[derive(Clone)]
pub struct TaskQueue {
	queue: Arc<Mutex<VecDeque<Task>>>,
	notifier: Arc<Sender<()>>,
	size: Arc<AtomicU32>,
}

impl Task {
	pub fn send_message(message: Message) -> Self {
		Self {
			kind: "send".to_string(),
			message: Some(message),
		}
	}

	pub fn receive_message(message: Message) -> Self {
		Self {
			kind: "receive".to_string(),
			message: Some(message),
		}
	}

	pub fn exit() -> Self {
		Self {
			kind: "exit".to_string(),
			message: None,
		}
	}
}

impl TaskQueue {
	pub fn new() -> Self {
		Self {
			queue: Arc::new(Mutex::new(VecDeque::new())),
			notifier: Arc::new(Sender::new(())),
			size: Arc::new(AtomicU32::new(0)),
		}
	}

	pub async fn push(&mut self, task: Task) {
		let mut queue = self.queue.lock().await;
		queue.push_back(task);
		self.size.fetch_add(1, Ordering::AcqRel);
		self.notifier.send(()).unwrap_or(());
	}

	pub async fn pop(&mut self) -> Task {
		while self.size.load(Ordering::Acquire) == 0 {
			let mut rec = self.notifier.subscribe();
			rec.changed().await.unwrap_or(());
		}
		let mut queue = self.queue.lock().await;
		self.size.fetch_sub(1, Ordering::AcqRel);
		queue.pop_front().unwrap()
	}
}
