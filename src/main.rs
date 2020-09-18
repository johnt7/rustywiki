#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate clap;
#[macro_use] extern crate rocket;
#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;


// TODO - allow using basic authentication
//      - add readme, including how to run it, need nightly
//      - russel had to run as cargo run -- -s ../../site
//      - test user api calls
//      - implement file uploads
//      - implement delay
//      - implement https
//      - cleanups
//      - set nocache stuff for non boilerplate
//      - create types for top level rocket data and then impl shortcuts for uses
//      - refactor main
//      - wikisave should put the logged in user's id into version info when saving a page
//      - fixes to index.html
//          update revision data
//      - look at other loggers
//      - add ctrlC handler - https://github.com/Detegr/rust-ctrlc

use std::{
    fs::{self, File},
    io::{self, Cursor},
    path::*
};

use rocket::{
    config::{Config, Environment},
    http::{Status, Header},
//    request::Form,
    Response,
    response::Redirect, 
    State
};

use rocket_prometheus::PrometheusMetrics;


// Macro
#[macro_export]
macro_rules! wrapper {
    ( $newType:ident , $oldType:ty ) => {
        #[derive(Serialize, Deserialize, Debug)]
        pub struct $newType ( $oldType );
        impl Deref for $newType {
            type Target = $oldType;
        
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}



// Modules
#[cfg(test)] mod tests;
mod authstruct;
mod basic;
mod cmdline;
mod config;
mod jsadmin;
mod jsuser;
mod logs;
mod media;
mod pagemap;
mod user;
mod wikifile;

use authstruct::AuthStruct;
use wikifile::WikiStruct;

use user::{User, PageUser};




// Constants
const DATE_FORMAT : &str = "%Y/%m/%d %H:%M:%S%.3f";


static  DIR_NAMES: [&str; 3] = ["css", "js", "media"];




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


/// Any get request to the site that gets here is not authorized.  Does not handle "/"
/// Rank is set high enough to be after all other file handling
#[post("/<_pathnames..>", rank = 20)] 
fn site_post_nonauth(_pathnames: PathBuf) -> Status {
   Status::Unauthorized
}


/// Gets the wiki page requested
#[get("/wiki/<page_name>/<version>")]
fn rocket_route_wiki(_user: User, page_name : String, version: Option<String>) -> io::Result<String> {
    let version = version.unwrap_or("current".to_string());
    let path_name = wikifile::get_path("wiki").join(page_name).join(version);
   fs::read_to_string(path_name)
}

/// Get the index page, with default set to page_name.  Need to replace the dummy with the current page name
#[get("/page/<page_name>")]
fn rocket_route_page(_user: User, page_name : String) -> Response<'static> {
    let path_name = wikifile::get_path("index.html");
    let mut response = Response::new();
    response.set_header(Header::new("Content-Type", "text/html"));
    match fs::read_to_string(&path_name) {
        Err(err) => {
            let err = format!("{}", err);
            response.set_status(Status::InternalServerError);
            response.set_sized_body(Cursor::new(err));
        },
        Ok(index_str) => {
            response.set_status(Status::Ok);
            response.set_sized_body(Cursor::new(index_str.replace("DUMMYSTARTPAGE", &page_name)));
        }
    }   
    response
}

/// redirect from / to /index.html
#[get("/")]
fn site_root() -> Redirect {
    Redirect::to(uri!(site_top: "index.html"))
}


/// get one of the top level files
#[get("/<file_name>", rank=5)]
fn site_top(_user: PageUser, file_name: String) -> Option<File> {
    if file_name!="index.html" &&
        file_name!="favicon.ico" {
        return None
    }
    File::open(&wikifile::get_path(&file_name)).ok()
}

/// get a static file
#[get("/<path_name>/<file_name>", rank=5)]
fn site_files(_user: PageUser, path_name: String, file_name: String) -> Option<File> {
    error!("path={}", path_name);
    if !DIR_NAMES.contains(&&path_name[..]) {
        error!("not in dirname");
        return None;
    }
    File::open(&wikifile::get_path(&format!("{}/{}", &path_name, &file_name))).ok()
}

/// get a media file
#[get("/media/<date_dir>/<file_name>", rank=5)]
fn media_files(_user: PageUser, date_dir: String, file_name: String) -> Option<File> {
    File::open(&wikifile::get_path(&format!("media/{}/{}", &date_dir, &file_name))).ok()
}

/// any get request to the site (does not include /) that get here is not authorized
/// TODO, only needed for login page
#[get("/x/<_pathnames..>", rank = 20)] // rank high enough to be after the static files which are 10
fn site_get_nonauth_login(_pathnames: PathBuf) -> Redirect {
   Redirect::to(uri!(site_top: "login.html"))
}

#[get("/<_pathnames..>", rank = 20)] // high enough to be after the static files which are 10
fn site_get_nonauth(user: Option<User>, _pathnames: PathBuf) -> Response<'static> {
    let mut response = Response::new();
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
}
/*
/// already logged in, redirect to /index.html
#[get("/login.html", rank = 1)]
fn login_user(_user: PageUser) -> Redirect {
    Redirect::to(uri!(site_top: "index.html"))
}
/// return login page
#[get("/login.html", rank = 2)]
fn login_page() -> Option<File> {
    File::open(&wikifile::get_path("login.html")).ok()
}

/// Post from the login page, try to set auth cookie
#[post("/login", data = "<login>")]
fn login(mut cookies: Cookies<'_>, login: Form<Login>, umap: State<WikiStruct<AuthStruct>>, cfg: State<config::WikiConfig>) -> Result<Redirect, ()> {
    if let Some(_) = authstruct::login_handle(login, &mut cookies, &umap, &cfg) {
        Ok(Redirect::to(uri!(site_top: "index.html")))
    } else {
        Ok(Redirect::to(uri!(site_top: "login.html")))
    }
}

/// got logout request, forget cookie
#[post("/logout", rank = 1)]
fn logout(mut cookies: Cookies<'_>) -> Redirect {
    cookies.remove_private(Cookie::named("wiki_auth"));
    Redirect::to(uri!(site_top: "index.html"))
 }
*/

