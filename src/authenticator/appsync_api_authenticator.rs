use super::Authenticator;
use std::collections::HashMap;

pub struct AppSyncAPIAuthenticator {
	hostname: String,
	api_key: String,
}

impl Authenticator for AppSyncAPIAuthenticator {
	fn authenticate(&mut self) -> bool {
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
