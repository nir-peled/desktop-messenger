/*
 * Settings is meant to represent program-wide settings read at runtime from
 * an env-like file, into any types (or, at least, those convertible from string),
 * in addition to raising errors on duplicate or missing fields.
 * This is achieved using a macro to build the struct & the reader function
*/
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
pub enum SettingsReadError {
	MissingField(String),
	DuplicateField(String),
	UnknownField(String),
	BadFile(String),
	BadFormatting(String),
}

impl std::error::Error for SettingsReadError {}

impl fmt::Display for SettingsReadError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::MissingField(name) => write!(f, "Field missing from env file: {}", name),
			Self::DuplicateField(name) => write!(f, "Duplicate field in env file: {}", name),
			Self::UnknownField(name) => write!(f, "Unknown field provided: {}", name),
			Self::BadFile(e) => write!(f, "Error in reading file: {}", e),
			Self::BadFormatting(e) => write!(f, "Error in parsing file: {}", e),
		}
	}
}

impl From<std::io::Error> for SettingsReadError {
	fn from(error: std::io::Error) -> Self {
		Self::BadFile(error.to_string())
	}
}

impl From<core::num::ParseIntError> for SettingsReadError {
	fn from(error: core::num::ParseIntError) -> Self {
		Self::BadFormatting(error.to_string())
	}
}

impl From<core::num::ParseFloatError> for SettingsReadError {
	fn from(error: core::num::ParseFloatError) -> Self {
		Self::BadFormatting(error.to_string())
	}
}

impl From<core::str::ParseBoolError> for SettingsReadError {
	fn from(error: core::str::ParseBoolError) -> Self {
		Self::BadFormatting(error.to_string())
	}
}

impl From<std::convert::Infallible> for SettingsReadError {
	fn from(error: std::convert::Infallible) -> Self {
		Self::BadFormatting(error.to_string())
	}
}

macro_rules! Settings {
	($($field:ident : $t:ty),* $(,)? ) => {
		#[derive(Debug)]
		#[allow(non_snake_case)]
		pub struct Settings {
			$(pub $field: $t),*
		}

		#[allow(non_snake_case)]
		struct SettingsReadValues {
			$(pub $field: Option<$t>),*
		}

		impl SettingsReadValues {
			pub fn empty() -> Self {
				Self {
					$($field: None,)*
				}
			}
		}

		impl Settings {
			pub fn from_env_file<P>(filename: P) -> Result<Self, SettingsReadError>
			where P: AsRef<Path>, {
				let mut read_value = SettingsReadValues::empty();
				let lines = Self::read_lines(filename)?;

				for line in lines {
					let line_value = line?;
					let (field_name, field_value) = Self::line_parts(&line_value)?;
					let mut found = false;
					$(
						if !found && stringify!($field) == field_name {
							found = true;
							match read_value.$field {
								None => read_value.$field = Some(field_value.parse()?),
								Some(_) => return Err(SettingsReadError::DuplicateField(stringify!($field).to_string()))
							}
						}
					)*
					if !found {
						return Err(SettingsReadError::UnknownField(field_name.to_string()));
					}
				}

				Ok(
					Self {
						$(
							$field: read_value.$field.ok_or(
								SettingsReadError::MissingField(stringify!($field).to_string())
							)?,
						)*
					}
				)
			}

			fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
			where P: AsRef<Path>, {
					let file = File::open(filename)?;
					Ok(io::BufReader::new(file).lines())
			}

			fn line_parts(line: &str) -> Result<(&str, &str), SettingsReadError> {
				let mut splitter = line.splitn(2, "=");
				let error_maker =
					|| SettingsReadError::BadFormatting(format!("bad line formatting: \"{}\"", line).to_string());
				let field_name = splitter.next().ok_or_else(error_maker)?;
				let field_value = splitter.next().ok_or_else(error_maker)?;

				Ok((field_name, field_value))
			}
		}
	};
}

Settings! {
	APPSYNC_HTTP_DOMAIN: String,
	APPSYNC_PUBLISH_URL: String,
	APPSYNC_API_KEY: String,
	APPSYNC_WEBSOCKET_URL: String,
}
