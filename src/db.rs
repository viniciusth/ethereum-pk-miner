use std::{cmp::Ordering, fs::File, io::{Read, Seek, SeekFrom}, sync::{LazyLock, Mutex}};

use rusqlite::Connection;

static CONN: LazyLock<Mutex<Connection>> = LazyLock::new(|| {
    Mutex::new(Connection::open("./data/data.db").unwrap())
});

pub fn sort_addresses(count: u64) {
    // maybe later to see the latency of doing it?
}

pub fn address_exists(addr: &str) -> bool {
    let conn = CONN.lock().unwrap();
    let mut stmt = conn.prepare_cached("SELECT 1 FROM accounts WHERE address = ?1").unwrap();
    let num: Result<i32, rusqlite::Error> = stmt.query_row([addr], |r| r.get(0));

    return match num {
        Ok(_) => true,
        Err(rusqlite::Error::QueryReturnedNoRows) => false,
        Err(e) => panic!("{e:?}")
    };

    let mut reader = File::open("./data/addressdb").unwrap();
    let mut len = [0; 8];
    reader.read(&mut len).unwrap();
    let len = usize::from_be_bytes(len);

    let mut read_element = |idx: usize| {
        reader.seek(SeekFrom::Start(idx as u64 * 42 + usize::BITS as u64 / 8)).unwrap();
        let mut s = vec![0; 42];
        reader.read(&mut s).unwrap();
        String::from_utf8(s).unwrap()
    };

    let mut lo = 0;
    let mut hi = len - 1;
    let mut mid;
    while lo <= hi {
        mid = (lo + hi) / 2;
        match addr.cmp(&read_element(mid)) {
            Ordering::Greater => lo = mid + 1,
            Ordering::Equal => return true,
            Ordering::Less => hi = mid - 1,
        }
    }

    false
}



