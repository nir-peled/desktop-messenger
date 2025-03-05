mod authenticator;
mod message;
mod message_receiver;
mod message_sender;
mod messenger;
mod settings;
mod task_queue;
mod ui_connector;

use std::sync::Arc;

use authenticator::Authenticator;
use message_receiver::appsync_message_receiver::AppSyncMessageReceiver;
use message_sender::appsync_message_sender::AppSyncMessageSender;
use settings::Settings;

use crate::authenticator::appsync_api_authenticator::AppSyncAPIAuthenticator;
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
		settings.APPSYNC_HTTP_DOMAIN.into_boxed_str(),
		settings.APPSYNC_API_KEY.into_boxed_str(),
	));

	let mut messenger = Messenger::new(
		Arc::clone(&auth),
		// DummyMessageReceiver::new(),
		// DummyMessageSender::new(),
		AppSyncMessageReceiver::new(
			settings.APPSYNC_WEBSOCKET_URL,
			Arc::clone(&auth) as Arc<dyn Authenticator + Send + Sync>,
		),
		AppSyncMessageSender::new(
			settings.APPSYNC_PUBLISH_URL,
			Arc::clone(&auth) as Arc<dyn Authenticator + Send + Sync>,
		),
		SimplifiedUI::new(),
	);

	messenger.start().await;
}
