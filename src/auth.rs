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
    type Error = std::convert::Infallible;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, Self::Error> {
        if let Some(ai) = request.headers().get_one("Authorization") {
//            let auth_info: header::Basic = header::Basic::from_str(ai);
            error!("basic={:?}", ai);
//            let foo = header::Basic::from_str(ai);
            let b = basic::BasicAuthRaw::from_request(request);
            error!("got cred={:?}", b);
            if let Outcome::Success(cred) = b {
                error!("got cred={:?}, {}", cred.username, cred.password);
                if cred.username == "root" {
                    request.cookies()
                    .add_private(Cookie::new("wiki_auth", cred.username))
                }
            }
        };
        request.cookies()
            .get_private("wiki_auth")
            .and_then(|cookie| {
                error!("ck={:?}", cookie);
//                let mut vals = cookie.value().split("::");
//                let auth = vals.next().unwrap_or("AuthNotAuth");
                let auth = AuthState::AuthAdmin;
//                let auth: AuthState = AuthState::jt_from_str(auth);
//                let name = vals.next().unwrap_or("").to_string();
                let name = cookie.value().to_string();
                Some(User {auth, name})
             })
            .or_forward(())
    }
}