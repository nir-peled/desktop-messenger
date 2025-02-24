use crate::message_receiver::MessageReceiver;
use crate::message_sender::MessageSender;
use crate::task_queue::TaskQueue;
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
		connection.lock().await.add_channel("/default"); // debug

		self.ui_connector.start(self.task_queue.clone());

		self.handle_tasks().await;

		println!("Shutting down server...");
	}

	async fn handle_tasks(&mut self) {
		let mut task = self.task_queue.pop().await;
		while task.kind != "exit" {
			match task.kind.as_str() {
				"send" => {
					self.message_sender.send_text_message(task.message.unwrap());
				}
				"receive" => {
					self.ui_connector.message_received(task.message.unwrap());
				}
				_ => {
					panic!("unknown task kind: {}", task.kind)
				}
			}

			task = self.task_queue.pop().await;
		}
	}
}
