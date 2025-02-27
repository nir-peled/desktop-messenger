mod authenticator;
mod message;
mod message_receiver;
mod message_sender;
mod messenger;
mod settings;
mod task_queue;
mod ui_connector;

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
		Ok(settings) => {
			let auth = AppSyncAPIAuthenticator::new(
				settings.APPSYNC_HTTP_DOMAIN,
				settings.APPSYNC_API_KEY,
			);
			let mut messenger = Messenger::new(
				DummyMessageReceiver::new(),
				DummyMessageSender::new(),
				SimplifiedUI::new(),
			);
			messenger.start().await;
		}
		Err(err) => println!("error reading settings: {}", err),
	}
}
