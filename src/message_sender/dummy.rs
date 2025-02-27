use async_trait::async_trait;

use super::MessageSender;
use crate::message::Message;

pub struct DummyMessageSender {}

impl DummyMessageSender {
	pub fn new() -> Self {
		DummyMessageSender {}
	}
}

#[async_trait]
impl MessageSender for DummyMessageSender {
	async fn send_text_message(&mut self, message: Message) {
		println!("Sending message: {:?}", message);
	}
}
