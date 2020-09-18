use std::{
    fmt,
    ops::Deref,
    str::FromStr,
};
use rocket::{
    http::Status,
    Outcome,
    request::{self, FromRequest, Request},
    State
};
use super::{
    authstruct,
    basic,
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
/* used for login version
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
                Outcome::Forward(())
            }
        }
    }
}
*/
impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
            let b = basic::BasicAuthRaw::from_request(request)?;
            let auth =  request.guard::<State<wikifile::WikiStruct<authstruct::AuthStruct>>>()?;
            let auth_hash = &auth.read().unwrap().data;
            let hentry = auth_hash.get(&b.username);
            if let Some(uc) = hentry {
                if b.password == uc.password {
                    return Outcome::Success(User {
                        auth: AuthState::AuthUser,
                        name: uc.user.clone()
                    })
                };
                /*
                login page handling
                error!("got cred={:?}, {}", cred.username, cred.password);
                if cred.username == "root" && cred.password=="adm" {
                    return Outcome::Success(User {auth: AuthState::AuthNotAuth, name: cred.username});
                } else if cred.username == "user" && cred.password=="user" {
                    return Outcome::Success(User {auth: AuthState::AuthUser, name: cred.username});
                }
                */

            }
            Outcome::Forward(())
        }
}

/// User has to be logged in as an admin
pub struct PageAdmin(pub User);
impl Deref for PageAdmin {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for PageAdmin {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<PageAdmin, Self::Error> {
        let logged_in = request.guard::<User>(); 
        match logged_in {
            Outcome::Success(user) 
                if user.auth == AuthState::AuthAdmin => {
                Outcome::Success(PageAdmin(user))
            },
            _ => Outcome::Forward(())
        }
    }
}

#[derive(Debug)]
/// Represents permission for a user to read the current page.  They may be logged in or site may be set to allow read without logging in
pub struct PageUser(pub User);
impl Deref for PageUser {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for PageUser {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<PageUser, Self::Error> {
        let logged_in = request.guard::<User>(); 
        let is_admin_page = request.guard::<IsAdminPage>()?.0;
        if is_admin_page {
            return match logged_in {
                Outcome::Success(u)
                    if u.auth == AuthState::AuthAdmin  => {
                        // user is logged in as admin, so allow
                         Outcome::Success(PageUser(u))
                },
                _ => { 
                    // not logged in, or not admin
                    Outcome::Forward(())
                }
            }
        }
 

        let need_auth = match request.guard::<State<config::WikiConfig>>() {
            Outcome::Success(cfg) => {
                cfg.0.read().unwrap().data.authentication_required_for_read
            },
            _ => true
        };
        match logged_in {
            Outcome::Success(u) => 
                Outcome::Success(PageUser(u)),
            _ => if need_auth {
                Outcome::Forward(())
             } else {
                Outcome::Success(PageUser(User::new_unauth()))
            }
        }
    }
}

#[derive(Debug)]
/// Represents permission for a user to read the current page.  They may be logged in or site may be set to allow read without logging in
pub struct PageWriter(pub User);
impl Deref for PageWriter {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for PageWriter {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<PageWriter, Self::Error> {
        let logged_in = request.guard::<User>(); 
        let is_admin_page = request.guard::<IsAdminPage>()?.0;

        match logged_in {
            Outcome::Success(u)
                if !is_admin_page || u.auth == AuthState::AuthAdmin =>
                Outcome::Success(PageWriter(u)),
            _ =>
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }

}

/// is the current page an admin page?
pub struct IsAdminPage(pub bool);
impl<'a, 'r> FromRequest<'a, 'r> for IsAdminPage {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<IsAdminPage, Self::Error> {
        match request.guard::<State<wikifile::WikiStruct<config::ConfigurationStruct>>>() {
            Outcome::Success(cfg) => {
                let path = request.uri().path();
                let adm_list = &cfg.read().unwrap().data.admin_pages;
                Outcome::Success(IsAdminPage(
                    adm_list.iter().any(|e| {
                        path.starts_with(e)
                   }
                )))
            },
            _ => {
                Outcome::Failure((Status::Unauthorized, ())) 
            }
        }
    }  
}
impl Deref for IsAdminPage {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}