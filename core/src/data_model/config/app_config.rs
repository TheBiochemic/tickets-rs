use std::sync::{
    Mutex, 
    Arc
};

use rusqlite::types::Value;
use std::str;

use crate::LocalDatabase;

use super::{ConfigOption, Config};
use super::config_option::ToConfig;


pub struct AppConfig {
    database: Arc<Mutex<LocalDatabase>>,
    config: Config
}

impl Drop for AppConfig {
    fn drop(&mut self) {

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        // Build dynamic expression
                        let mut expression: Vec<&str> = vec![];
                        let mut parameters: Vec<rusqlite::types::Value> = vec![];

                        //remove all old content
                        {
                            let mut statement = lock.prepare("DELETE FROM config").unwrap();

                            if let Err(err) = statement.execute([]) {
                                println!("There was an error executing this first writing operation! Reason: {}", err);
                            }
                        }

                        if self.config.is_empty() {
                            return;
                        };

                        //add new and updated entries
                        expression.push("INSERT INTO config");
                        expression.push("(name, value, display_options)"); 
                        expression.push("VALUES");

                        let options_vec = Vec::from_iter(self.config.iter());
                        if let Some((last_option, options)) = options_vec.split_last() {
                            for option in options {
                                expression.push("(?, ?, ?),");
                                parameters.push(Value::Text(option.0.to_string()));
                                parameters.push(Value::Text(option.1.value.clone()));
                                parameters.push(Value::Text(option.1.display_options.clone()));
                            }

                            expression.push("(?, ?, ?)");
                            parameters.push(Value::Text(last_option.0.to_string()));
                            parameters.push(Value::Text(last_option.1.value.clone()));
                            parameters.push(Value::Text(last_option.1.display_options.clone()));
                        }

                        // println!("{:?}", expression);
                        // println!("{:?}", parameters);

                        // Finally, execute query, that deletes old data, and updates config options
                        let mut statement = lock.prepare(expression.join(" ").as_str()).unwrap();

                        let execute_params = rusqlite::params_from_iter(parameters.iter());
                        if let Err(err) = statement.execute(execute_params) {
                            println!("There was an error executing this writing operation! Reason: {}", err);
                        };

                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for writing Config, {}", e);
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for writing Config, {}", e);
            }
        }

    }
}


impl AppConfig {

    pub fn new(database: Arc<Mutex<LocalDatabase>>) -> Self {

        let mut config = Config::default();

        //Lock the Database
        match database.lock() {
            Ok(mut lock) => {

                //create the table, of possible
                if lock.create_table(
                    &String::from("config"), vec![
                        String::from("name TEXT NOT NULL PRIMARY KEY"),
                        String::from("value TEXT NOT NULL"), 
                        String::from("display_options TEXT NOT NULL")]) {

                    config.put("username", "new User", "");
                }

                //Load all entries into RAM
                match lock.connection.lock() {
                    Ok(conn_lock) => {

                        let expression = String::from("SELECT * FROM config");
                        let mut statement = conn_lock.prepare(expression.as_str()).unwrap();

                        let mut rows = statement.query([]).unwrap();

                        while let Some(row) = rows.next().unwrap() {

                            let name: String = row.get(0).unwrap();
                            let value: String = row.get(1).unwrap();
                            let display_options: String = row.get(2).unwrap();

                            config.put(name.as_str(), value, display_options.as_str());
                        }

                    },
                    Err(_) => println!("Wasn't able to lock connection within Database for Config")
                }
            },
            Err(err) => println!("Wasn't able to lock Database for preparing config. Reason: {}", err)
        }


        AppConfig {
            database,
            config
        }
    }

    pub fn get(&self, name: &str) -> Option<&ConfigOption> {
        self.config.get(name)
    }

    pub fn get_or_default<T: ToConfig>(&mut self, name: &str, value: T, display_options: &str) -> &ConfigOption {
        self.config.get_or_default(name, value, display_options)
    }

    pub fn put<T: ToConfig>(&mut self, name: &str, value: T, display_options: &str) {
        self.config.put(name, value, display_options)
    }

    pub fn get_sub_config(&self, prefix: &str) -> Config {
        self.config.get_sub_config(prefix)
    }

    pub fn put_sub_config(&mut self, other: &Config, prefix: &str) {
        self.config.put_sub_config(other, prefix);
    }

    pub fn drop_entry(&mut self, entry_name: &str) -> bool {
        self.config.drop_entry(entry_name)
    }

    pub fn drop_sub_config(&mut self, prefix: &str) -> u32 {
        self.config.drop_sub_config(prefix)
    }
}