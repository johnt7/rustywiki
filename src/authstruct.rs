use std::{
    collections::HashMap,
    error,
    fs,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex}
};

#[derive(Debug)]
pub struct AuthStruct (Arc<Mutex<AuthStructInternal>>);
impl Deref for AuthStruct {
    type Target = Arc<Mutex<AuthStructInternal>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthStructInternal {
    pub user_map : HashMap<String, UserStruct>,
    pub header : PageRevisionStruct
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wrapper {
    #[serde(rename = "Userlist")]
    pub user_list: Vec<UserStruct>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UserStruct {
	pub user : String, 
	pub password : String, 
	pub salt : String, 
	pub comment : String 
}

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
fn load_parts<P: AsRef<Path>>(path: P) -> Result<(String, String), Box<dyn error::Error>> {
    let fs = fs::read_to_string(path)?;
    let res = super::split_version(&fs)?;
    Ok((res.1.to_string(), res.0.to_string()))
}

/// Tries to load the config file for a tiny wiki
pub fn load_auth() -> Result<AuthStruct, Box<dyn error::Error>> {
    if let Ok((um, hdr)) = load_parts("site/wiki/_user/current") {
        let umwin: Wrapper = serde_json::from_str(&um)?;
        // TODO, examine to see about cleaning this code up a bit
        let umap = umwin.user_list.iter().map(|us| (us.user.clone(), us.clone())).collect();
        return Ok(AuthStruct(Arc::new(Mutex::new(AuthStructInternal{user_map: umap, header: serde_json::from_str(&hdr)?}))))
    }
    Err("Failed to load".into())
}

/// Generates an AuthStruct - debug only
pub fn gen_auth() -> AuthStruct {
    AuthStruct(Arc::new(Mutex::new(AuthStructInternal{     
        user_map : HashMap::new(),
        header : PageRevisionStruct {
            page : "_user".to_string(),
            revision : "000000000".to_string(),
            previous_revision : "000000000".to_string(),
            create_date : String::new(),
            revision_date : String::new(),
            revised_by : String::new(),
            comment : String::new(),
            lock : String::new(),
            data : String::new()
        }
    })))
}