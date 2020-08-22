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
/// What kind of authorization is this?
#[derive(Debug, PartialEq)]
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
/// Structure that is used to represent a logged in user
pub struct User {
    pub auth: AuthState,
    pub name: String
}
impl User {
    pub fn new_unauth() -> User {
        User{auth: AuthState::AuthNotAuth, name: "".to_string()}
    }
    pub fn new_user(uname: &str, admin: bool) -> User {
        User {
            auth:  if admin { AuthState::AuthAdmin } else { AuthState::AuthUser },
            name: uname.to_string()
        }
    }
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
            None => {
                error!("forwarding from user");
                Outcome::Forward(())
            }
        }
    }
}

pub struct PageAdmin(User);
impl<'a, 'r> FromRequest<'a, 'r> for PageAdmin {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<PageAdmin, Self::Error> {
        let logged_in = request.guard::<User>(); 
        match logged_in {
            Outcome::Success(user) => {
                if user.auth == AuthState::AuthAdmin {
                    return Outcome::Success(PageAdmin(user));
                }
            },
            _ => {}
        };
        Outcome::Forward(())
    }
}

#[derive(Debug)]
/// Represents permission for a user to access the current page.
pub struct PageUser(User);
impl<'a, 'r> FromRequest<'a, 'r> for PageUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<PageUser, Self::Error> {
        // see if the user is logged in
        let logged_in = request.guard::<User>(); 
        error!("logged in={}", logged_in);
        // find out if this page is an admin page
        let is_admin = request.guard::<IsAdminPage>()?.0;
        error!("is admin={:?}", is_admin);
        // this is an admin page
        if is_admin {
            error!("got admin=");
            return match logged_in {
                Outcome::Success(u) => {
                    error!("admin have user");
                    if u.auth == AuthState::AuthAdmin {
                        // user is logged in as admin, so allow
                        error!("admin is admin user");
                        Outcome::Success(PageUser(u))
                    } else {
                        // logged in but not admin
                        error!("admin not admin user");
                        Outcome::Failure((Status::Unauthorized, ()))
                    }
                },
                _ => { 
                    // not logged in
                    error!("admin not logged in");
                    Outcome::Failure((Status::Unauthorized, ()))
                }
            }
        }
 
        error!("NOTadmin");
        // not admin, if don't need auth for read, then go ahead, otherwise ok if logged in.
        if let Outcome::Success(cfg) = request.guard::<State<wikifile::WikiStruct<config::ConfigurationStruct>>>() {
            error!("NOTadmin got cfg");
            if !cfg.0.read().unwrap().data.authentication_required_for_read {
                error!("NOTadmin not req");
                return Outcome::Success(PageUser(
                    // return the logged in user, or non-auth
                    match logged_in {
                        Outcome::Success(u) => u,
                        _ => User::new_unauth()
                    }
                ));
            }
            error!("NOTadmin need re1");
            if let Outcome::Success(u) = logged_in {
                error!("NOTadmin logged in");
                return Outcome::Success(PageUser(u));
            }
        };
        error!("NOTadmin failed");
        Outcome::Failure((Status::Unauthorized, ()))
    }
}


/// is the current page an admin page?
pub struct IsAdminPage(bool);
impl<'a, 'r> FromRequest<'a, 'r> for IsAdminPage {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<IsAdminPage, Self::Error> {
        match request.guard::<State<wikifile::WikiStruct<config::ConfigurationStruct>>>() {
            Outcome::Success(cfg) => {
                let path = request.uri().path();
                let adm_list = &cfg.read().unwrap().data.admin_pages;
                Outcome::Success(IsAdminPage(adm_list.iter().any(|e| {
                    error!("e={} path={}", e, path);
                    path.starts_with(e)//.starts_with(path)
                })))
            },
            _ => {
                Outcome::Failure((Status::Unauthorized, ())) 
            }
        }
    }  
}
