use rocket::{
    Data,
    http::{ContentType, Status},
    State
};

use rocket_contrib::{
    json::Json
};

use super::{
    authstruct,
    pagemap::PageMap,
    user::{self, PageWriter},
    wikifile
};


#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserModify {
    pub user: String,
    pub password: String,
    pub new_password: String,
    pub new_password_check: String,
    pub comment: String
}


// also used in unlock
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Wikilock {
 page : String,
 lock : String
}

#[post("/jsUser/UserModify", data = "<input>")]
pub fn rocket_route_user_modify(user: PageWriter, input: Json<UserModify>, auth: State<super::WikiStruct<authstruct::AuthStruct>>) 
-> Status
{
    debug!("user modify {} {} {} {} {}", input.user, input.password, input.new_password, input.new_password_check, input.comment);
    if user.0.auth != user::AuthState::AuthAdmin && user.0.name != input.user { return Status::Unauthorized };
    if !authstruct::modify_user(&auth, &input) {
        error!("failed to modify");
        return Status::new(519, "failed to modify");
    }
    if let Err(_) = authstruct::save_auth(&auth, &user.name) {
        return Status::new(519, "failed to save");
    }
    Status::Ok
}

#[post("/jsUser/Wikisave", data = "<input>")]
/// save a wiki page, updating revision info
pub fn rocket_route_wiki_save(_user: PageWriter, lock_data : State<PageMap>, input: Json<wikifile::PageRevisionStruct>) -> Status {
    if input.revision == "" || input.previous_revision == "" {
        return Status::new(519, "no revision or previous revision");
    }
    if input.page == "" || input.lock == "" {
        return Status::new(519, "no lock or page");
    }
    
    let mp = lock_data.read().unwrap();
    if let Some(lock_token) = mp.get(&input.page) {
        if lock_token != &input.lock {
            return Status::new(520, "wrong lock");
        }
    } else {
        return Status::new(521, "no lock");
    }
    match wikifile::write_parts(&input, &input.data) {
        Ok(_) => Status::Ok,
        _ => Status::new(522, "failed to write")
    }
}

#[post("/jsUser/Wikilock", data = "<input>")]
/// Locks wiki entry for editing
pub fn rocket_route_user_lock(_user: PageWriter, lock_data : State<PageMap>, input: Json<Wikilock>) -> Status {
    if input.page == "" || input.lock == "" {
        return Status::new(519, "no lock page");
    }
    let mut mp = lock_data.write().unwrap();
    if let Some(_) = mp.get(&input.page) {
        return Status::new(520, "already locked");
    }
    let res = mp.insert(input.page.clone(), input.lock.clone());
    let ct = mp.len();
    info!("user lock len = {} res={:?}", ct, res);
    Status::Ok
}

#[post("/jsUser/Wikiunlock", data = "<input>")]
/// Unlocks wiki entry after editing
pub fn rocket_route_user_unlock(_user: PageWriter, lock_data : State<PageMap>, input: Json<Wikilock>) -> Status {
     if input.page == "" {
        return Status::new(540, "bad page");
    }
    let mut mp = lock_data.write().unwrap();
    // find if there is a lock on the page and if so make sure the lock tokens match
    if let Some(ll) = mp.get(&input.page) {
        if ll != &input.lock {
            return Status::new(540, "lock doesn't match");
        }
    } else {
        return Status::new(540, "lock not found");;
    }
    let res = mp.remove(&input.page);
    if let Some(_) = res {
        Status::Ok
    } else {
        Status::new(520, "Failed to remove the lock")
    }
}

// TODO
#[post("/jsUser/Upload", data = "<_input>")]
/// Upload a media file
pub fn rocket_route_user_upload(_user: PageWriter, content_type: &ContentType, _input: Data) -> String {
    debug!("user upload {}", content_type);
    String::from("Ok")
}
