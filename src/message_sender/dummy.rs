use super::MessageSender;
use crate::message::Message;

pub struct DummyMessageSender {}

impl DummyMessageSender {
	pub fn new() -> Self {
		DummyMessageSender {}
	}
}

impl MessageSender for DummyMessageSender {
	fn send_text_message(&mut self, message: Message) {
		println!("Sending message: {:?}", message);
	}
}
