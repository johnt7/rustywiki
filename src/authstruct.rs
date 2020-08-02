use std::{
    collections::HashMap,
    ops::Deref,
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
    pub UserMap : HashMap<String, UserStruct>,
    pub Header : PageRevisionStruct
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

pub fn load_auth() -> Option<AuthStruct> {
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
}

pub fn gen_auth() -> AuthStruct {
    AuthStruct(Arc::new(Mutex::new(AuthStructInternal{     
        UserMap : HashMap::new(),
        Header : PageRevisionStruct {
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