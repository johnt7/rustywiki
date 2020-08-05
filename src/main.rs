#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate clap;
#[macro_use] extern crate rocket;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Cursor},
    ops::Deref,
    path::*,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use rocket::{
    Data,
    http::{ContentType, Cookie, Cookies, RawStr, Status, Header},
    request::Form,
    Response,
    response::Redirect, 
    State
};

use rocket_contrib::{
    serve::StaticFiles,
    json::Json
};


// Modules
#[cfg(test)] mod tests;
mod auth;
mod basic;
mod authstruct;
use auth::{User};
use authstruct::{AuthStruct, UserStruct};



// Constants
const DATE_FORMAT : &str = "%Y/%m/%d %H:%M:%S%.3f";
const DELIMETER : &str = "<!--REVISION HEADER DEMARCATION>";

// types

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

#[derive(Responder)]
enum StringResponder {
    #[response(status=200, content_type="text/html")]
    Content(String),
    #[response(status=500)]
    Nothing(String)
}
*/

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
	"user_list": [
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


#[serde(rename_all = "PascalCase")]
#[derive(Deserialize)]
struct LogData {
    log_text: String,
}

#[derive(Deserialize)]
struct UserModify {
    #[serde(rename = "User")]
    user: String,
    #[serde(rename = "Password")]
    password: String,
    #[serde(rename = "NewPassword")]
    new_password: String,
    #[serde(rename = "NewPasswordCheck")]
    new_password_check: String,
    #[serde(rename = "Comment")]
    comment: String
}

#[serde(rename_all = "PascalCase")]
#[derive(Deserialize)]
struct Wikisave {
//    #[serde(rename = "Page")]
    page : String,
//    #[serde(rename = "Revision")]
    revision : String,
//    #[serde(rename = "PreviousRevision")]
    previous_revision : String,
//    #[serde(rename = "CreateDate")]
    create_date : String,
//    #[serde(rename = "RevisionDate")]
    revision_date : String,
//    #[serde(rename = "RevisedBy")]
    revised_by : String,
//    #[serde(rename = "Comment")]
    comment : String,
//    #[serde(rename = "Lock")]
    lock : String,
//    #[serde(rename = "Data")]
    data : String
}

// also acts as unlock
#[derive(Clone, Deserialize, Debug)]
#[allow(non_snake_case)]
struct Wikilock {
 Page : String,
 Lock : String
}

#[derive(FromForm)]
struct Login {
    username: String,
    password: String
}
#[derive(Serialize, Deserialize,FromForm)]
struct _Upload {
    uploadfile : String,
    token : String,
    #[serde(rename = "imageName")]
    image_name : String
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
    #[serde(rename = "user_list")]
    user_list : Vec<UserStruct>
}

struct RequestDelayStruct {
    _delay : Duration,
    _last : Instant
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
    warn!("RustyWiki Dbg: {}", input.log_text);
    String::from("Ok t")
}

#[post("/jsLog/Debug", data = "<input>")]
fn rocket_route_js_debug(input: Json<LogData>) -> String {
    let in_length = input.log_text.len();
    warn!("RustyWiki Dbg({}): {}", in_length, input.log_text.chars().take(256).collect::<String>());
    String::from("Ok d")
}

#[post("/jsLog/Error", data = "<input>")]
fn rocket_route_js_error(input: Json<LogData>) -> String {
    error!("RustyWiki Err: {}", input.log_text.chars().collect::<String>());
    String::from("Ok")
}

#[post("/jsLog/Exception", data = "<input>")]
fn rocket_route_js_exception(input: Json<LogData>) -> String {
    error!("RustyWiki Exc: {}", input.log_text.chars().collect::<String>());
    String::from("Ok")
}

#[post("/jsLog/<rq>", data = "<input>")]
fn rocket_route_js_log(rq: &RawStr, input: String) -> String {
    info!("RustyWiki Log failed parse: {} {}", rq.as_str(), input);
    String::from("520")
}

// TODO
#[post("/jsUser/UserModify", data = "<input>")]
fn rocket_route_user_modify(input: Json<UserModify>) -> String {
    debug!("user modify {} {} {} {} {}", input.user, input.password, input.new_password, input.new_password_check, input.comment);
    // make sure user is authenticated
    String::from("Ok")
}

// TODO
#[post("/jsUser/Wikisave", data = "<input>")]
fn rocket_route_wiki_save(lock_data : State<PageMap>, input: Json<Wikisave>) -> Status {
    debug!("wiki save {} {} {} {} {} {} {} {} {}", input.page, input.revision, input.previous_revision, input.create_date, input.revision_date, input.revised_by, input.comment, input.lock, input.data);
    if input.revision == "" || input.previous_revision == "" {
        return Status::new(519, "no revision of previous revision");
    }
    if input.page == "" || input.lock == "" {
        return Status::new(519, "no lock or page");
    }
    let mp = lock_data.lock().unwrap();
    if let Some(lock_token) = mp.get(&input.page) {
        if lock_token != &input.lock {
            return Status::new(520, "wrong lock");
        }
    } else {
        return Status::new(521, "wrong lock");
    }
    // TODO
    // make sure directory /wiki/input.page exists
    // open /wiki/input.page/input.revision
    // write data to file
    // close file
    // write to /wiki/input.page/current
    // close file
    Status::Ok
}

// TODO
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

// TODO
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

// TODO
#[post("/jsAdmin/UserDelete", data = "<input>")]
fn rocket_route_user_delete(input: Json<UserDelete>) -> String {
    debug!("user delete {}", input.User);
    String::from("Ok")
}

// TODO
#[post("/jsUser/Upload", data = "<_input>")]
fn rocket_route_user_upload(content_type: &ContentType, _input: Data) -> String {
    debug!("user upload {}", content_type);
    String::from("Ok")
}

// TODO
#[get("/jsAdmin/MasterReset")]
fn rocket_route_master_reset() -> String {
    debug!("master reset");
    String::from("Ok")
}

// TODO
#[get("/page/MediaIndex")]
fn rocket_route_media_index(_user: User, ) -> String {
    debug!("media index");
    String::from("Ok")
}

#[get("/wiki/<page_name>/<version>")]
fn rocket_route_wiki(_user: User, page_name : String, version: Option<String>) -> io::Result<String> {
    let version = version.unwrap_or("current".to_string());
    let path_name = format!("site/wiki/{}/{}", page_name, version);
   fs::read_to_string(path_name)
}

#[get("/page/<page_name>")]
fn rocket_route_page(_user: User, page_name : String) -> Response<'static> {
    let path_name = "site/index.html".to_string();
    let mut response = Response::new();
    response.set_header(Header::new("Content-Type", "text/html"));
    match fs::read_to_string(&path_name) {
        Err(err) => {
            let err = format!("{}", err);
            response.set_status(Status::InternalServerError);
            error!("Could not load {}. Error-{}", path_name, err);
            response.set_sized_body(Cursor::new(err));
        },
        Ok(index_str) => {
            response.set_status(Status::Ok);
            response.set_sized_body(Cursor::new(index_str.replace("DUMMYSTARTPAGE", &page_name)));
        }
    }   
    response
}
/*
#[get("/pagex/foo")]
fn rocket_route_pagex() -> Response<'static> {
    let path_name = "site/index.html".to_string();
    let ids = fs::read_to_string(path_name).unwrap();
    let header = Header::new("Content-Type", "text/html");
    let mut response = Response::new();
    response.set_status(Status::Ok);
    response.set_header(header);
    response.set_sized_body(Cursor::new(ids));
    response
}

*/
#[get("/")]
fn site_root() -> Redirect {
    Redirect::to(uri!(site_top: "index.html"))
}
/*
#[get("/foo/xx")]
fn site_index1(user: User) -> String {
    error!("got authenticated xx-{:?}", user);
    "got xx".to_string()
}

#[get("/foo/xx", rank = 2)]
fn site_index1a(user: Option<User>) -> Response<'static> {
    let mut response = Response::new();
    if user.is_some() {
        return response;
    }
    error!("got user 2={:?}", user);
    let header = Header::new("WWW-Authenticate", "Basic realm=RWIKI");
    response.set_status(Status::Unauthorized);
    response.set_header(header);
    response.set_sized_body(Cursor::new("Unauthorized!"));
    response
}

#[get("/index.html")]
fn site_index2() -> Option<File> {
    do_index()
}

#[get("/favicon.ico")]
fn site_favicon() -> Option<File> {
    let filename = "site/favicon.ico";
    File::open(&filename).ok()
}
*/
#[get("/<file_name>", rank=5)]
fn site_top(_user: User, file_name: String) -> Option<File> {
    if file_name!="index.html" &&
    file_name!="favicon.ico" {
        return None
    }
    let filename = format!("site/{}", file_name);
    File::open(&filename).ok()
}

