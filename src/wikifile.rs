use std::{
    error,
    fs,
    path::Path
};


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PageRevisionStruct {
	pub page : String,
	pub revision : String,
	pub previous_revision : String,
	pub create_date : String,
	pub revision_date : String,
	pub revised_by : String,
	pub comment : String,
	pub lock : String,
	pub data : String
}

/// Tries to load a tinywiki file, splitting it into two string, the content and the version info
pub fn load_parts<P: AsRef<Path>>(path: P) -> Result<(String, PageRevisionStruct), Box<dyn error::Error>> {
    let fs = fs::read_to_string(path)?;
    let res = super::split_version(&fs)?;
    let header: PageRevisionStruct = serde_json::from_str(res.0)?;
    Ok((res.1.to_string(), header))
}

