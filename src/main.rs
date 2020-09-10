#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate rusqlite;
extern crate bcrypt;

use std::env;
use std::sync::Mutex;
use rocket::Rocket;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::Template;

mod views;
mod db;
mod users;
mod agent;

/**
 * Configure the Rocket instance, and initialize the database if it does not exist
 */
fn rocket() -> Rocket {
    let conn = db::create_db();

    rocket::ignite()
        .manage(Mutex::new(conn))
        .attach(Template::fairing())
        .mount("/", routes![
            views::handle_link,
            views::handle_link_delete,
            views::handle_notfound,
            views::handle_submit,

            views::handle_login,
            views::handle_logout,
            views::handle_register,

            views::index_admin,
            views::index
        ])
        .mount("/static", StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
}

fn main() {
    
    rocket().launch();
}
