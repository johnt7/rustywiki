#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate clap;
#[macro_use] extern crate rocket;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;

// TODO - refactor main
//      - finish applying PageUser and then test user auth
//      - user api calls
//      - cleanups
//      - wikisave should put user into version info
//      - fixes to index.html
//          update revision data
//      - look at other loggers


use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Cursor},
    ops::Deref,
    path::*,
    sync::{RwLock},
    time::{Duration, Instant},
};

use rocket::{
    config::{Config, Environment},
    Data,
    http::{ContentType, Cookie, Cookies, Status, Header},
    request::Form,
    Response,
    response::Redirect, 
    State
};

use rocket_contrib::{
    serve::StaticFiles,
    json::Json
};
use rocket_prometheus::PrometheusMetrics;

// Modules
#[cfg(test)] mod tests;
mod authstruct;
mod cmdline;
mod basic;
mod config;
mod logs;
mod media;
mod user;
mod wikifile;

use authstruct::AuthStruct;
use wikifile::WikiStruct;

use user::{User, PageUser};



// Constants
const DATE_FORMAT : &str = "%Y/%m/%d %H:%M:%S%.3f";



// types

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
pub struct Login {
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

/// Structure passed to Rocket to allow storage of delay  information
struct DelayMap ( RwLock<HashMap<String, RequestDelayStruct>> );
impl Deref for DelayMap {
    type Target = RwLock<HashMap<String, RequestDelayStruct>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DelayMap {
    pub fn new() -> DelayMap {
        DelayMap ( RwLock::new(HashMap::new()) )
    }
}

/// Structure passed to Rocket to store page locks
struct PageMap ( RwLock<HashMap<String, String>> );
impl Deref for PageMap {
    type Target = RwLock<HashMap<String, String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl PageMap {
    pub fn new() -> PageMap {
        PageMap ( RwLock::new(HashMap::new()) )
    }
}


// TODO
#[post("/jsUser/UserModify", data = "<input>")]
fn rocket_route_user_modify(input: Json<UserModify>) -> String {
    debug!("user modify {} {} {} {} {}", input.user, input.password, input.new_password, input.new_password_check, input.comment);
    // make sure user is authenticated
    String::from("Ok")
}

// TODO - clean up messages
#[post("/jsUser/Wikisave", data = "<input>")]
fn rocket_route_wiki_save(_user: User, lock_data : State<PageMap>, input: Json<wikifile::PageRevisionStruct>) -> Status {
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
        return Status::new(521, "wrong lock");
    }
    match wikifile::write_parts(&input, &input.data) {
        Ok(_) => Status::Ok,
        _ => Status::new(522, "failed to write")
    }
}

#[post("/jsUser/Wikilock", data = "<input>")]
fn rocket_route_user_lock(_user: User, lock_data : State<PageMap>, input: Json<Wikilock>) -> Status {
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
fn rocket_route_user_unlock(_user: User, lock_data : State<PageMap>, input: Json<Wikilock>) -> Option<String> {
     if input.page == "" {
        return None;
    }
    let mut mp = lock_data.write().unwrap();
    // find if there is a lock and if so make sure the lock keys match
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

// TODO - cleanup output
/// do master reset of the system
#[get("/jsAdmin/MasterReset")]
fn rocket_route_master_reset(_user: User, delay_map: State<DelayMap>, page_locks: State<PageMap>, auth: State<WikiStruct<AuthStruct>>, cfg: State<config::WikiConfig>, mi: State<media::MediaIndex>) -> String {
    *delay_map.write().unwrap() = HashMap::new();
    *page_locks.write().unwrap() = HashMap::new();
    *auth.write().unwrap() =  authstruct::load_auth_int().unwrap();
    *cfg.write().unwrap() =  config::load_config_int().unwrap();
    error!("master reset 5");
    *mi.write().unwrap() = media::media_str();
    error!("master reset 5");
    String::from("Ok")
}

/// Gets the wiki page requested
#[get("/wiki/<page_name>/<version>")]
fn rocket_route_wiki(_user: User, page_name : String, version: Option<String>) -> io::Result<String> {
    let version = version.unwrap_or("current".to_string());
//    let path_name = format!("site/wiki/{}/{}", page_name, version);
    let path_name = wikifile::get_path("wiki").join(page_name).join(version);
   fs::read_to_string(path_name)
}

/// Get the index page, with default set to page_name
#[get("/page/<page_name>")]
fn rocket_route_page(_user: User, page_name : String) -> Response<'static> {
//    let path_name = "site/index.html".to_string();
    let path_name = wikifile::get_path("index.html");
    let mut response = Response::new();
    response.set_header(Header::new("Content-Type", "text/html"));
    match fs::read_to_string(&path_name) {
        Err(err) => {
            let err = format!("{}", err);
            response.set_status(Status::InternalServerError);
            error!("Could not load {:?}. Error-{}", path_name, err);
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
/// redirect from / to /index.html
fn site_root() -> Redirect {
    Redirect::to(uri!(site_top: "index.html"))
}


#[get("/<file_name>", rank=5)]
/// get one of the top level files
fn site_top(_user: User, file_name: String) -> Option<File> {
    if file_name!="index.html" &&
        file_name!="favicon.ico" {
        return None
    }
//   let filename = format!("site/{}", file_name);
//    let filename = wikifile::get_path(&file_name);
//    File::open(&filename).ok()
    File::open(&wikifile::get_path(&file_name)).ok()
}

#[get("/<_pathnames..>", rank = 20)] // rank high enough to be after the static files which are 10
/// any get request to the site (does not include /) that get here is not authorized
fn site_nonauth(_pathnames: PathBuf) -> Redirect {
   Redirect::to(uri!(site_top: "login.html"))
}

#[get("/login.html", rank = 1)]
/// already logged in, redirect to /index.html
fn login_user(_user: User) -> Redirect {
    Redirect::to(uri!(site_top: "index.html"))
}

#[get("/login.html", rank = 2)]
/// return login page
fn login_page() -> Option<File> {
//    let filename = format!("site/login.html");
//    File::open(&filename).ok()
    File::open(&wikifile::get_path("login.html")).ok()
}

#[post("/login", data = "<login>")]
/// Post from the login page, try to set auth cookie
fn login(mut cookies: Cookies<'_>, login: Form<Login>, umap: State<WikiStruct<AuthStruct>>, cfg: State<config::WikiConfig>) -> Result<Redirect, ()> {

    if let Some(_) = authstruct::login_handle(login, &mut cookies, &umap, &cfg) {
        error!("handled login");
        Ok(Redirect::to(uri!(site_top: "index.html")))
    } else {
        error!("failed login");
        Ok(Redirect::to(uri!(site_top: "login.html")))
    }
}

#[post("/logout", rank = 1)]
/// got logout request, forget cookie
fn logout(mut cookies: Cookies<'_>) -> Redirect {
    cookies.remove_private(Cookie::named("wiki_auth"));
    Redirect::to(uri!(site_top: "index.html"))
 }


/// create the Rocket instance.  Having it separate allows easier testing.
fn create_rocket(cmd_cfg: cmdline::ConfigInfo) -> rocket::Rocket {
    // TODO - use info in cmd_cfg
    let config = Config::build(Environment::Staging)
    .address("localhost")
    .port(cmd_cfg.port)
    .finalize().unwrap();

    println!("set path to={:?}", cmd_cfg);
    wikifile::set_path(cmd_cfg.site);
    println!("done.....");
    let path = wikifile::get_path("foo");
    println!("path={:?} last={:?}", path, path.file_name());

    let auth =  authstruct::load_auth().unwrap();
    println!("suth={:?}", auth.read());
    let cfg = config::load_config().unwrap();
    let mi = media::MediaIndex::new();

    let delay_map = DelayMap::new();
    let lock_map = PageMap::new(); 
    let prometheus = PrometheusMetrics::new();

//    rocket::ignite()
    rocket::custom(config)
    .manage(auth)
    .manage(cfg)
    .manage(delay_map)
    .manage(lock_map)
    .manage(mi)
    .attach(prometheus.clone())
    .mount("/metrics", prometheus)
    .mount("/css", StaticFiles::from(wikifile::get_path("css")))  // TODO, secure? use the site value from config
    .mount("/js", StaticFiles::from(wikifile::get_path("js")))  // use the site value from config
    .mount("/media", StaticFiles::from(wikifile::get_path("media")))  // use the site value from config
    .mount("/", routes![logs::rocket_route_js_debug_no_trunc, site_root, site_top, site_nonauth,
        login_user, login_page, logout, login,
        logs::rocket_route_js_debug, logs::rocket_route_js_exception, logs::rocket_route_js_error, logs::rocket_route_js_log,
        rocket_route_user_modify, rocket_route_wiki_save, rocket_route_user_lock,
        rocket_route_user_unlock, rocket_route_user_upload, rocket_route_user_delete, rocket_route_master_reset, 
        media::rocket_route_media_index, rocket_route_page, rocket_route_wiki])
}


fn main() {
    let config = cmdline::get_command_line();
    create_rocket(config).launch();
}
