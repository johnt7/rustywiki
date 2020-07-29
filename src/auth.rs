use std::{fmt, str::FromStr};
use rocket::{
    http::{Cookie, hyper::header},
    request::{self, FromRequest, Request},
    Outcome,
    outcome::IntoOutcome
};
use super::basic;

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

impl AuthState {
    pub fn jt_from_str(input: &str) -> AuthState {
        match input {
            "AuthNotAuth"  => AuthState::AuthNotAuth,
            "AuthUser"  => AuthState::AuthUser,
            "AuthAdmin"  => AuthState::AuthAdmin,
            _      => AuthState::AuthNotAuth
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

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
            let b = basic::BasicAuthRaw::from_request(request);
            error!("got cred={:?}", b);
            if let Outcome::Success(cred) = b {
                error!("got cred={:?}, {}", cred.username, cred.password);
                if cred.username == "root" && cred.password=="adm" {
                    return Outcome::Success(User {auth: AuthState::AuthNotAuth, name: cred.username});
                } else if cred.username == "user" && cred.password=="user" {
                    return Outcome::Success(User {auth: AuthState::AuthUser, name: cred.username});
                }

            };
            Outcome::Forward(())
//            Outcome::failure(Status::BadRequest)
        }
}