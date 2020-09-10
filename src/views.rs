use std::env;
use std::sync::Mutex;
use rocket::{State, uri};
use rocket::http::{Cookies, Cookie, RawStr};
use rocket_contrib::templates::Template;
use rocket::request::{Form, FlashMessage};
use rocket::response::{Flash, Redirect};

use rusqlite::Connection;

use super::db;
use super::users;
use super::agent;

// create a nice type alias
pub type DBConn = Mutex<Connection>;


#[derive(Debug, Serialize)]
struct TemplateContext {
    all: Vec<db::Link>,
    total_hits: u32,
    user: Option<db::User>,
    flash_msg: Option<String>
}

#[derive(FromForm)]
pub struct LinkFormSubmission {
    url:   String,
    short: String
}

#[derive(FromForm)]
pub struct CreateAccountFormSubmission {
    email:    String,
    password: String,
    invite:   String
}

#[derive(FromForm)]
pub struct LoginFormSubmission {
    email:    String,
    password: String
}

fn get_flash_string(flash: Option<FlashMessage>) -> Option<String> {
    match flash {
        Some( flash ) => Some( String::from(flash.msg()) ),
        None          => None
    }
}


#[get("/")]
pub fn index_admin(db_conn: State<DBConn>, user_id: users::UserID, flash: Option<FlashMessage>) -> Template {
    let conn = db_conn.lock()
        .unwrap();

    println!("Rendering admin panel for user {}...", user_id.0);

    let context = TemplateContext {
        all: db::get_all_links(&conn).unwrap_or(vec![]),
        total_hits: db::get_total_hit_count(&conn).unwrap_or(1),
        user: match db::get_user_profile_by_id(&conn, user_id.0) {
            Ok(user) => Some(user),
            Err(_)   => None
        },
        flash_msg: get_flash_string(flash)
    };

    Template::render("index", &context)
}

#[get("/", rank=2)]
pub fn index(db_conn: State<DBConn>, flash: Option<FlashMessage>) -> Template {

    let conn = db_conn.lock()
        .unwrap();

    let context = TemplateContext {
        all: db::get_all_links(&conn).unwrap_or(vec![]),
        total_hits: db::get_total_hit_count(&conn).unwrap_or(1),
        user: None,
        flash_msg: get_flash_string(flash)
    };

    Template::render("index", &context)
}


#[get("/x/<short>")]
pub fn handle_link(short: &RawStr, db_conn: State<DBConn>, agent: agent::UserAgent) -> Redirect {
    format!("Checking for {}", short.as_str());
    
    let conn = db_conn.lock()
        .unwrap();

    let link = match db::get_link(&conn, short.as_str()) {
        Ok(link) => {
            db::write_hit(&conn, &link, agent.ua);
            Redirect::to(link.original)
        },
        Err(()) =>  Redirect::to(uri!(handle_notfound))
    };

    return link;
}

#[get("/delete/<short>")]
pub fn handle_link_delete(short: &RawStr, db_conn: State<DBConn>, _user_id: users::UserID) -> Flash<Redirect> {
    format!("Checking for {}", short.as_str());
    
    let conn = db_conn.lock()
        .unwrap();

    match db::delete_link(&conn, short.as_str()) {
        Ok(_)  => Flash::success(Redirect::to("/"), "Deleted successfully!"),
        Err(_) => Flash::error(Redirect::to("/"), "Delete failed!")
    }
}

#[get("/notfound")]
pub fn handle_notfound() -> &'static str {
    "The specified shortlink was not found."
}


#[post("/submit", data = "<link>")]
pub fn handle_submit(link: Form<LinkFormSubmission>, db_conn: State<DBConn>, _user_id: users::UserID) -> Redirect {

    let conn = db_conn.lock()
        .unwrap();

    println!("Got: {} : {}", link.short, link.url);

    db::create_link(&conn, &link.short, &link.url);

    Redirect::to(uri!(index))
}

// LOGIN

#[post("/account/login", data="<form>")]
pub fn handle_login(form: Form<LoginFormSubmission>, db_conn: State<DBConn>, mut cookies: Cookies) -> Result<Redirect, Flash<Redirect>> {

    let conn = db_conn.lock()
        .unwrap();

    let profile = db::get_user_profile(&conn, &form.email)
        .expect("Get a profile");

    match users::validate_login(&profile, &form.password) {
        true => {
            cookies.add_private(Cookie::new("user_id", profile.id.to_string()));
            Ok(Redirect::to("/"))
        }
        false => {
            Err(Flash::error(Redirect::to("/"), "Invalid username or password"))
        }
    }

}

#[post("/account/create", data="<form>")]
pub fn handle_register(form: Form<CreateAccountFormSubmission>, db_conn: State<DBConn>, mut cookies: Cookies) -> Flash<Redirect> {
    
    let conn = db_conn.lock()
        .unwrap();

    let invite_code = match env::var("SHORTS_INVITE") {
        Ok( val ) => val,
        Err( _ )  => return Flash::error(Redirect::to("/"), "no invite set!")
    };

    println!("Invite code: {}", invite_code);

    if form.invite != invite_code {
        return Flash::error(Redirect::to("/"), "invalid invite code");
    }

    let perhaps_profile = db::create_user(&conn, &form.email, &form.password);

    match perhaps_profile {
        Ok( profile ) => {
            cookies.add_private(Cookie::new("user_id", profile.id.to_string()));
            Flash::success(Redirect::to("/"), "Created account successfully")
        },
        Err(msg)        => {
            Flash::error(Redirect::to("/"), msg)
        }
    }
}

#[get("/account/logout")]
pub fn handle_logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id"));

    Flash::success(Redirect::to("/"), "Logged out successfully!")
}

