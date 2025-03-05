use std::sync::Arc;

use crate::authenticator::Authenticator;
use crate::message_receiver::{MessageReceiver, OpenConnectionHolder};
use crate::message_sender::MessageSender;
use crate::task_queue::{TaskData, TaskQueue};
use crate::ui_connector::UIConnector;

pub struct Messenger<
	TAuth: Authenticator,
	TReceiver: MessageReceiver,
	TSender: MessageSender,
	TUI: UIConnector,
> {
	authenticator: Arc<TAuth>,
	message_receiver: TReceiver,
	message_sender: TSender,
	ui_connector: TUI,
	task_queue: TaskQueue,
}

impl<
		TAuth: Authenticator,
		TReceiver: MessageReceiver,
		TSender: MessageSender,
		TUI: UIConnector,
	> Messenger<TAuth, TReceiver, TSender, TUI>
{
	pub fn new(
		authenticator: Arc<TAuth>,
		message_receiver: TReceiver,
		message_sender: TSender,
		ui_connector: TUI,
	) -> Self {
		return Messenger {
			authenticator,
			message_receiver,
			message_sender,
			ui_connector,
			task_queue: TaskQueue::new(),
		};
	}

	pub async fn start(&mut self) {
		println!("Starting Server");
		if !self.authenticator.authenticate() {
			println!("Authentication Failed!");
			return;
		}

		let connect_result = self.message_receiver.listen(self.task_queue.clone()).await;
		match connect_result {
			Ok(connection) => {
				self.ui_connector.start(self.task_queue.clone());

				self.handle_tasks(&connection).await;

				println!("Shutting down server...");
			}
			Err(e) => println!("Could not connect: {}", e),
		}
	}

	async fn handle_tasks(&mut self, connection: &OpenConnectionHolder) {
		let mut task = self.task_queue.pop().await;
		loop {
			match task {
				TaskData::SendMessage(message) => {
					let res = self.message_sender.send_text_message(message).await;
					if let Err(e) = res {
						println!("Error sending message: {}", e);
					}
				}
				TaskData::ReceiveMessage(message) => self.ui_connector.message_received(message),
				TaskData::NewChannel(channel) => {
					connection.lock().await.add_channel(&channel).await;
				}
				TaskData::RemoveChannel(channel) => {
					connection.lock().await.remove_channel(&channel).await;
				}
				TaskData::Exit => break,
			};

			task = self.task_queue.pop().await;
		}
	}
}
