use std::{
    error,
    sync::RwLock
};

use super::{
	config,
	wikifile
};

pub type WikiConfig = wikifile::WikiStruct<config::ConfigurationStruct>;

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
#[serde(default)]
/// represents the tinywiki configuration file
pub struct ConfigurationStruct {
	pub case_sensitive : bool, // This should be set if the file system and thus wiki page names are case sensitive. If in doubt set to false.
	pub authentication_required_for_read : bool, // If true unautheticated users can read wiki pages
	pub authentication_required_for_logging : bool, // Allows unauthenticated users to log debug. This is a potential denial of service vector.
	pub allow_media_overwrite : bool, // Set to true to allow the overwriting media files on uploads.
	pub start_page : String, // the page loaded by default as the starting wiki page.
	pub number_of_concurrent_locks : u32, // The number of pages which can be concurrently locked for editing.
	pub max_number_of_users : u32, // The maximum number of users
	pub max_velocity : u32, // Minimum time in nanoseconds between authenticated requests from an IP address
	pub unauth_max_velocity : u32, // Minimum time in nanoseconds between unauthenticated requests from an IP address
	pub admin_users : Vec<String>, // An array of admin user names
	pub admin_pages : Vec<String> // An array of pages and rest calls only available to admim users
}

/// Loads it into a rocket sharable, with a lock
pub fn load_config() -> Result<WikiConfig, Box<dyn error::Error>> {
	return Ok(wikifile::WikiStruct(RwLock::new( load_config_int()? )))
}

/// try to load core structure from the core data for a tiny wiki config
pub fn load_config_int() -> Result<wikifile::WikiContainer<ConfigurationStruct>, Box<dyn error::Error>> {
    if let Ok((cfg, hdr)) = wikifile::load_parts(wikifile::get_path("wiki/_config/current")) {
		Ok(wikifile::WikiContainer{data: serde_json::from_str(&cfg)?, header: hdr})
    } else {
		Err("Failed to load".into())
	}
}
