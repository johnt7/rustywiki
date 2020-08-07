use std::{
    error,
    fs,
    path::Path
};

const DELIMETER : &str = "<!--REVISION HEADER DEMARCATION>";

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
    #[serde(default)]
    pub lock : String,
    #[serde(default)]
	pub data : String
}

// TODO - handle local file path handling??
/// Tries to load a tinywiki file, splitting it into two string, the content and the version info
pub fn load_parts<P: AsRef<Path>>(path: P) -> Result<(String, PageRevisionStruct), Box<dyn error::Error>> {
    let fs = fs::read_to_string(path)?;
    let res = split_version(&fs)?;
//    let x : Result<PageRevisionStruct ,_> = serde_json::from_str(res.0);
    let header: PageRevisionStruct = serde_json::from_str(res.0)?;
    Ok((res.1.to_string(), header))
}


fn split_version<'a>(in_str : &'a str) -> Result<(&'a str, &'a str), &'a str> {
    let v: Vec<&str> = in_str.split(DELIMETER).collect();
    match v.len() {
        2 => Ok( (v[0], v[1]) ),
        _ => Err("Bad versioned string")
    }
}



fn _join_version(ver_str : &str, data_str : &str) -> String {
    let mut res = String::with_capacity(ver_str.len()+data_str.len()+11);
    res.push_str(ver_str);
    res.push_str("\n");
    res.push_str(DELIMETER);
    res.push_str("\n");
    res.push_str(data_str);
    res
}

