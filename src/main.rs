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
mod authstruct;
mod basic;
mod config;
mod user;
mod wikifile;

use authstruct::AuthStruct;
use user::User;



// Constants
const DATE_FORMAT : &str = "%Y/%m/%d %H:%M:%S%.3f";



// types

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LogData {
    log_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct UserModify {
    user: String,
    password: String,
    new_password: String,
    new_password_check: String,
    comment: String
}

// also used in unlock
#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Wikilock {
 page : String,
 lock : String
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
#[serde(rename_all = "PascalCase")]
struct UserDelete {
 user : String
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
fn rocket_route_wiki_save(lock_data : State<PageMap>, input: Json<wikifile::PageRevisionStruct>) -> Status {
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
    if input.page == "" || input.lock == "" {
        return Status::new(519, "no lock page");
    }
    let mut mp = lock_data.lock().unwrap();
    if let Some(_) = mp.get(&input.page) {
        return Status::new(520, "already locked");
    }
    let res = mp.insert(input.page.clone(), input.lock.clone());
    let ct = mp.len();
    info!("user lock len = {} res={:?}", ct, res);
    Status::Ok
}

// TODO
#[post("/jsUser/Wikiunlock", data = "<input>")]
fn rocket_route_user_unlock(lock_data : State<PageMap>, input: Json<Wikilock>) -> Option<String> {
     if input.page == "" {
        return None;
    }
    let mut mp = lock_data.lock().unwrap();
    if let Some(ll) = mp.get(&input.page) {
        if ll != &input.lock {
            return None;
        }
    } else {
        return None;
    }
    let res = mp.remove(&input.page);

    let ct = mp.len();
    info!("user lock {} {} = len={} res={:?}", input.lock, input.page, ct, res);
    Some(String::from("Ok"))
}

// TODO
#[post("/jsAdmin/UserDelete", data = "<input>")]
fn rocket_route_user_delete(input: Json<UserDelete>) -> String {
    debug!("user delete {}", input.user);
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

#[get("/")]
fn site_root() -> Redirect {
    Redirect::to(uri!(site_top: "index.html"))
}

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

    if let Some(_) = authstruct::login_handle(&login.username, &login.password, &mut cookies, &umap) {
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
    let auth =  authstruct::load_auth().unwrap();
//    println!("auth={:?}", auth);
    let cfg = config::load_config().unwrap();
//    println!("cfg={:?}", cfg);

    let delay_map = DelayMap ( Arc::new(Mutex::new(HashMap::new())) );
    let lock_map = PageMap ( Arc::new(Mutex::new(HashMap::new())) );

    rocket::ignite()
    .manage(auth)
    .manage(cfg)
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
   let _config = get_command_line();
//    println!("got config={:?}", _config);
    create_rocket().launch();
}






/// Command line configuration information
#[derive(Debug)]
pub struct ConfigInfo {
    pub log_file_name : String,
    pub site : String,
    pub console_log : bool,
    pub cert : String,
    pub key : String,
    pub port : u16
}

/// Loads information from the command line
pub fn get_command_line() -> ConfigInfo {
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
