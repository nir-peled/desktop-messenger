#[derive(Debug, Clone)]
pub struct Message {
	pub sender: Box<str>,
	pub channel: Box<str>,
	pub contents: Box<str>,
}
