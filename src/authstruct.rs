use std::{
    collections::HashMap,
    error,
    ops::Deref,
    sync::RwLock
};

use rocket::{
    http::{Cookies, Cookie},
    request::Form
};

use super::{
    config,
    user::{User, AuthState},
    wikifile
};




// Wrapper for auth data
wrapper!(AuthStruct, HashMap<String, UserStruct>);

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

/// Tries to load the user file for a tiny wiki
pub fn load_auth() -> Result<wikifile::WikiStruct<AuthStruct>, Box<dyn error::Error>> {
	return Ok(wikifile::WikiStruct(RwLock::new( load_auth_int()? )))
}

/// Tries to load the user file to an wiki container
pub fn load_auth_int() -> Result<wikifile::WikiContainer<AuthStruct>, Box<dyn error::Error>> {
    // TODO, look at cleaning this code up a bit
    if let Ok((um, hdr)) = wikifile::load_parts(wikifile::get_path("wiki/_user/current")) {
            let umwin: Wrapper = serde_json::from_str(&um)?;
        let umap = AuthStruct(
            umwin.user_list.iter().map(   |us| (us.user.clone(), us.clone())    ).collect()
        );
        return Ok(wikifile::WikiContainer{data: umap, header: hdr})
    }
    Err("Failed to load".into())
}


// TODO - not happy with the encapsulation, look at refactoring
pub fn login_handle(login: Form<super::Login>, cookies: &mut Cookies<'_>, umap: &wikifile::WikiStruct<AuthStruct>, cfg: &config::WikiConfig) -> Option<User> {
    let thing = &umap.read().unwrap().data;

    let entry = thing.get(&login.username)?;
    if entry.password != login.password { return None };
    let adm_list = &cfg.read().unwrap().data.admin_users;

    let lvl = match adm_list.iter().any(|n| n==&login.username) {
        true => AuthState::AuthAdmin,
        false => AuthState::AuthUser
    };
    let u_tok = User {auth:lvl, name: login.username.to_string()};
    cookies.add_private(Cookie::new("wiki_auth", u_tok.to_string()));
    Some(u_tok)
}

