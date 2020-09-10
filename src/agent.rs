use rocket::Outcome;
use rocket::request::{self, FromRequest, Request};

pub struct UserAgent {
    pub ua: Option<String>
}

impl<'a, 'r> FromRequest<'a, 'r> for UserAgent {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {

        println!("Checking request...");

        match request.headers().get_one("User-Agent") {
            Some( ua ) => Outcome::Success( UserAgent{ ua: Some(String::from(ua)) } ),
            None       => Outcome::Success( UserAgent{ ua: None } )
        }
    }
}
