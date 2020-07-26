use std::{fmt, str::FromStr};
use rocket::{
    request::{self, FromRequest, Request},
    outcome::IntoOutcome
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
        error!("rq={:?}", request);
        request.cookies()
            .get_private("user_id")
            .and_then(|cookie| {
                error!("ck={:?}", cookie);
                let mut vals = cookie.value().split("::");
                let auth = vals.next().unwrap_or("AuthNotAuth");
                let auth: AuthState = AuthState::jt_from_str(auth);
//                let auth: AuthState = AuthState::AuthNotAuth;
                let name = vals.next().unwrap_or("").to_string();
                Some(User {auth, name})
             })//.or_else(|| Some(User {auth: AuthState::AuthNotAuth, name: "nobody".to_string()}) )
             /*
            .map(|val : &str| {
                let mut vals = val.split("::");
//                let auth: AuthState = vals.next().unwrap_or("AuthNotAuth").from_string().unwrap_or(AuthState::AuthNotAuth);
                let auth: AuthState = AuthState::AuthNotAuth;
                let name = vals.next().unwrap_or("default").to_string();
                User {auth, name}
            })
            */
            .or_forward(())
    }
}