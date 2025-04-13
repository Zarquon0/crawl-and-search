use crate::prelude::*;
use crate::crawler_datatypes::SiteMap;
use rusqlite::{params, Connection, OpenFlags};

pub fn load_db(db_path: &PathBuf, site_map: Arc<SiteMap>) -> Result<()> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
    let mut raw_data = conn.prepare("SELECT url, title FROM site")?;
    let tuple_data = raw_data.query_map([], |row| {
        let url: String = row.get(0)?;
        //let title: Option<String> = row.get(1)?;
        Ok(url)
    })?;
    for row in tuple_data {
        let url = row?;
        site_map.insert_previously(url);
    }
    Ok(())
}

pub fn update_db(db_path: &PathBuf, site_map: Arc<SiteMap>) {
    match Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE) {
        Ok(mut conn) => {
            let cursor = conn.transaction().expect("Unable to create transaction");
            let map = site_map.get_map();
            for (url, data) in &*map {
                match cursor.execute("INSERT INTO site VALUES (?1, ?2)", params![url, data.title]) {
                    Ok(_) => {},
                    Err(e) => {
                        //cursor.rollback().unwrap();
                        return eprintln!("DATABASE ERROR, continuing: {e}")
                    }
                }
            }
            cursor.commit().unwrap();
        },
        Err(e) => eprintln!("DATABASE ERROR: {e}\nTLDR; The input database path probably didn't lead to a proper database")
    }
}