#[get("/<_pathnames..>", rank = 20)] // high enough to be after the static files which are 10
fn site_nonauth(_pathnames: PathBuf) -> Redirect {
   Redirect::to(uri!(site_top: "login.html"))
//    let mut response = Response::new();
     /*
    if user.is_some() {
        response.set_status(Status::InternalServerError);
        error!("Non auth, has auth user");
        return response;
    }
    error!("got user 2={:?}", user);
    let header = Header::new("WWW-Authenticate", "Basic realm=RWIKI");
    response.set_status(Status::Unauthorized);
    response.set_header(header);
    response.set_sized_body(Cursor::new("Unauthorized!"));
    response
    */
}

#[get("/login.html", rank = 1)]
fn login_user(_user: User) -> Redirect {
    Redirect::to(uri!(site_top: "index.html"))
}

#[get("/login.html", rank = 2)]
fn login_page() -> Option<File> {
    let filename = format!("site/login.html");
    File::open(&filename).ok()
}

#[post("/login", data = "<login>")]
fn login(mut cookies: Cookies<'_>, login: Form<Login>, umap: State<AuthStruct>) -> Result<Redirect, ()> {
    if let Some(_) = auth::login_handle(&login.username, &login.password, &mut cookies, &umap) {
        error!("handled login");
        Ok(Redirect::to(uri!(site_top: "index.html")))
    } else {
        error!("failed login");
        Ok(Redirect::to(uri!(site_top: "login.html")))
    }
}

