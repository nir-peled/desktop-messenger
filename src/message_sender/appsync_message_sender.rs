use std::sync::Arc;

use async_trait::async_trait;

use super::MessageSender;
use crate::{authenticator::Authenticator, message::Message};

type Auth = dyn Authenticator + Send + Sync;

struct AppSyncMessageSender {
	auth: Arc<Auth>,
	client: hyper::Client<hyper::client::connect::HttpConnector>,
}

impl AppSyncMessageSender {
	pub fn new(auth: Arc<Auth>) -> Self {
		let client = hyper::Client::new();
		Self { auth, client }
	}
}

#[async_trait]
impl MessageSender for AppSyncMessageSender {
	async fn send_text_message(&mut self, message: Message) {
		todo!()
	}
}
