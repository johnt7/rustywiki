use std::{
    fmt,
    str::FromStr,
};
use rocket::{
    http::Status,
    Outcome,
    request::{self, FromRequest, Request},
    State
};
use super::{
    config,
    wikifile
};
#[derive(Debug)]
pub enum AuthState {
    AuthNotAuth,
    AuthUser,
    AuthAdmin
}
impl FromStr for AuthState {
    type Err = ();
    fn from_str(input: &str) -> Result<AuthState, Self::Err> {
        match input {
            "AuthNotAuth"  => Ok(AuthState::AuthNotAuth),
            "AuthUser"  => Ok(AuthState::AuthUser),
            "AuthAdmin"  => Ok(AuthState::AuthAdmin),
            _      => Ok(AuthState::AuthNotAuth),
        }
    }
}
impl fmt::Display for AuthState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            AuthState::AuthNotAuth => "AuthNotAuth",
            AuthState::AuthUser => "AuthUser",
            AuthState::AuthAdmin => "AuthAdmin",
        })
    }
}

#[derive(Debug)]
pub struct User {
    pub auth: AuthState,
    pub name: String
}
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.auth)
    }
}
impl FromStr for User {
    type Err = ();
    fn from_str(input: &str) -> Result<User, Self::Err> {
        let mut in_str = input.split(":");

        let name = in_str.next().ok_or(())?;
        let auth_st = in_str.next().ok_or(())?;
        Ok(User{auth: AuthState::from_str(auth_st)?, name: name.to_string()})
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, Self::Error> {
        let res = request.cookies().get_private("wiki_auth")
        .and_then(|cookie| {
            User::from_str(cookie.value()).ok()
        });
        match res {
            Some(user) => Outcome::Success(user),
            None => Outcome::Forward(())
        }
    }
}

pub struct LogUser;
impl<'a, 'r> FromRequest<'a, 'r> for LogUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<LogUser, Self::Error> {
        let res = request.cookies().get_private("wiki_auth")
        .and_then(|cookie| {
            User::from_str(cookie.value()).ok()
        });
        match request.guard::<State<wikifile::WikiStruct<config::ConfigurationStruct>>>() {
            Outcome::Success(cfg) => {
                error!("log?={}", cfg.0.read().unwrap().data.authentication_required_for_logging);
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