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
#[allow(non_snake_case)]
pub struct AuthStructInternal {
    pub user_map : HashMap<String, UserStruct>,
    pub header : PageRevisionStruct
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Wrapper {
    pub Userlist: Vec<UserStruct>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserList(pub Vec<UserStruct>);

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
    let fs = fs::read_to_string(path)?;
    let res = super::split_version(&fs)?;
    Ok((res.1.to_string(), res.0.to_string()))
}

fn adj_list(instr: &str) -> String {
    let si = instr.find('{').unwrap_or(0);
    let fi = instr.rfind('}').unwrap_or(instr.len());
    instr[si+1..fi].to_string()
}

pub fn load_auth() -> Result<AuthStruct, Box<dyn error::Error>> {
    if let Ok((um, hdr)) = load_parts("site/wiki/_user/current") {
        println!("la found");
        let x1: Result<HashMap<String, UserStruct>,_> = serde_json::from_str(&um);
        let x1a: Result<Vec<UserStruct>,_> = serde_json::from_str(&um);
        let x1b: Result<Wrapper,_> = serde_json::from_str(&um);

        println!("ums:{:?}", um);
        println!("um:{:?}", x1);
        println!("umA:{:?}", x1a);
        println!("umB:{:?}", x1b);
        let x2: Result<PageRevisionStruct,_> = serde_json::from_str(&hdr);

        let fs = adj_list(&um);
        println!("fs={}", fs);
        let fsin: Result<UserList,_> = serde_json::from_str(&fs);
        println!("fsin={:?}", fsin);

        let umwin: Wrapper = serde_json::from_str(&um)?;
        let fswin: Result<Wrapper,_> = serde_json::from_str(&fs);
        println!("umin={:?}", umwin);
        println!("fswin={:?}", fswin);

        println!("hdrs:{:?}",hdr);
        println!("hdrs:{:?}", x2);

        let umap = umwin.Userlist.iter().map(|us| (us.User.clone(), us.clone())).collect();

        return Ok(AuthStruct(Arc::new(Mutex::new(AuthStructInternal{user_map: umap, header: serde_json::from_str(&hdr)?}))))
//        return Ok(AuthStruct(Arc::new(Mutex::new(AuthStructInternal{user_map: serde_json::from_str(&um)?, header: serde_json::from_str(&hdr)?}))))
    }
    println!("failed to load authorizations");
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