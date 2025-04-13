use std::sync::{LazyLock, Mutex};

use rusqlite::Connection;

static CONN: LazyLock<Mutex<Connection>> =
    LazyLock::new(|| Mutex::new(Connection::open("./data/data.db").unwrap()));

pub fn address_exists(addr: &str) -> bool {
    let conn = CONN.lock().unwrap();
    let mut stmt = conn
        .prepare_cached("SELECT 1 FROM accounts WHERE address = ?1")
        .unwrap();
    let num: Result<i32, rusqlite::Error> = stmt.query_row([addr], |r| r.get(0));

    match num {
        Ok(_) => true,
        Err(rusqlite::Error::QueryReturnedNoRows) => false,
        Err(e) => panic!("{e:?}"),
    }
}
