#[derive(Debug, Clone)]
pub struct Message {
	pub sender: String,
	pub channel: String,
	pub contents: String,
}
