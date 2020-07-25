#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate clap;
#[macro_use] extern crate rocket;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json::Json;
use rocket::{State, Request};
use rocket::response::{Responder, Response};
use rocket::http::RawStr;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::Data;

use simplelog::{SimpleLogger, LevelFilter, Config};
//use rocket::request::Form;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::ops::Deref;

const DATE_FORMAT : &str = "%Y/%m/%d %H:%M:%S%.3f";
const DELIMETER : &str = "<!--REVISION HEADER DEMARCATION>";

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

/*
TODO - return all the strange 500 errors from the GO
pub struct WikiResponse {
    value: u16,
    str: String
}

#[catch(404)]
fn not_found(req: &rocket::Request) -> content::Html<String> {
    content::Html(format!("<p>Sorry, but '{}' is not a valid path!</p>
            <p>Try visiting /hello/&lt;name&gt;/&lt;age&gt; instead.</p>",
            req.uri()))
}

impl Responder<'static> for WikiResponse {
    fn respond_to(self, _: &Request) ->  Response {
    let mut response = Response::new();
    response.set_status(Status::new(self.value, self.str.to_owned()));
    return response
   }
}
*/
const SDF1 : &str = r#"{
	"Page": "start",
	"Revision": "000000000",
	"PreviousRevision": "000000000",
	"CreateDate": "2018/05/05 19:53:05.248-07:00",
	"RevisionDate": "2018/05/05 19:53:05.248-07:00",
	"RevisedBy": "user",
	"Comment": ""
}
<!--REVISION HEADER DEMARCATION>
"#;

const SDF2 : &str = r#"{
	"Page"sdf2: "_user",
	"Revision": "000000001",
	"PreviousRevision": "000000000",
	"CreateDate": "2018/05/12 19:53:05.248-07:00",
	"RevisionDate": "2018/05/12 19:53:05.248-07:00",
	"RevisedBy": "user",
	"Comment": "Initial save"
}
<!--REVISION HEADER DEMARCATION>
{
	"Userlist": [
		{
			"User": "user",
			"Password": "pwd",
			"Salt": "",
			"Comment": ""
		},
		{
			"User": "root",
			"Password": "pwd",
			"Salt": "",
			"Comment": ""
		}
	]
}"#;

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
 Revision : String,
 PreviousRevision : String,
 CreateDate : String,
 RevisionDate : String,
 RevisedBy : String,
 Comment : String,
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
    #[allow(non_snake_case)]
    imageName : String
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct UserDelete {
 User : String
}

