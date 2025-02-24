use std::collections::HashMap;

pub trait Authenticator {
	fn authenticate(&mut self) -> bool;
	fn publish_auth_headers(&self) -> HashMap<String, String>;
	fn subscribe_auth_headers(&self) -> HashMap<String, String>;
}

pub mod appsync_api_authenticator;
