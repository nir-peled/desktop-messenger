use async_trait::async_trait;

use crate::message::Message;

#[async_trait]
pub trait MessageSender {
	async fn send_text_message(&mut self, message: Message);
}

pub mod appsync_message_sender;
pub mod dummy;
