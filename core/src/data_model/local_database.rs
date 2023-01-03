use std::{
    sync::{
        Arc, 
        Mutex
    }, 
    time::{
        SystemTime, 
        UNIX_EPOCH
    }
};

use rusqlite::{Connection, Error};


pub struct LocalDatabase {
    pub connection: Arc<Mutex<Connection>>
}

impl LocalDatabase {

    fn _now_timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    pub fn create_table(&mut self, tablename: &String, attributes: Vec<String>) -> bool {
        let connection = self.connection.lock().unwrap();
        let mut stmt_table_exists = connection.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=(?)").unwrap();
        let mut rows = stmt_table_exists.query([tablename]).unwrap();

        let mut tables: Vec<String> = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            tables.push(row.get(0).unwrap());
        }

        if tables.is_empty() {

            let expression = ["CREATE TABLE ", tablename.as_str(), "(", attributes.join(", ").as_str(), ");"].join("");

            let mut stmt_table_create = connection.prepare(expression.as_str()).unwrap();
            stmt_table_create.execute([]).unwrap();
            true
        } else {
            false
        }
    }

    pub fn open(path: String) -> Result<LocalDatabase, Error> {
        match Connection::open(path.as_str()) {
            Ok(conn) => Ok(LocalDatabase{connection: Arc::new(Mutex::new(conn))}),
            Err(err) => Err(err),
        }

        
    }
}