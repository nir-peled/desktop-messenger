use super::Authenticator;
use std::collections::HashMap;

pub struct AppSyncAPIAuthenticator {
	hostname: String,
	api_key: String,
}

impl AppSyncAPIAuthenticator {
	pub fn new(hostname: String, api_key: String) -> Self {
		Self { hostname, api_key }
	}
}

impl Authenticator for AppSyncAPIAuthenticator {
	fn authenticate(&self) -> bool {
		return true;
	}

	fn publish_auth_headers(&self) -> HashMap<String, String> {
		let mut result = HashMap::new();

		result.insert(String::from("x-api-key"), self.api_key.clone());

		result
	}

	fn subscribe_auth_headers(&self) -> HashMap<String, String> {
		let mut result = HashMap::new();

		result.insert(String::from("x-api-key"), self.api_key.clone());
		result.insert(String::from("host"), self.hostname.clone());

		result
	}
}
