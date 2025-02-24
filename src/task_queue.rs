use std::{
	collections::VecDeque,
	sync::{
		atomic::{AtomicU32, Ordering},
		Arc,
	},
};
use tokio::sync::{watch::Sender, Mutex};

use crate::message::Message;

pub enum TaskData {
	SendMessage(Message),
	ReceiveMessage(Message),
	NewChannel(String),
	RemoveChannel(String),
	Exit,
}

#[derive(Clone)]
pub struct TaskQueue {
	queue: Arc<Mutex<VecDeque<TaskData>>>,
	notifier: Arc<Sender<()>>,
	size: Arc<AtomicU32>,
}

impl TaskQueue {
	pub fn new() -> Self {
		Self {
			queue: Arc::new(Mutex::new(VecDeque::new())),
			notifier: Arc::new(Sender::new(())),
			size: Arc::new(AtomicU32::new(0)),
		}
	}

	pub async fn push(&mut self, task: TaskData) {
		let mut queue = self.queue.lock().await;
		queue.push_back(task);
		self.size.fetch_add(1, Ordering::AcqRel);
		self.notifier.send(()).unwrap_or(());
	}

	pub async fn pop(&mut self) -> TaskData {
		while self.size.load(Ordering::Acquire) == 0 {
			let mut rec = self.notifier.subscribe();
			rec.changed().await.unwrap_or(());
		}
		let mut queue = self.queue.lock().await;
		self.size.fetch_sub(1, Ordering::AcqRel);
		queue.pop_front().unwrap()
	}
}
