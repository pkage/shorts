use rocket::Outcome;
use rocket::request::{self, FromRequest, Request};

use bcrypt;

use super::db::User;

pub struct UserID(pub i32);

pub fn validate_login(profile: &User, password: &String) -> bool {

    // check the hash against what we get out from the username
    bcrypt::verify(password, &profile.hash)
        .expect("Hash the password")
}

impl<'a, 'r> FromRequest<'a, 'r> for UserID {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {

        println!("Checking request...");

        match request.cookies().get_private("user_id") {
            Some( uid ) => Outcome::Success( UserID(uid.value().parse::<i32>().unwrap()) ),
            None        => Outcome::Forward( () )
        }
    }
}

