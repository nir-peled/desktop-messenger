mod authenticator;
mod message;
mod message_receiver;
mod message_sender;
mod messenger;
mod settings;
mod task_queue;
mod ui_connector;

use std::sync::Arc;

use settings::Settings;

use crate::authenticator::appsync_api_authenticator::AppSyncAPIAuthenticator;
use crate::message_receiver::dummy::DummyMessageReceiver;
use crate::message_sender::dummy::DummyMessageSender;
use crate::messenger::Messenger;
use crate::ui_connector::simplified::SimplifiedUI;

#[tokio::main]
async fn main() {
	let settings = Settings::from_env_file(".env.local");
	match settings {
		Ok(settings) => run_client(settings).await,
		Err(err) => println!("error reading settings: {}", err),
	}
}

async fn run_client(settings: Settings) {
	let auth = Arc::new(AppSyncAPIAuthenticator::new(
		settings.APPSYNC_HTTP_DOMAIN,
		settings.APPSYNC_API_KEY,
	));
	let mut messenger = Messenger::new(
		Arc::clone(&auth),
		DummyMessageReceiver::new(),
		DummyMessageSender::new(),
		SimplifiedUI::new(),
	);
	messenger.start().await;
}
