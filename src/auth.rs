use std::{fmt, str::FromStr};
use rocket::{
    http::{Cookies, Cookie},
    Outcome,
    request::{self, FromRequest, Request},
    State
 };
use super::authstruct::AuthStruct;

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

pub fn login_handle(uname: &str, pwd: &str, cookies: &mut Cookies<'_>, umap: &AuthStruct) -> Option<User> {
    let thing = &umap.lock().unwrap().user_map;
    // TODO handle no auth case
    let entry = thing.get(uname)?;
    if entry.Password != pwd { return None };
    let u_tok = User{auth: AuthState::AuthAdmin, name: uname.to_string()}; // TODO get auth from list of admin
    cookies.add_private(Cookie::new("wiki_auth", u_tok.to_string()));
    Some(u_tok)
}
