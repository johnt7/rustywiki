use rocket::{
    State
};
use rocket_contrib::{
    json::Json
};
use std::{
    collections::HashMap,
    error,
    ops::Deref,
    sync::{RwLock},
    time::{Duration, Instant},
};
use super::{
    authstruct,
    config,
    media,
    pagemap,
    user
};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct UserDelete {
 user : String
}

pub struct RequestDelayStruct {
    _delay : Duration,
    _last : Instant
}

pub struct DelayMap ( RwLock<HashMap<String, RequestDelayStruct>> );
/// Structure passed to Rocket to allow storage of delay  information
impl DelayMap {
    pub fn new() -> DelayMap {
        DelayMap ( RwLock::new(HashMap::new()) )
    }
}
impl Deref for DelayMap {
    type Target = RwLock<HashMap<String, RequestDelayStruct>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


/// do master reset of the system
#[get("/jsAdmin/MasterReset")]
pub fn rocket_route_master_reset(_user: user::PageAdmin, 
    delay_map: State<DelayMap>, page_locks: State<pagemap::PageMap>, 
    auth: State<super::WikiStruct<authstruct::AuthStruct>>, cfg: State<config::WikiConfig>, mi: State<media::MediaIndex>) 
    -> String {
        *delay_map.write().unwrap() = HashMap::new();
        *page_locks.write().unwrap() = HashMap::new();
        *auth.write().unwrap() =  authstruct::load_auth_int().unwrap();
        *cfg.write().unwrap() =  config::load_config_int().unwrap();
        *mi.write().unwrap() = media::media_str();
        String::from("Ok")
}

#[post("/jsAdmin/UserDelete", data = "<input>")]
pub fn rocket_route_user_delete(admin: user::PageAdmin, input: Json<UserDelete>, auth: State<super::WikiStruct<authstruct::AuthStruct>>) 
-> Result<String, Box<dyn error::Error>> {
     error!("user delete {}", input.user);
    if !authstruct::delete_user(&auth, &input.user) {
        error!("failed to delete");
        return Err("Failed to Delete".into());
    }
    authstruct::save_auth(&auth, &admin.name)?;
    Ok(String::from("Ok"))
}
