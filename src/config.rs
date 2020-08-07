use std::{
    error
};

use super::wikifile;

#[derive(Deserialize, Serialize, Debug)]
pub struct ConfigContainer {
    pub config : ConfigurationStruct,
    pub header : wikifile::PageRevisionStruct
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
#[serde(default)]
pub struct ConfigurationStruct {
	case_sensitive : bool, // This should be set if the file system and thus wiki page names are case sensitive. If in doubt set to false.
	authenticationequired_for_read : bool, // If true unautheticated users can read wiki pages
	authentication_required_for_logging : bool, // Allows unauthenticated users to log debug. This is a potential denial of service vector.
	allow_media_overwrite : bool, // Set to true to allow the overwriting media files on uploads.
	start_page : String, // the page loaded by default as the starting wiki page.
	number_of_concurrent_locks : u32, // The number of pages which can be concurrently locked for editing.
	max_number_of_users : u32, // The maximum number of users
	max_velocity : u32, // Minimum time in nanoseconds between authenticated requests from an IP address
	unauth_max_velocity : u32, // Minimum time in nanoseconds between unauthenticated requests from an IP address
	admin_users : Vec<String>, // An array of admin user names
	admin_pages : Vec<String> // An array of pages and rest calls only available to admim users
}


/// Tries to load the config file for a tiny wiki
pub fn load_config() -> Result<ConfigContainer, Box<dyn error::Error>> {
    if let Ok((cfg, hdr)) = wikifile::load_parts("site/wiki/_config/current") {
//		let tres: Result<ConfigurationStruct, _> = serde_json::from_str(&cfg);
        return Ok(ConfigContainer{config: serde_json::from_str(&cfg)?, header: hdr});
    }
    Err("Failed to load".into())
}
