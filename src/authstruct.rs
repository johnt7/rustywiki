use std::{
    collections::HashMap,
    error,
    fs,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex}
};
 

pub struct AuthStruct (Arc<Mutex<AuthStructInternal>>);
impl Deref for AuthStruct {
    type Target = Arc<Mutex<AuthStructInternal>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct AuthStructInternal {
    pub user_map : HashMap<String, UserStruct>,
    pub header : PageRevisionStruct
}
/*
impl AuthStructInternal {
    fn save_to_string(&self) -> String {
        let ul : 5;
        let mut res_str = String::new();
        hdr = serde_json::to_string(self.Header).unwrap();
        res_str.append(self.UserMap);
        res_str.append(&DELIM)
        res_str
    }
}
*/



#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct UserStruct {
	pub User : String, 
	pub Password : String, 
	pub Salt : String, 
	pub Comment : String 
}


#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct PageRevisionStruct {
	pub Page : String,
	pub Revision : String,
	pub PreviousRevision : String,
	pub CreateDate : String,
	pub RevisionDate : String,
	pub RevisedBy : String,
	pub Comment : String,
	pub Lock : String,
	pub Data : String
}

fn load_parts<P: AsRef<Path>>(path: P) -> Result<(String, String), Box<dyn error::Error>> {
    let fs = fs::read_to_string("site/wiki/_user/current")?;
    let res = super::split_version(&fs)?;
    Ok((res.0.to_string(), res.1.to_string()))
}


pub fn load_auth() -> Result<AuthStruct, Box<dyn error::Error>> {
    if let Ok((um, hdr)) = load_parts("site/wiki/_user/current") {
        return Ok(AuthStruct(Arc::new(Mutex::new(AuthStructInternal{user_map: serde_json::from_str(&um)?, header: serde_json::from_str(&hdr)?}))))
    }
    error!("failed to load authorizations");
    Err("Failed to load".into())
    /*
    Some(AuthStruct(Arc::new(Mutex::new(AuthStructInternal{     
        UserMap : HashMap::new(),
        Header : PageRevisionStruct {
            Page : String::new(),
            Revision : String::new(),
            PreviousRevision : String::new(),
            CreateDate : String::new(),
            RevisionDate : String::new(),
            RevisedBy : String::new(),
            Comment : String::new(),
            Lock : String::new(),
            Data : String::new()
        }
    }))))
    */
}

pub fn gen_auth() -> AuthStruct {
    AuthStruct(Arc::new(Mutex::new(AuthStructInternal{     
        user_map : HashMap::new(),
        header : PageRevisionStruct {
            Page : "_user".to_string(),
            Revision : "000000000".to_string(),
            PreviousRevision : "000000000".to_string(),
            CreateDate : String::new(),
            RevisionDate : String::new(),
            RevisedBy : String::new(),
            Comment : String::new(),
            Lock : String::new(),
            Data : String::new()
        }
    })))
}