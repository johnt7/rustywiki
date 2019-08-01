#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate clap;
#[macro_use] extern crate rocket;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json::Json;
use rocket::State;

use simplelog::{SimpleLogger, LevelFilter, Config};
//use rocket::request::Form;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::ops::Deref;

const DATE_FORMAT : &str = "%Y/%m/%d %H:%M:%S%.3f";

/// Command line configuration information
#[derive(Debug)]
pub struct ConfigInfo {
    log_file_name : String,
    site : String,
    console_log : bool,
    cert : String,
    key : String,
    port : u16
}

/*
#[derive(FromForm)]
struct Task {
   description: String,
   completed: bool
}
*/

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct LogData {
    LogText: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct UserModify {
 User: String,
 Password: String,
 NewPassword: String,
 NewPasswordCheck: String,
 Comment: String
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Wikisave {
 Page : String,
 Revision : u16,
 PreviousRevision : u16,
 CreateDate : String,
 RevisionDate : String,
 RevisedBy : String,
 comment : String,
 Lock : String,
 Data : String
}

// also acts as unlock
#[derive(Clone, Deserialize, Debug)]
#[allow(non_snake_case)]
struct Wikilock {
 Page : String,
 Lock : String
}

#[derive(FromForm)]
#[allow(non_snake_case)]
struct Upload {
    uploadfile : String,
    token : String,
    imageName : String
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct UserDelete {
 User : String
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct AuthlistStruct {
    Userlist : Vec<UserStruct>
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct PageRevisionStruct {
	Page : String,
	Revision : String,
	PreviousRevision : String,
	CreateDate : String,
	RevisionDate : String,
	RevisedBy : String,
	Comment : String,
	Lock : String,
	Data : String
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct UserStruct {
	User : String, 
	Password : String, 
	Salt : String, 
	Comment : String 
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct AuthStruct {
    UserMap : HashMap<String, UserStruct>,
    Header : PageRevisionStruct
}

struct RequestDelayStruct {
    delay : Duration,
    last : Instant
}


struct PageMap ( Arc<Mutex<HashMap<String, String>>> );
impl Deref for PageMap {
    type Target = Mutex<HashMap<String, String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/// Loads information from the command line
fn get_command_line() -> ConfigInfo {
    let matches = clap_app!(rtinywiki =>
        (version: "0.0.0")
        (author: "Johnt <johnt7@gmail.com>")
        (about: "Rust Tiny Wiki")
        (@arg logfile: -l --log +takes_value "Log file name")
        (@arg cert: -c --cert +takes_value "Server certificate file")
        (@arg site: -s --site +takes_value "Location of site directory")
        (@arg key: -k --key +takes_value "Server key file")
        (@arg port: -p --port +takes_value "Port number")
        (@arg consoleLog: -d --dump "Dump log to console?")
    ).get_matches();
    ConfigInfo {
        log_file_name : matches.value_of("logfile").unwrap_or("site/wikiserver.log").to_string(),
        site : matches.value_of("site").unwrap_or("site").to_string(),
        cert : matches.value_of("cert").unwrap_or("server.cert").to_string(),
        key : matches.value_of("key").unwrap_or("server.key").to_string(),
        console_log : !matches.is_present("consoleLog"),
        port : matches.value_of("port").unwrap_or("9990").parse().unwrap_or(9990)
    }
}

#[post("/jsLog/DebugNoTrunc", data = "<input>", rank=1)]
fn rocket_route_js_debug_no_trunc(input: Json<LogData>) -> String {
    warn!("RustyWiki Dbg: {}", input.LogText);
    String::from("Ok t")
}
#[post("/jsLog/Debug", data = "<input>")]
fn rocket_route_js_debug(input: Json<LogData>) -> String {
    let in_length = input.LogText.len();
    warn!("RustyWiki Dbg({}): {}", in_length, input.LogText.chars().take(256).collect::<String>());
    String::from("Ok d")
}
#[post("/jsLog/Exception", data = "<input>")]
fn rocket_route_js_exception(input: Json<LogData>) -> String {
    error!("RustyWiki Exc: {}", input.LogText.chars().take(5).collect::<String>());
    String::from("Ok")
}
#[post("/jsLog", data = "<input>")]
fn rocket_route_js_log(input: Json<LogData>) -> String {
    info!("RustyWiki Log: {}", input.LogText.chars().take(5).collect::<String>());
    String::from("Ok")
}
#[post("/jsUser/UserModify", data = "<input>")]
fn rocket_route_user_modify(input: Json<UserModify>) -> String {
    debug!("user modify {} {}", input.User, input.Password);
    String::from("Ok")
}
#[post("/jsUser/Wikisave", data = "<input>")]
fn rocket_route_wiki_save(input: Json<Wikisave>) -> String {
    debug!("wiki save {} {}", input.Lock, input.PreviousRevision);
    String::from("Ok")
}
#[post("/jsUser/Wikilock", data = "<input>")]
fn rocket_route_user_lock(lock_data : State<PageMap>, input: Json<Wikilock>) -> Option<String> {
    info!("user lock {:?}", &input);
    let res;
    if input.Page == "" || input.Lock == "" {
        info!("bad page");
        return None;
    }
    if let Some(_) = lock_data.lock().unwrap().get(&input.Page) {
        info!("already locked page");
        return None;
    }
    info!("good page");
    res = lock_data.lock().unwrap().insert(input.Page.clone(), input.Lock.clone());
    info!("done page");
    let ct = lock_data.lock().unwrap().len();
    info!("user lock len = {} res={:?}", ct, res);
    Some(String::from("Ok"))
}

#[post("/jsUser/Wikiunlock", data = "<input>")]
fn rocket_route_user_unlock(lock_data : State<PageMap>, input: Json<Wikilock>) -> Option<String> {
    info!("user unlock {:?}", &input);
    let res;
    if input.Page == "" {
        info!("bad unlock");
        return None;
    }
    if let Some(ll) = lock_data.lock().unwrap().get(&input.Page) {
        info!("have unlock");
        if ll == &input.Lock {
         info!("match unlock");
        } else {
        info!("nomatch unlock");
            return None;
        }
    } else {
        info!("none unlock");
        return None;
    }
    res = lock_data.lock().unwrap().remove(&input.Page);
    info!("none unlock");
    let ct = lock_data.lock().unwrap().len();
    info!("user lock {} {} = len={} res={:?}", input.Lock, input.Page, ct, res);
    Some(String::from("Ok"))
}

#[post("/jsAdmin/UserDelete", data = "<input>")]
fn rocket_route_user_delete(input: Json<UserDelete>) -> String {
    debug!("user delete {}", input.User);
    String::from("Ok")
}
#[get("/jsAdmin/MasterReset")]
fn rocket_route_master_reset() -> String {
    debug!("master reset");
    String::from("Ok")
}
#[get("/page/MediaIndex")]
fn rocket_route_media_index() -> String {
    debug!("media index");
    String::from("Ok")
}
#[get("/page/<page_name>")]
fn rocket_route_page(page_name : String) -> String {
    debug!("get page {}", page_name);
    String::from("Ok")
}

fn testserde() {
    println!("testserde");
    let u1 = UserStruct {
        User : "u1".to_string(),
        Password : "u1p".to_string(), 
        Salt : "u1s".to_string(), 
        Comment : "u1c".to_string() 
    };
    let u2 = UserStruct {
        User : "u2".to_string(),
        Password : "u2p".to_string(), 
        Salt : "u2s".to_string(), 
        Comment : "u2c".to_string() 
    };
    println!("u1={:?}", u1);
    let mut al = AuthlistStruct { Userlist: Vec::new() };
    let serialized = serde_json::to_string(&u1).unwrap();
    println!("serialized = {}", serialized);
    al.Userlist.push(u1);
    al.Userlist.push(u2);
    let s2 = serde_json::to_string(&al).unwrap();
    let al2 : AuthlistStruct = serde_json::from_str(&s2).unwrap();
    println!("s2 = {}", s2);
    println!("al2 = {:?}", al2);
}
fn load_auth() -> Option<AuthStruct> {
    Some(AuthStruct{     
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
    })
}
fn gen_auth() -> AuthStruct {
    AuthStruct{     
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
    }
}
fn main() {
    let _config = get_command_line();
    println!("got config={:?}", _config);
    testserde();
    let auth = match load_auth() {
        Some(a) => a,
        None => gen_auth()
    };
    let delay_map : HashMap<String, RequestDelayStruct> =  HashMap::new();
    let lock_map = PageMap ( Arc::new(Mutex::new(HashMap::new())) );
    let d1 = Wikilock { Page: "foo".to_string(), Lock: "something".to_string() };
    lock_map.lock().unwrap().insert(d1.Page, d1.Lock);
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());    
    rocket::ignite()
    .manage(auth)
    .manage(delay_map)
    .manage(lock_map)
    .mount("/", StaticFiles::from("site"))
    .mount("/", routes![rocket_route_js_debug_no_trunc, 
        rocket_route_js_debug, rocket_route_js_exception, rocket_route_js_log,
        rocket_route_user_modify, rocket_route_wiki_save, rocket_route_user_lock,
        rocket_route_user_unlock, rocket_route_user_delete, rocket_route_master_reset, 
        rocket_route_media_index, rocket_route_page])
    .launch();
}
