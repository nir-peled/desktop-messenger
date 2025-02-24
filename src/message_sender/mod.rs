use crate::message::Message;

pub trait MessageSender {
	fn send_text_message(&mut self, message: Message);
}

pub mod dummy;
