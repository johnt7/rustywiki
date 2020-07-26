
//! A [Rocket](https://github.com/SergioBenitez/Rocket) library to provide a base for
//! Basic Authentication over which a concrete authentication mechanism can be built.
//!
//! This library exports [BasicAuthRaw] which you could directly use on the request handler.
//! # Example
//!
//! ```
//! use basic_auth_raw::BasicAuthRaw;
//!
//! #[get("/secure-path")
//! fn secret(basic: BasicAuthRaw) -> String {
//!     format!("Your username is {}", basic.username);
//! }
//! ```
//!
//! Or you could build Request Guards on top of it (Recommended).
//! # Example
//!
//! ```
//! use basic_auth_raw::BasicAuthRaw;
//!
//! struct Admin(User);
//!
//! impl<'a, 'r> FromRequest<'a, 'r> for Admin {
//!     type Error = ();
//!
//!     fn from_request(request: &Request) -> Outcome<Self, Self::Error> {
//!         let basic = BasicAuthRaw::from_request(request)?;
//!         let user = User::from_db(basic.username, basic.password);
//!         if user.is_admin {
//!             Outcome::Success(user);
//!         } else {
//!             Outcome::Failure((Status::Unauthorized, ()));
//!         }
//!     }
//! }
//!
//! #[get("/secure-path")
//! fn secret(admin: Admin) -> String {
//!     format!("Your username is {}", admin.user.username);
//! }
//! ```

extern crate rocket;
extern crate base64;

use rocket::{
    Request,
    Outcome,
    outcome::IntoOutcome,
    http::Status,
    request::FromRequest
};

pub struct BasicAuthRaw {
    pub username: String,
    pub password: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for BasicAuthRaw {
    type Error = ();

    fn from_request(request: &Request) -> Outcome<Self, (Status, <Self as FromRequest<'a, 'r>>::Error), ()> {
        let auth_header = request.headers().get_one("Authorization");
        if let Some(auth_header) = auth_header {
            let split = auth_header.split_whitespace().collect::<Vec<_>>();
            if split.len() != 2 {
                return Outcome::Failure((Status::Unauthorized, ()));
            }
            let (basic, payload) = (split[0], split[1]);
            if basic != "Basic" {
                return Outcome::Failure((Status::Unauthorized, ()));
            }
            let decoded = base64::decode(payload)
                .ok()
                .into_outcome((Status::BadRequest, ()))?;

            let decoded_str = String::from_utf8(decoded)
                .ok()
                .into_outcome((Status::BadRequest, ()))?;

            let split = decoded_str.split(":").collect::<Vec<_>>();

            // If exactly username & password pair are present
            if split.len() != 2 {
                return Outcome::Failure((Status::BadRequest, ()));
            }

            let (username, password) = (split[0].to_string(), split[1].to_string());

            Outcome::Success(BasicAuthRaw {
                username,
                password
            })
        } else {
            Outcome::Failure((Status::Unauthorized, ()))
        }
    }
}