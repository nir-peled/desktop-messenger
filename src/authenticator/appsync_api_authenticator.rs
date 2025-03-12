use super::Authenticator;
use std::collections::HashMap;

pub struct AppSyncAPIAuthenticator {
	hostname: Box<str>,
	api_key: Box<str>,
}

impl AppSyncAPIAuthenticator {
	pub fn new(hostname: &Box<str>, api_key: &Box<str>) -> Self {
		Self {
			hostname: hostname.clone(),
			api_key: api_key.clone(),
		}
	}
}

impl Authenticator for AppSyncAPIAuthenticator {
	fn authenticate(&self) -> bool {
		return true;
	}

	fn publish_auth_headers(&self) -> HashMap<String, String> {
		let mut result = HashMap::new();

		result.insert(
			String::from("x-api-key"),
			self.api_key.clone().into_string(),
		);

		result
	}

	fn subscribe_auth_headers(&self) -> HashMap<String, String> {
		let mut result = HashMap::new();

		result.insert(
			String::from("x-api-key"),
			self.api_key.clone().into_string(),
		);
		result.insert(String::from("host"), self.hostname.clone().into_string());

		result
	}
}
