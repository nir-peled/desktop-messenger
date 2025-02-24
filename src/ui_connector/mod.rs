use crate::{message::Message, task_queue::TaskQueue};

pub trait UIConnector {
	fn message_received(&mut self, message: Message);
	fn start(&mut self, task_queue: TaskQueue);
}

pub mod simplified;
