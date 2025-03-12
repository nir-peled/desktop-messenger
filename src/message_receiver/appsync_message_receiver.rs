use std::{
	collections::HashMap,
	str::FromStr,
	sync::{Arc, Weak},
};

use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD as base64_engine, Engine as _};
use futures_util::{sink::SinkExt, stream::SplitSink, StreamExt};
use serde_json::{json, Value};
use tokio::{net::TcpStream, sync::Mutex, task::JoinHandle};
use tokio_tungstenite::{
	connect_async_tls_with_config,
	tungstenite::{
		http::{Request as WebSocketRequest, Uri},
		protocol::Message as WebSocketMessage,
		Utf8Bytes,
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
type WebSocketHolder = Arc<Mutex<SplitSink<WebSocket, WebSocketMessage>>>;

pub struct AppSyncOpenConnection {
	websocket_send: WebSocketHolder,
	authenticator: Arc<Auth>,
	channels_ids: HashMap<Box<str>, Box<str>>,
	task_queue: TaskQueue,
	listener_handle: JoinHandle<()>,
}

pub struct AppSyncMessageReceiver {
	authenticator: Arc<Auth>,
	uri: Box<str>,
}

impl AppSyncMessageReceiver {
	pub fn new(uri: &Box<str>, authenticator: Arc<Auth>) -> Self {
		Self {
			authenticator,
			uri: uri.clone(),
		}
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
		let uri = Uri::from_str(&self.uri)?;
		let host = uri.host().ok_or(MessageReceiverError::ConnectionError(
			"URI missing host".to_owned(),
		))?;

		let request = WebSocketRequest::builder()
			.uri(&uri)
			.header("Host", host)
			.header("Connection", "Upgrade")
			.header("Upgrade", "websocket")
			.header("Sec-WebSocket-Version", "13")
			.header(
				"Sec-WebSocket-Key",
				tokio_tungstenite::tungstenite::handshake::client::generate_key(),
			)
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
		let (send, mut receive) = websocket.split();

		let result = Arc::new(Mutex::new(Self {
			websocket_send: Arc::new(Mutex::new(send)),
			authenticator,
			channels_ids: HashMap::new(),
			task_queue,
			listener_handle: tokio::task::spawn(async {}),
		}));
		let weak_copy = Arc::downgrade(&result);

		result.lock().await.listener_handle = tokio::task::spawn(async move {
			while let Some(received_message) = receive.next().await {
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
					Err(_) => break,
				}
			}
		});

		result
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

		// ignore any non-data messages for the meanwhile
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
		let websocket = Arc::clone(&self.websocket_send);

		tokio::task::spawn(async move {
			for id in ids {
				Self::send_unsubscribe(&websocket, &id).await;
			}

			let _ = websocket.lock().await.close().await;
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

		let result = self.websocket_send.lock().await.send(message).await;
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
				Self::send_unsubscribe(&self.websocket_send, channel_id).await;
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

impl From<tokio_tungstenite::tungstenite::http::uri::InvalidUri> for MessageReceiverError {
	fn from(error: tokio_tungstenite::tungstenite::http::uri::InvalidUri) -> Self {
		Self::ConnectionError(error.to_string())
	}
}
