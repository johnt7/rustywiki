use chrono::Utc;

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
    jsuser,
    user::{User, AuthState},
    wikifile
};



#[derive(FromForm)]
pub struct Login {
    username: String,
    password: String
}

// Wrapper for auth data
wrapper!(AuthStruct, HashMap<String, UserStruct>);

/// Use to load and write to file
#[derive(Serialize, Deserialize, Debug)]
struct Wrapper {
    #[serde(rename = "Userlist")]
    pub user_list: Vec<UserStruct>
}


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct UserStruct {
	pub user : String, 
	pub password : String, 
	pub salt : String, 
	pub comment : String 
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

pub fn save_auth(auth: &wikifile::WikiStruct<AuthStruct>, user: &str) -> Result<(), Box<dyn error::Error>> {
    let auth_hash = &auth.read().unwrap().data;
    let mut auth_vers = auth.read().unwrap().header.clone();
    let user_hash: Vec<_> = auth_hash.iter().map(|(_,v)| v).collect();
    let uh_str = serde_json::to_string_pretty(&user_hash).unwrap();
    auth_vers.previous_revision = auth_vers.revision.to_owned();
    let new_vers = auth_vers.revision.parse::<usize>().unwrap() + 1;
    auth_vers.revision = format!("{:09}", new_vers); 
    auth_vers.revision_date = Utc::now().format("%Y/%m/%d %H:%M:%S.%.3f%:z").to_string(); // format "2018/05/12 19:53:05.248-07:00"
    auth_vers.revised_by = user.to_owned();
    wikifile::write_parts(&auth_vers, &uh_str)?;
    Ok(())   
}

// TODO - not happy with the encapsulation, look at refactoring
pub fn login_handle(login: Form<Login>, cookies: &mut Cookies<'_>, umap: &wikifile::WikiStruct<AuthStruct>, cfg: &config::WikiConfig) -> Option<User> {
    let user_hash = &umap.read().unwrap().data;

    let entry = user_hash.get(&login.username)?;
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

pub fn delete_user( umap: &wikifile::WikiStruct<AuthStruct>, user_name: &str) -> bool {
    let user_hash = &mut umap.write().unwrap().data;
    user_hash.0.remove(user_name).is_some()
}

pub fn modify_user( umap: &wikifile::WikiStruct<AuthStruct>, user_info: &jsuser::UserModify) -> bool {
    let user_hash = &mut umap.write().unwrap().data;
    let entry = user_hash.0.entry(user_info.user.to_owned()).or_default();
    entry.user = user_info.user.to_owned();
    entry.password = user_info.new_password.to_owned(); // TODO, change to salt handling
    entry.comment = user_info.comment.to_owned();
    true
}