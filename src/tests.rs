use super::create_rocket;
use rocket::local::Client;
use rocket::http::Status;

#[test]
fn root_test() {
    let client = Client::new(create_rocket()).expect("valid rocket instance");
    let mut response = client.get("/").dispatch();
    assert_eq!(response.status(), Status::Ok);
//    assert_eq!(response.body_string(), Some("Hello, world!".into()));
}