/// create the Rocket instance.  Having it separate allows easier testing.
fn create_rocket(cmd_cfg: cmdline::ConfigInfo) -> rocket::Rocket {
    // TODO - use info in cmd_cfg
    //      - write to log file or to console
    //      - certificate location

    // tell Rocket to use the specified port
    let config = Config::build(Environment::Staging)
    .address("localhost")
    .port(cmd_cfg.port)
    .finalize().unwrap();

    // set the centralized file system roo
    wikifile::set_path(cmd_cfg.site);

    // create the objects needed for running the wiki
    let auth =  authstruct::load_auth().unwrap();
    let cfg = config::load_config().unwrap();
    let mi = media::MediaIndex::new();
    let delay_map = jsadmin::DelayMap::new();
    let lock_map = pagemap::PageMap::new(); 
    let prometheus = PrometheusMetrics::new();

    rocket::custom(config)
    .manage(auth)
    .manage(cfg)
    .manage(delay_map)
    .manage(lock_map)
    .manage(mi)
    .attach(prometheus.clone())
    .mount("/metrics", prometheus)
    .mount("/", routes![logs::rocket_route_js_debug_no_trunc, site_root, site_top,
        // login_user, login_page, logout, login, - login version
        site_files, media_files, site_get_nonauth, site_post_nonauth,

        logs::rocket_route_js_debug, logs::rocket_route_js_exception, logs::rocket_route_js_error, logs::rocket_route_js_log,

        jsuser::rocket_route_user_modify, jsuser::rocket_route_wiki_save, jsuser::rocket_route_user_lock,
        jsuser::rocket_route_user_unlock, jsuser::rocket_route_user_upload,

        jsadmin::rocket_route_user_delete, jsadmin::rocket_route_master_reset, 
        
        media::rocket_route_media_index, rocket_route_page, rocket_route_wiki])
}


fn main() {
    let config = cmdline::get_command_line();
    create_rocket(config).launch();
}

/*
Status
	unauth
io::Result<String>
	
Response<'static>
Redirect
Option<File>
String
Result<String, Box<dyn error::Error>> 
*/