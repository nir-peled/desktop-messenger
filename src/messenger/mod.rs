use crate::message_receiver::{MessageReceiver, OpenConnectionHolder};
use crate::message_sender::MessageSender;
use crate::task_queue::{TaskData, TaskQueue};
use crate::ui_connector::UIConnector;

pub struct Messenger<TReceiver: MessageReceiver, TSender: MessageSender, TUI: UIConnector> {
	message_receiver: TReceiver,
	message_sender: TSender,
	ui_connector: TUI,
	task_queue: TaskQueue,
}

impl<TReceiver: MessageReceiver, TSender: MessageSender, TUI: UIConnector>
	Messenger<TReceiver, TSender, TUI>
{
	pub fn new(message_receiver: TReceiver, message_sender: TSender, ui_connector: TUI) -> Self {
		return Messenger {
			message_receiver,
			message_sender,
			ui_connector,
			task_queue: TaskQueue::new(),
		};
	}

	pub async fn start(&mut self) {
		println!("Starting Server");

		let connection = self.message_receiver.listen(self.task_queue.clone()).await;

		self.ui_connector.start(self.task_queue.clone());

		self.handle_tasks(&connection).await;

		println!("Shutting down server...");
	}

	async fn handle_tasks(&mut self, connection: &OpenConnectionHolder) {
		let mut task = self.task_queue.pop().await;
		loop {
			match task {
				TaskData::SendMessage(message) => {
					self.message_sender.send_text_message(message).await
				}
				TaskData::ReceiveMessage(message) => self.ui_connector.message_received(message),
				TaskData::NewChannel(channel) => {
					connection.lock().await.add_channel(channel.as_str())
				}
				TaskData::RemoveChannel(channel) => {
					connection.lock().await.remove_channel(channel.as_str())
				}
				TaskData::Exit => break,
			};

			task = self.task_queue.pop().await;
		}
	}
}
