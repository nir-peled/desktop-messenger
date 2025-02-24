use crate::{
	message::Message,
	task_queue::{Task, TaskQueue},
};

use super::UIConnector;

pub struct SimplifiedUI {}

impl SimplifiedUI {
	pub fn new() -> Self {
		Self {}
	}
}

impl UIConnector for SimplifiedUI {
	fn message_received(&mut self, message: Message) {
		println!("message received: {:?}", message)
	}

	fn start(&mut self, mut task_queue: TaskQueue) {
		tokio::task::spawn(async move {
			let mut buffer = String::new();
			std::io::stdin().read_line(&mut buffer).ok()?;
			let mut line = buffer.trim();

			while line != "exit" {
				let mut splitter = line.splitn(3, " ");
				let sender = splitter.next().unwrap_or("").to_string();
				let channel = splitter.next().unwrap_or("").to_string();
				let contents = splitter.next().unwrap_or("").to_string();

				let message = Message {
					sender,
					channel,
					contents,
				};
				task_queue.push(Task::send_message(message)).await;

				buffer.clear();
				std::io::stdin().read_line(&mut buffer).ok()?;
				line = buffer.trim();
			}

			task_queue.push(Task::exit()).await;
			Some(())
		});
	}
}