#[post("/logout", rank = 1)]
fn logout(mut cookies: Cookies<'_>) -> Redirect {
    cookies.remove_private(Cookie::named("wiki_auth"));
    Redirect::to(uri!(site_top: "index.html"))
 }



fn create_rocket() -> rocket::Rocket {
    let auth = match authstruct::load_auth() {
        Ok(a) => a,
        _ => panic!("failed load uinfo")
//        _ => authstruct::gen_auth()
    };
    println!("auth={:?}", auth);
    let delay_map = DelayMap ( Arc::new(Mutex::new(HashMap::new())) );
    let lock_map = PageMap ( Arc::new(Mutex::new(HashMap::new())) );
    rocket::ignite()
    .manage(auth)
    .manage(delay_map)
    .manage(lock_map)
    .mount("/css", StaticFiles::from("site/css"))  // use the site value from config
    .mount("/js", StaticFiles::from("site/js"))  // use the site value from config
    .mount("/media", StaticFiles::from("site/media"))  // use the site value from config
    .mount("/", routes![rocket_route_js_debug_no_trunc, site_root, site_top, site_nonauth,
        login_user, login_page, logout, login,
        rocket_route_js_debug, rocket_route_js_exception, rocket_route_js_error, rocket_route_js_log,
        rocket_route_user_modify, rocket_route_wiki_save, rocket_route_user_lock,
        rocket_route_user_unlock, rocket_route_user_upload, rocket_route_user_delete, rocket_route_master_reset, 
        rocket_route_media_index, rocket_route_page, rocket_route_wiki])
}


fn main() {
//    _testserde();
    let _config = get_command_line();
    println!("got config={:?}", _config);
    create_rocket().launch();
}





//////////////////////////////////////////////////////
/// Unused
/// 


fn _testserde() {
    println!("testserde");
    let u1 = UserStruct {
        user : "u1".to_string(),
        password : "u1p".to_string(), 
        salt : "u1s".to_string(), 
        comment : "u1c".to_string() 
    };
    let u2 = UserStruct {
        user : "u2".to_string(),
        password : "u2p".to_string(), 
        salt : "u2s".to_string(), 
        comment : "u2c".to_string() 
    };
    println!("u1={:?}", u1);
    let mut v1 = Vec::new();
    let mut al = AuthlistStruct { user_list: Vec::new() };
    let serialized = serde_json::to_string(&u1).unwrap();
    println!("serialized = {}", serialized);
    al.user_list.push(u1.clone());
    al.user_list.push(u2.clone());
    v1.push(u1.clone());
    v1.push(u2.clone());
    let s2 = serde_json::to_string(&al).unwrap();
    let v2s = serde_json::to_string(&v1).unwrap();
    let al2 : AuthlistStruct = serde_json::from_str(&s2).unwrap();
    println!("v1 = {:?}", v1);
    println!("v1s = {:?}", v2s);
    println!("s2 = {}", s2);
    println!("al2 = {:?}", al2);
    /*
    let altest = authstruct::load_auth().unwrap();
    altest.lock().unwrap().user_map.insert(u1.User.clone(), u1.clone());
    altest.lock().unwrap().user_map.insert(u2.User.clone(), u2.clone());
    let s3 = serde_json::to_string(&al).unwrap();
    println!("s3 = {:?}", &s3);
    */

    let mut testa1 = Vec::new();
    testa1.push(u1.clone());
    testa1.push(u2.clone());
    let wr = authstruct::Wrapper { user_list: testa1};
    println!("wrapper={:?}", wr);
    let strwr = serde_json::to_string(&wr).unwrap();
    println!("stwrapper={:?}", strwr);
    let recom: authstruct::Wrapper= serde_json::from_str(&strwr).unwrap();
    println!("recom={:?}", recom);
}


fn split_version<'a>(in_str : &'a str) -> Result<(&'a str, &'a str), &'a str> {
    let v: Vec<&str> = in_str.split(DELIMETER).collect();
    match v.len() {
        2 => Ok( (v[0], v[1]) ),
        _ => Err("Bad versioned string")
    }
}

fn _join_version(ver_str : &str, data_str : &str) -> String {
    let mut res = String::with_capacity(ver_str.len()+data_str.len()+11);
    res.push_str(ver_str);
    res.push_str("\n");
    res.push_str(DELIMETER);
    res.push_str("\n");
    res.push_str(data_str);
    res
}
