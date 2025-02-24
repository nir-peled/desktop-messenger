// mod authenticator;
mod message;
mod message_receiver;
mod message_sender;
mod messenger;
mod task_queue;
mod ui_connector;

// use crate::authenticator::appsync_api_authenticator::AppSyncAPIAuthenticator;
use crate::message_receiver::dummy::DummyMessageReceiver;
use crate::message_sender::dummy::DummyMessageSender;
use crate::messenger::Messenger;
use crate::ui_connector::simplified::SimplifiedUI;

#[tokio::main]
async fn main() {
	let mut messenger = Messenger::new(
		DummyMessageReceiver::new(),
		DummyMessageSender::new(),
		SimplifiedUI::new(),
	);
	messenger.start().await;
}
