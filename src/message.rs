use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
	pub sender: Box<str>,
	pub channel: Box<str>,
	pub contents: Box<str>,
}
