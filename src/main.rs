#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate clap;
#[macro_use] extern crate rocket;
#[macro_use] extern crate log;
extern crate simplelog;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::serve::StaticFiles;
use rocket_contrib::json::Json;
use simplelog::{SimpleLogger, LevelFilter, Config};
//use rocket::request::Form;

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
struct LogData {
    LogText: String,
}

#[derive(Deserialize)]
struct UserModify {
 User: String,
 Password: String,
 NewPassword: String,
 NewPasswordCheck: String,
 Comment: String
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
struct Wikilock {
 Page : String,
 Lock : String
}

#[derive(FromForm)]
struct Upload {
    uploadfile : String,
    token : String,
    imageName : String
}

#[derive(Deserialize)]
struct UserDelete {
 User : String
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
/*
#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}
*/
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
fn rocket_route_user_lock(input: Json<Wikilock>) -> String {
    debug!("user lock {} {}", input.Lock, input.Page);
    String::from("Ok")
}
#[post("/jsUser/Wikiunlock", data = "<input>")]
fn rocket_route_user_unlock(input: Json<Wikilock>) -> String {
    debug!("user unlock {} {}", input.Lock, input.Page);
    String::from("Ok")
}#[post("/jsAdmin/UserDelete", data = "<input>")]
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
/*
#[post("/jsLog/Debug", data = "<input>", rank=2)]
fn jsDebugs(input: String) -> String {
    format!("logged data s={}.", input)
}
*/
/*
#[get("/site/<ftype>/<filename>")]
fn stsite(ftype: String, filename: String) -> String {
    StaticFiles::from("site")
}
*/
fn main() {
    let _config = get_command_line();
    let _ = SimpleLogger::init(LevelFilter::Warn, Config::default());    
    rocket::ignite()
    .mount("/", StaticFiles::from("site"))
    .mount("/", routes![rocket_route_js_debug_no_trunc, 
        rocket_route_js_debug, rocket_route_js_exception, rocket_route_js_log,
        rocket_route_user_modify, rocket_route_wiki_save, rocket_route_user_lock,
        rocket_route_user_unlock, rocket_route_user_delete, rocket_route_master_reset, 
        rocket_route_media_index, rocket_route_page])
    .launch();
}
