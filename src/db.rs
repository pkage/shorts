/**
 * Module for database-adjacent operations
 */

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;
use rusqlite::{params, Connection, Result};


#[derive(Debug, Serialize)]
pub struct Link {
    pub id: i32,
    pub short: String,
    pub original: String,
    pub hit_count: i32
}

#[derive(Debug, Serialize)]
pub struct Hit {
    pub id: i32,
    pub link: i32,
    pub time: i64,
    pub user_agent: String
}

#[derive(Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub hash: String
}

#[derive(Debug, Serialize)]
pub struct Session {
    pub user: i32,
    pub key: String,
    pub expires: i64
}


/**
 * Initialize the database connection, running the init script if
 * it wasn't run before.
 */
pub fn create_db() -> Connection {
    let db_path = Path::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shorts.db")
    );

    print!("{}\n", db_path.display());

    let mut should_run_init = false;
    if !db_path.exists() {
        print!("database shorts.db does not exist!\n");
        should_run_init = true;
    };

    let conn = Connection::open(db_path)
        .expect("open database path");

    if should_run_init {
        let init_path = Path::new(
            concat!(env!("CARGO_MANIFEST_DIR"), "/schema.sql")
        );

        let mut file = File::open(init_path)
            .expect("open schema file");

        let mut init_script = String::new();
        file.read_to_string(&mut init_script)
            .expect("read init schema");

        print!("{}\n", init_script);
        

        conn.execute_batch(init_script.as_str())
            .expect("Init script error!");
    }

    return conn
}

pub fn create_link(conn: &Connection, short: &str, original: &str) -> bool {
    let res = conn.execute(
        "INSERT INTO Links (short, original) VALUES ($1, $2)",
        params![short, original]
    );

    // if we get zero rows changed, something is wrong
    match res {
        Ok(0) => false,
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn get_link(conn: &Connection, short: &str) -> Result<Link, ()> {
    let query = conn.query_row(
        "SELECT original, short, id, COUNT() FROM Links WHERE short = $1",
        params![short],
        |row| {
            Ok(Link {
                original: row.get(0)?,
                short: row.get(1)?,
                id: row.get(2)?,
                hit_count: 0
            })
        }
    );

    match query {
        Ok(link) => Ok(link),
        Err(_) => Err(())
    }
}

pub fn delete_link(conn: &Connection, short: &str) -> Result<usize> {
    conn.execute("DELETE FROM Links WHERE short=$1", params![short])
}

pub fn write_hit(conn: &Connection, link: &Link, ua: Option<String>) -> () {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    conn.execute(
        "INSERT INTO Hits (parent, time, user_agent) VALUES ($1, $2, $3)",
        params![link.id, time as u32, ua]
    )
        .expect("should write hit");
}

pub fn get_all_links(conn: &Connection) -> Result<Vec<Link>> {

    let mut stmt = conn.prepare("
        SELECT
            l.id,
            l.short,
            l.original,
            (SELECT COUNT(h.id) FROM Hits h WHERE h.parent = l.id)
        FROM Links l;
    ")?;

    let link_iter = stmt.query_map(params![], |row|
        Ok(Link {
            id: row.get(0)?,
            short: row.get(1)?,
            original: row.get(2)?,
            hit_count: row.get(3)?
        })
    )?;

    Ok(link_iter.map(|r| r.unwrap()).collect())
}

pub fn get_total_hit_count(conn: &Connection) -> Result<u32> {
    let total_hits = conn.query_row("
        SELECT
            COUNT(h.id)
        FROM Hits h;
    ",
        params![],
        |row| { row.get(0) }
    )?;

    Ok(total_hits)
}

pub fn create_user(conn: &Connection, email: &String, password: &String) -> Result<User, &'static str> {
    let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .expect("Hash password for user creation");

    let result = conn.execute(
        "INSERT INTO Users (email, password) VALUES ($1, $2)",
        params![email, hash]
    );

    match result {
        Ok( _ ) => get_user_profile(&conn, &email),
        Err( _ ) => Err( "This user already exists!" )
    }
}

pub fn get_user_profile_by_id(conn: &Connection, user_id: i32) -> Result<User, &'static str> {
    // Retrieve the user profile
    let result = conn.query_row(
        "SELECT id, email, password FROM Users WHERE id=$1",
        params![user_id],
        |row| { 
            Ok(User{
                id: row.get(0)?,
                email: row.get(1)?,
                hash: row.get(2)?
            }) 
        }
    );

    match result {
        Ok( user ) => Ok(user),
        Err( _ )   => Err( "Could not retrieve user details!" )
    }
}

pub fn get_user_profile(conn: &Connection, email: &String) -> Result<User, &'static str> {
    // Retrieve the user profile
    let result = conn.query_row(
        "SELECT id, email, password FROM Users WHERE email=$1",
        params![email],
        |row| { 
            Ok(User{
                id: row.get(0)?,
                email: row.get(1)?,
                hash: row.get(2)?
            }) 
        }
    );

    match result {
        Ok( user ) => Ok(user),
        Err( _ )   => Err( "Could not retrieve user details!" )
    }
}
