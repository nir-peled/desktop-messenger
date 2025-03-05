use std::{
	collections::HashMap,
	sync::{Arc, Weak},
};

use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as base64_engine, Engine as _};
use futures_util::{sink::SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::{net::TcpStream, sync::Mutex, task::JoinHandle};
use tokio_tungstenite::{
	connect_async_tls_with_config,
	tungstenite::{
		http::Request as WebSocketRequest, protocol::Message as WebSocketMessage, Utf8Bytes,
	},
	MaybeTlsStream, WebSocketStream,
};
use uuid::Uuid;

use crate::{
	authenticator::Authenticator,
	message::Message,
	task_queue::{TaskData, TaskQueue},
};

use super::{MessageReceiver, MessageReceiverError, OpenConnection, OpenConnectionHolder};

type Auth = dyn Authenticator + Send + Sync;
type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WebSocketHolder = Arc<Mutex<WebSocket>>;

pub struct AppSyncOpenConnection {
	websocket: WebSocketHolder,
	authenticator: Arc<Auth>,
	channels_ids: HashMap<Box<str>, Box<str>>,
	task_queue: TaskQueue,
	listener_handle: JoinHandle<()>,
}

pub struct AppSyncMessageReceiver {
	authenticator: Arc<Auth>,
	uri: String,
}

impl AppSyncMessageReceiver {
	pub fn new(authenticator: Arc<Auth>, uri: String) -> Self {
		Self { authenticator, uri }
	}

	fn auth_header(&self) -> Box<str> {
		let auth_components = self.authenticator.subscribe_auth_headers();
		let auth_str = json!(auth_components).to_string();
		let b64_str = base64_engine.encode(auth_str);
		format!("header-{}", b64_str).into_boxed_str()
	}
}

impl MessageReceiver for AppSyncMessageReceiver {
	async fn listen(
		&self,
		task_queue: TaskQueue,
	) -> Result<OpenConnectionHolder, MessageReceiverError> {
		let auth_header = self.auth_header();
		let subprotocols = format!("aws-appsync-event-ws,{}", auth_header);

		let request = WebSocketRequest::builder()
			.uri(self.uri.clone())
			.header("Sec-WebSocket-Protocol", subprotocols)
			.body(())?;

		let (websocket, _) = connect_async_tls_with_config(request, None, false, None).await?;

		Ok(
			AppSyncOpenConnection::new(task_queue, websocket, Arc::clone(&self.authenticator))
				.await,
		)
	}
}

impl AppSyncOpenConnection {
	pub async fn new(
		task_queue: TaskQueue,
		websocket: WebSocket,
		authenticator: Arc<Auth>,
	) -> OpenConnectionHolder {
		let result = Arc::new(Mutex::new(Self {
			websocket: Arc::new(Mutex::new(websocket)),
			authenticator,
			channels_ids: HashMap::new(),
			task_queue,
			listener_handle: tokio::task::spawn(async {}),
		}));
		let weak_copy = Arc::downgrade(&result);
		let websocket = Arc::clone(&result.lock().await.websocket);

		result.lock().await.listener_handle = tokio::task::spawn(async move {
			while let Some(received_message) = websocket.blocking_lock().next().await {
				match received_message {
					Ok(message_base) => {
						if let WebSocketMessage::Text(message) = message_base {
							let result =
								Self::handle_incoming_message(weak_copy.clone(), message).await;
							if result == None {
								break;
							}
						}
					}
					Err(_e) => break,
				}
			}
		});

		todo!()
	}

	async fn send_unsubscribe(websocket: &WebSocketHolder, channel_id: &str) {
		let message = WebSocketMessage::text(format!(
			r#"{{"type":"unsubscribe", "client_id":{}}}"#,
			channel_id
		));
		let _ = websocket.lock().await.send(message).await;
	}

	async fn handle_incoming_message(
		connection: Weak<Mutex<AppSyncOpenConnection>>,
		message_raw: Utf8Bytes,
	) -> Option<()> {
		let message_value: Value = serde_json::from_str(message_raw.as_str()).ok()?;
		let message_obj = message_value.as_object()?;

		if message_obj.get("type")? != "data" {
			return Some(());
		}

		let message: Message = serde_json::from_str(message_obj.get("event")?.as_str()?).ok()?;

		match connection.upgrade() {
			Some(connection) => {
				connection
					.lock()
					.await
					.receive_message(message.clone())
					.await
			}
			None => return None,
		};

		Some(())
	}
}

impl Drop for AppSyncOpenConnection {
	fn drop(&mut self) {
		let ids: Vec<Box<str>> = self.channels_ids.values().map(|id| id.clone()).collect();
		let websocket = Arc::clone(&self.websocket);

		tokio::task::spawn(async move {
			for id in ids {
				Self::send_unsubscribe(&websocket, &id).await;
			}
		});
	}
}

#[async_trait]
impl OpenConnection for AppSyncOpenConnection {
	async fn add_channel(&mut self, channel: &str) {
		if self.channels_ids.contains_key(channel) {
			return;
		}

		let uuid = Uuid::new_v4().simple();
		let mut buf = [b'!'; 36];
		let uuid_str = uuid.encode_lower(&mut buf);

		let message_raw = json!({
			"type": "subscribe",
			"id": uuid_str,
			"channel": channel,
			"authorization": self.authenticator.subscribe_auth_headers(),
		})
		.to_string();
		let message = WebSocketMessage::text(message_raw);

		let result = self.websocket.lock().await.send(message).await;
		if let Err(e) = result {
			panic!("Error sending subscribe messsage: {}", e);
		}

		self.channels_ids.insert(channel.into(), uuid_str.into());
	}

	async fn remove_channel(&mut self, channel: &str) {
		let channel_id = self.channels_ids.get(channel);
		match channel_id {
			None => (),
			Some(channel_id) => {
				Self::send_unsubscribe(&self.websocket, channel_id).await;
				self.channels_ids.remove(channel);
			}
		}
	}

	fn channels(&self) -> Vec<Box<str>> {
		self.channels_ids.keys().map(|k| k.clone()).collect()
	}

	async fn receive_message(&mut self, message: Message) {
		self.task_queue
			.push(TaskData::ReceiveMessage(message))
			.await
	}
}

impl From<tokio_tungstenite::tungstenite::http::Error> for MessageReceiverError {
	fn from(error: tokio_tungstenite::tungstenite::http::Error) -> Self {
		Self::ConnectionError(error.to_string())
	}
}

impl From<tokio_tungstenite::tungstenite::Error> for MessageReceiverError {
	fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
		Self::ConnectionError(error.to_string())
	}
}
