use rocket::{
    http::{
        RawStr,
        Status
    },
    request::{FromRequest, Outcome, Request},
    State
};
use rocket_contrib::json::Json;
use std::str::FromStr;
use super::{
    config, 
    user
};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LogData {
    pub log_text: String,
}

/// Used to represent if the user is allowed to write to logs
pub struct LogUser;
impl<'a, 'r> FromRequest<'a, 'r> for LogUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<LogUser, Self::Error> {
        let res = request.cookies().get_private("wiki_auth")
        .and_then(|cookie| {
            user::User::from_str(cookie.value()).ok()
        });
        match request.guard::<State<config::WikiConfig>>() {
            Outcome::Success(cfg) => {
                if !cfg.0.read().unwrap().data.authentication_required_for_logging {
                    return Outcome::Success(LogUser);
                }
            },
            _ => {}
        };
        match res {
            Some(_) => Outcome::Success(LogUser),
            None => Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}

#[post("/jsLog/DebugNoTrunc", data = "<input>", rank=1)]
pub fn rocket_route_js_debug_no_trunc(_log_user: LogUser, input: Json<LogData>) -> String {
    warn!("RustyWiki Dbg: {}", input.log_text);
    String::from("Ok t")
}

#[post("/jsLog/Debug", data = "<input>")]
pub fn rocket_route_js_debug(_log_user: LogUser, input: Json<LogData>) -> String {
    let in_length = input.log_text.len();
    warn!("RustyWiki Dbg({}): {}", in_length, input.log_text.chars().take(256).collect::<String>());
    String::from("Ok d")
}

#[post("/jsLog/Error", data = "<input>")]
pub fn rocket_route_js_error(_log_user: LogUser, input: Json<LogData>) -> String {
    error!("RustyWiki Err: {}", input.log_text.chars().collect::<String>());
    String::from("Ok")
}

#[post("/jsLog/Exception", data = "<input>")]
pub fn rocket_route_js_exception(_log_user: LogUser, input: Json<LogData>) -> String {
    error!("RustyWiki Exc: {}", input.log_text.chars().collect::<String>());
    String::from("Ok")
}

#[post("/jsLog/<rq>", data = "<input>")]
pub fn rocket_route_js_log(_log_user: LogUser, rq: &RawStr, input: String) -> String {
    info!("RustyWiki Log failed parse: {} {}", rq.as_str(), input);
    String::from("520")
}