#[derive(Deserialize, Serialize)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
struct onfigurationStruct {
	CaseSensitive : bool, // This should be set if the file system and thus wiki page names are case sensitive. If in doubt set to false.
	AuthenticationRequiredForRead : bool, // If true unautheticated users can read wiki pages
	AuthenticationRequiredForLogging : bool, // Allows unauthenticated users to log debug. This is a potential denial of service vector.
	AllowMediaOverwrite : bool, // Set to true to allow the overwriting media files on uploads.
	StartPage : String, // the page loaded by default as the starting wiki page.
	NumberOfConcurrentLocks : u32, // The number of pages which can be concurrently locked for editing.
	MaxNumberOfUsers : u32, // The maximum number of users
	MaxVelocity : u32, // Minimum time in nanoseconds between authenticated requests from an IP address
	UnauthMaxVelocity : u32, // Minimum time in nanoseconds between unauthenticated requests from an IP address
	AdminUsers : Vec<String>, // An array of admin user names
	AdminPages : Vec<String> // An array of pages and rest calls only available to admim users
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
/*
impl AuthStruct {
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
struct RequestDelayStruct {
    delay : Duration,
    last : Instant
}

struct DelayMap ( Arc<Mutex<HashMap<String, RequestDelayStruct>>> );
impl Deref for DelayMap {
    type Target = Mutex<HashMap<String, RequestDelayStruct>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
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
#[post("/jsLog/Error", data = "<input>")]
fn rocket_route_js_error(input: Json<LogData>) -> String {
    error!("RustyWiki Err: {}", input.LogText.chars().collect::<String>());
    String::from("Ok")
}
#[post("/jsLog/Exception", data = "<input>")]
fn rocket_route_js_exception(input: Json<LogData>) -> String {
    error!("RustyWiki Exc: {}", input.LogText.chars().collect::<String>());
    String::from("Ok")
}
#[post("/jsLog/<rq>", data = "<input>")]
fn rocket_route_js_log(rq: &RawStr, input: String) -> String {
    info!("RustyWiki Log failed parse: {} {}", rq.as_str(), input);
    String::from("520")
}
#[post("/jsUser/UserModify", data = "<input>")]
fn rocket_route_user_modify(input: Json<UserModify>) -> String {
    debug!("user modify {} {}", input.User, input.Password);
    // make sure user is authenticated
    String::from("Ok")
}
#[post("/jsUser/Wikisave", data = "<input>")]
fn rocket_route_wiki_save(lock_data : State<PageMap>, input: Json<Wikisave>) -> Status {
    debug!("wiki save {} {}", input.Lock, input.PreviousRevision);
    if input.Revision == "" || input.PreviousRevision == "" {
        return Status::new(519, "no revision of previous revision");
    }
    if input.Page == "" || input.Lock == "" {
        return Status::new(519, "no lock or page");
    }
    let mp = lock_data.lock().unwrap();
    if let Some(lock_token) = mp.get(&input.Page) {
        if lock_token != &input.Lock {
            return Status::new(520, "wrong lock");
        }
    } else {
        return Status::new(521, "wrong lock");
    }
    // make sure directory /wiki/input.page exists
    // open /wiki/input.page/input.revision
    // write data to file
    // close file
    // write to /wiki/input.page/current
    // close file
    Status::Ok
}
#[post("/jsUser/Wikilock", data = "<input>")]
fn rocket_route_user_lock(lock_data : State<PageMap>, input: Json<Wikilock>) -> Status {
    if input.Page == "" || input.Lock == "" {
        return Status::new(519, "no lock page");
    }
    let mut mp = lock_data.lock().unwrap();
    if let Some(_) = mp.get(&input.Page) {
        return Status::new(520, "already locked");
    }
    let res = mp.insert(input.Page.clone(), input.Lock.clone());
    let ct = mp.len();
    info!("user lock len = {} res={:?}", ct, res);
    Status::Ok
}

#[post("/jsUser/Wikiunlock", data = "<input>")]
fn rocket_route_user_unlock(lock_data : State<PageMap>, input: Json<Wikilock>) -> Option<String> {
     if input.Page == "" {
        return None;
    }
    let mut mp = lock_data.lock().unwrap();
    if let Some(ll) = mp.get(&input.Page) {
        if ll != &input.Lock {
            return None;
        }
    } else {
        return None;
    }
    let res = mp.remove(&input.Page);

    let ct = mp.len();
    info!("user lock {} {} = len={} res={:?}", input.Lock, input.Page, ct, res);
    Some(String::from("Ok"))
}
#[post("/jsAdmin/UserDelete", data = "<input>")]
fn rocket_route_user_delete(input: Json<UserDelete>) -> String {
    debug!("user delete {}", input.User);
    String::from("Ok")
}

#[post("/jsUser/Upload", data = "<_input>")]
fn rocket_route_user_upload(content_type: &ContentType, _input: Data) -> String {
    debug!("user upload {}", content_type);
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
fn rocket_route_page(page_name : String) -> io::Result<String> {
    debug!("get page {}", page_name);
    // TODO use the site setting in config
//    let mut file = File::open("site/index.html")?;
    //let mut buf = BufReader::new(file);
//    let mut filedata = Vec::new();
    let sdata = fs::read_to_string("site/index.html")?;
//    let sdata = filedata.to_string();
    Ok(sdata.replace("DUMMYSTARTPAGE", &page_name))
//    Ok(sdata)
}

fn _testserde() {
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
    al.Userlist.push(u1.clone());
    al.Userlist.push(u2.clone());
    let s2 = serde_json::to_string(&al).unwrap();
    let al2 : AuthlistStruct = serde_json::from_str(&s2).unwrap();
    println!("s2 = {}", s2);
    println!("al2 = {:?}", al2);
    let mut altest = load_auth().unwrap();
    altest.UserMap.insert(u1.User.clone(), u1.clone());
    altest.UserMap.insert(u2.User.clone(), u2.clone());
    let s3 = serde_json::to_string(&al).unwrap();
    println!("s3 = {:?}", &s3);
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

fn split_version<'a>(in_str : &'a str) -> Result<(&'a str, &'a str), &'a str> {
    let v: Vec<&str> = in_str.split(DELIMETER).collect();
    match v.len() {
        0 => Ok( ("", "") ),
        1 => Ok( ("", v[0]) ),
        2 => Ok( (v[0], v[1]) ),
        _ => Err("Bad versioned string")
    }
}
fn join_version(ver_str : &str, data_str : &str) -> String {
    let mut res = String::with_capacity(ver_str.len()+data_str.len()+11);
    res.push_str(ver_str);
    res.push_str("\n");
    res.push_str(DELIMETER);
    res.push_str("\n");
    res.push_str(data_str);
    res
}

fn create_rocket() -> rocket::Rocket {
    let auth = match load_auth() {
        Some(a) => a,
        None => gen_auth()
    };
    let delay_map = DelayMap ( Arc::new(Mutex::new(HashMap::new())) );
    let lock_map = PageMap ( Arc::new(Mutex::new(HashMap::new())) );
    rocket::ignite()
    .manage(auth)
    .manage(delay_map)
    .manage(lock_map)
    .mount("/", StaticFiles::from("site"))  // use the site value from config
    .mount("/", routes![rocket_route_js_debug_no_trunc, 
        rocket_route_js_debug, rocket_route_js_exception, rocket_route_js_error, rocket_route_js_log,
        rocket_route_user_modify, rocket_route_wiki_save, rocket_route_user_lock,
        rocket_route_user_unlock, rocket_route_user_upload, rocket_route_user_delete, rocket_route_master_reset, 
        rocket_route_media_index, rocket_route_page])
}

fn main() {
    let _config = get_command_line();
    println!("got config={:?}", _config);
    /*
    testserde();
    let auth = match load_auth() {
        Some(a) => a,
        None => gen_auth()
    };
    let delay_map = DelayMap ( Arc::new(Mutex::new(HashMap::new())) );
    let lock_map = PageMap ( Arc::new(Mutex::new(HashMap::new())) );
    let d1 = Wikilock { Page: "foo".to_string(), Lock: "something".to_string() };
    lock_map.lock().unwrap().insert(d1.Page, d1.Lock);
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());    
    rocket::ignite()
    .manage(auth)
    .manage(delay_map)
    .manage(lock_map)
    .mount("/", StaticFiles::from("site"))  // use the site value from config
    .mount("/", routes![rocket_route_js_debug_no_trunc, 
        rocket_route_js_debug, rocket_route_js_exception, rocket_route_js_error, rocket_route_js_log,
        rocket_route_user_modify, rocket_route_wiki_save, rocket_route_user_lock,
        rocket_route_user_unlock, rocket_route_user_upload, rocket_route_user_delete, rocket_route_master_reset, 
        rocket_route_media_index, rocket_route_page])
        */
    let rkt = create_rocket();
    rkt.launch();
}
