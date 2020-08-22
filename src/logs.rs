use rocket::{
    http::{
        RawStr,
        Status
    },
    request::{FromRequest, Outcome, Request},
    State
};
use rocket_contrib::json::Json;
use super::{
    config, 
    user
};

const TRUNC_DEBUG_LEN: usize = 256;
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
/// Structure used for log request bodies
pub struct LogData {
    pub log_text: String,
}

/// Structure used to represent whether the user is allowed to write to logs
/// Either unauthenticated logging is allowed, or they have to be logged in
pub struct LogUser(user::User);

impl<'a, 'r> FromRequest<'a, 'r> for LogUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<LogUser, Self::Error> {
        let logged_in = request.guard::<user::User>(); 
        let need_auth = match request.guard::<State<config::WikiConfig>>() {
            Outcome::Success(cfg) => {
                cfg.0.read().unwrap().data.authentication_required_for_logging
            },
            _ => true
        };
        match logged_in {
            Outcome::Success(u) => 
                Outcome::Success(LogUser(u)),
            _ => if need_auth {
                Outcome::Failure((Status::Unauthorized, ()))
             } else {
                Outcome::Success(LogUser(user::User::new_unauth()))
            }
        }
    }
}

#[post("/jsLog/DebugNoTrunc", data = "<input>", rank=1)]
/// Logs debug output from wiki page.  Does not truncate the message
pub fn rocket_route_js_debug_no_trunc(_log_user: LogUser, input: Json<LogData>) -> String {
    warn!("RustyWiki Dbg: {}", input.log_text);
    String::from("Ok")
}

#[post("/jsLog/Debug", data = "<input>")]
/// Logs debug output from wiki page.  Truancates to TRUNC_DEBUG_LEN (256) charaters.
pub fn rocket_route_js_debug(_log_user: LogUser, input: Json<LogData>) -> String {
    let in_length = input.log_text.len();
    warn!("RustyWiki Dbg({}): {}", in_length, input.log_text.chars().take(TRUNC_DEBUG_LEN).collect::<String>());
    String::from("Ok")
}

#[post("/jsLog/Error", data = "<input>")]
/// log an error from wiki page
pub fn rocket_route_js_error(_log_user: LogUser, input: Json<LogData>) -> String {
    error!("RustyWiki Err: {}", input.log_text.chars().collect::<String>());
    String::from("Ok")
}

#[post("/jsLog/Exception", data = "<input>")]
/// Logs an exception from wiki page
pub fn rocket_route_js_exception(_log_user: LogUser, input: Json<LogData>) -> String {
    error!("RustyWiki Exc: {}", input.log_text.chars().collect::<String>());
    String::from("Ok")
}

#[post("/jsLog/<rq>", data = "<input>")]
/// Fallback if any of the log attempts didn't parse.
pub fn rocket_route_js_log(_log_user: LogUser, rq: &RawStr, input: String) -> String {
    info!("RustyWiki Log failed parse: {} {}", rq.as_str(), input);
    String::from("520")
}

#[post("/jsLog/<rq>", data = "<input>")]
/// Fallback if any of the log attempts aren't authorized.
pub fn rocket_nonauth_js_log(rq: &RawStr, input: String) -> String {
    info!("RustyWiki Log failed parse: {} {}", rq.as_str(), input);
    String::from("Unauthroized")
}