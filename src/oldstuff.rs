const SDF1 : &str = r#"{
	"Page": "start",
	"Revision": "000000000",
	"PreviousRevision": "000000000",
	"CreateDate": "2018/05/05 19:53:05.248-07:00",
	"RevisionDate": "2018/05/05 19:53:05.248-07:00",
	"RevisedBy": "user",
	"Comment": ""
}
<!--REVISION HEADER DEMARCATION>
"#;

const SDF2 : &str = r#"{
	"Page"sdf2: "_user",
	"Revision": "000000001",
	"PreviousRevision": "000000000",
	"CreateDate": "2018/05/12 19:53:05.248-07:00",
	"RevisionDate": "2018/05/12 19:53:05.248-07:00",
	"RevisedBy": "user",
	"Comment": "Initial save"
}
<!--REVISION HEADER DEMARCATION>
{
	"user_list": [
		{
			"User": "user",
			"Password": "pwd",
			"Salt": "",
			"Comment": ""
		},
		{
			"User": "root",
			"Password": "pwd",
			"Salt": "",
			"Comment": ""
		}
	]
}"#;


/*

#[derive(Responder)]
enum StringResponder {
    #[response(status=200, content_type="text/html")]
    Content(String),
    #[response(status=500)]
    Nothing(String)
}
*/

/*
#[derive(FromForm)]
struct Task {
   description: String,
   completed: bool
}
*/



/*
TODO - return all the strange 500 errors from the GO
pub struct WikiResponse {
    value: u16,
    str: String
}

#[catch(404)]
fn not_found(req: &rocket::Request) -> content::Html<String> {
    content::Html(format!("<p>Sorry, but '{}' is not a valid path!</p>
            <p>Try visiting /hello/&lt;name&gt;/&lt;age&gt; instead.</p>",
            req.uri()))
}

impl Responder<'static> for WikiResponse {
    fn respond_to(self, _: &Request) ->  Response {
    let mut response = Response::new();
    response.set_status(Status::new(self.value, self.str.to_owned()));
    return response
   }
}
*/
