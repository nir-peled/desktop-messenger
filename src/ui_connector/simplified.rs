use crate::{
	message::Message,
	task_queue::{TaskData, TaskQueue},
};

use super::UIConnector;

enum UIStatus {
	Continue,
	Stop,
}

pub struct SimplifiedUI {}

impl SimplifiedUI {
	pub fn new() -> Self {
		Self {}
	}

	async fn handle_command(task_queue: &mut TaskQueue, line: String) -> UIStatus {
		let mut splitter = line.splitn(4, " ");
		let command = splitter.next().unwrap_or("");
		let arg1 = splitter.next().unwrap_or("").into();
		let arg2 = splitter.next().unwrap_or("").into();
		let arg3 = splitter.next().unwrap_or("").into();

		match command {
			"" => (),
			"add_channel" => task_queue.push(TaskData::NewChannel(arg1)).await,
			"remove_channel" => task_queue.push(TaskData::RemoveChannel(arg1)).await,
			"send" => {
				let message = Message {
					sender: arg1,
					channel: arg2,
					contents: arg3,
				};
				task_queue.push(TaskData::SendMessage(message)).await;
			}
			"exit" => return UIStatus::Stop,
			_ => println!("Unknown command"),
		}

		UIStatus::Continue
	}

	fn read_line() -> Option<String> {
		let mut buffer = String::new();
		std::io::stdin().read_line(&mut buffer).ok()?;
		Some(buffer.trim().to_string())
	}
}

impl UIConnector for SimplifiedUI {
	fn message_received(&mut self, message: Message) {
		println!("message received: {:?}", message)
	}

	fn start(&mut self, mut task_queue: TaskQueue) {
		tokio::task::spawn(async move {
			loop {
				let read_line = SimplifiedUI::read_line();
				match read_line {
					Some(line) => {
						let new_status = SimplifiedUI::handle_command(&mut task_queue, line).await;
						if let UIStatus::Stop = new_status {
							break;
						}
					}
					None => break,
				}
			}

			task_queue.push(TaskData::Exit).await;
		});
	}
}
