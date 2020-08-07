use std::{
    collections::HashMap,
    error,
    ops::Deref,
    sync::{Arc, Mutex}
};

use rocket::http::{Cookies, Cookie};

use super::{
    user::{User, AuthState},
    wikifile
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
    pub header : wikifile::PageRevisionStruct
}



/// Use to load and write to file
#[derive(Serialize, Deserialize, Debug)]
struct Wrapper {
    #[serde(rename = "Userlist")]
    pub user_list: Vec<UserStruct>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UserStruct {
	user : String, 
	password : String, 
	salt : String, 
	comment : String 
}

/// Tries to load the config file for a tiny wiki
pub fn load_auth() -> Result<AuthStruct, Box<dyn error::Error>> {
        // TODO, examine to see about cleaning this code up a bit
        if let Ok((um, hdr)) = wikifile::load_parts("site/wiki/_user/current") {
        let umwin: Wrapper = serde_json::from_str(&um)?;
        let umap = umwin.user_list.iter().map(|us| (us.user.clone(), us.clone())).collect();
        return Ok(AuthStruct(Arc::new(Mutex::new(AuthStructInternal{user_map: umap, header: hdr}))))
    }
    Err("Failed to load".into())
}


// TODO - not happy with the encapsulation,
pub fn login_handle(uname: &str, pwd: &str, cookies: &mut Cookies<'_>, umap: &AuthStruct) -> Option<User> {
    let thing = &umap.lock().unwrap().user_map;
    // TODO handle no auth case
    let entry = thing.get(uname)?;
    if entry.password != pwd { return None };
    let u_tok = User {auth:AuthState::AuthAdmin, name: uname.to_string()}; // TODO get auth from list of admin
    cookies.add_private(Cookie::new("wiki_auth", u_tok.to_string()));
    Some(u_tok)
}



/// Generates an AuthStruct - debug only
pub fn gen_auth() -> AuthStruct {
    AuthStruct(Arc::new(Mutex::new(AuthStructInternal{     
        user_map : HashMap::new(),
        header : wikifile::PageRevisionStruct {
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