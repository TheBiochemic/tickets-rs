
use std::{path::Path, sync::{Arc, Mutex}};

use rusqlite::types::Value;

use tickets_rs_core::{
    Bucket,
    Ticket,
    State,
    Filter,
    Tag,
    TicketAdapter,
    AdapterError,
    AdapterErrorType, Config, LocalDatabase, AppConfig, TicketProvider, FilterType, StateIdentifier, BucketIdentifier
};

use super::{
    LocalTicketAdapter,
    interpreter::{AdapterInterpreter}
};

impl TicketAdapter for LocalTicketAdapter {

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_type_name() -> String where Self: Sized {
        "local".to_string()
    }

    fn get_fancy_type_name() -> String where Self: Sized {
        "Local Tickets".to_string()
    }

    fn get_icon(&self) -> &Path {
        Path::new("assets/adapters/database.png")
    }

    fn get_fancy_name(&self) -> String {
        self.display_name.clone()
    }

    fn create_config() -> Config where Self: Sized {

        TicketProvider::get_default_config::<LocalTicketAdapter>()
            .with("database", "./local.db3", "string")
            .with("include_default_data", true, "bool")
    }

    
    fn from_config(app_config: Arc<Mutex<AppConfig>>, config: &Config, finished: Arc<Mutex<bool>>) -> Result<Box<dyn TicketAdapter + Send + Sync>, AdapterError> where Self: Sized {

        let name: String = match config.get("name") {
            Some(option) => match option.get() {
                Some(result) => result,
                None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
            },
            None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
        };

        let display_name: String = match config.get("display") {
            Some(option) => match option.get() {
                Some(result) => result,
                None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
            },
            None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
        };

        let create_default_data: bool = match config.get("include_default_data") {
            Some(option) => match option.get() {
                Some(result) => result,
                None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
            },
            None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
        };

        let database_name: String = match config.get("database") {
            Some(option) => match option.get() {
                Some(result) => result,
                None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
            },
            None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
        };

        let database = {
            let database = match LocalDatabase::open(database_name) {
                Ok(success) => success,
                Err(_) => {
                    println!("Failed to read Local SQLite Database, exiting!"); 
                    return Err(AdapterError::new(AdapterErrorType::Access));
                }
            };
            Arc::new(Mutex::new(database))
        };


        let local_tickets = LocalTicketAdapter{
            database,
            config: app_config,
            name,
            display_name,
        };

        local_tickets.prepare_database(create_default_data);

        Ok(Box::new(local_tickets))
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn bucket_list_all(&self) -> Vec<Bucket> {

        let mut buckets: Vec<Bucket> = Vec::new();

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {
                        let mut stmt_select = lock.prepare("SELECT * FROM buckets").unwrap();
                
                        let iter = stmt_select.query_map([], |row| {
                            Ok(Bucket {
                                identifier: BucketIdentifier {
                                    adapter: self.get_name(),
                                id: row.get(0).unwrap()
                                },
                                name: row.get(1).unwrap(),
                                last_change: row.get(2).unwrap()
                            })
                        }).unwrap();
                
                        for row in iter {
                            let bucket = row.unwrap();
                            buckets.push(bucket)
                        };
                
                    },
                    Err(e) => println!("Wasn't able to lock for listing Buckets on local, {}", e)
                }
            },
            Err(e) => println!("Wasn't able to lock Database, {}", e)
        }
        buckets
    }

    fn bucket_write(&self, bucket: &mut Bucket) -> Result<(), AdapterError> {

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        if bucket.identifier.id != 0 {
                            // If we're replacing an existing bucket
                            let expression = [
                                "REPLACE INTO buckets",
                                "(id, name, last_change)",
                                "VALUES (:id, :name, :last_change);"].join("");

                            let mut stmt_write = lock.prepare(expression.as_str()).unwrap();

                            match stmt_write.execute(rusqlite::named_params! {
                                ":id": bucket.identifier.id,
                                ":name": bucket.name,
                                ":last_change": bucket.last_change
                            }) {
                                Ok(_) => Ok(()),
                                Err(_) => {
                                    println!("There was an error executing this bucket writing operation!");
                                    Err(AdapterError::new(AdapterErrorType::BucketWrite))
                                }
                            }
                            

                        } else {
                            //If we're just inserting data into the buckets table
                            let expression = [
                                "INSERT INTO buckets",
                                "(name, last_change)",
                                "VALUES (:name, :last_change) RETURNING id;"].join("");

                            let mut stmt_write = lock.prepare(expression.as_str()).unwrap();
                            let write_query = stmt_write.query(rusqlite::named_params! {
                                ":name": bucket.name,
                                ":last_change": bucket.last_change
                            });

                            match write_query {
                                Ok(mut rows) => {
                                    bucket.identifier.id = match rows.next() {
                                        Ok(row_option) => match row_option {
                                            Some(row_value) => match row_value.get(0) {
                                                Ok(found_value) => found_value,
                                                Err(_) => {
                                                    println!("There was an error executing this bucket writing operation!");
                                                    return Err(AdapterError::new(AdapterErrorType::BucketWrite))
                                                },
                                            },
                                            None => {
                                                println!("There was an error executing this bucket writing operation!");
                                                return Err(AdapterError::new(AdapterErrorType::BucketWrite))
                                            },
                                        },
                                        Err(_) => {
                                            println!("There was an error executing this bucket writing operation!");
                                            return Err(AdapterError::new(AdapterErrorType::BucketWrite))
                                        },
                                    };
                                    Ok(())
                                },
                                Err(_) => {
                                    println!("There was an error executing this bucket writing operation!");
                                    return Err(AdapterError::new(AdapterErrorType::BucketWrite))
                                }
                            }
                        }
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for writing Bucket on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::BucketWrite))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for writing Bucket on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::BucketWrite))
            }
        }
    }

    fn state_list_all(&self) -> Vec<State> {
        
        let mut states: Vec<State> = Vec::new();

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {
                        let expression = "SELECT * FROM states";
                        let mut stmt_select = lock.prepare(expression).unwrap();

                        let iter = stmt_select.query_map([], |row| {
                            Ok(State {
                                identifier: StateIdentifier {
                                    adapter: self.get_name(),
                                    name: row.get(0).unwrap()
                                },
                                description: row.get(1).unwrap(),
                                sorting_order: row.get(2).unwrap()
                            })
                        }).unwrap();
                
                        for row in iter {
                            let state = row.unwrap();
                            states.push(state)
                        };
                
                    },
                    Err(e) => println!("Wasn't able to lock for listing States on local, {}", e)
                }
            },
            Err(e) => println!("Wasn't able to lock Database, {}", e)
        }
        states
    }

    fn state_write(&self, state: &State) -> Result<(), AdapterError> {
        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        let expression = [
                            "INSERT INTO states",  
                            "(name, description, sorting_order)", 
                            "VALUES (:name, :description, :sorting_order);"].join("");

                        let mut stmt_write = lock.prepare(expression.as_str()).unwrap();

                        match stmt_write.execute(rusqlite::named_params! {
                            ":name": state.identifier.name,
                            ":description": state.description,
                            ":sorting_order": state.sorting_order
                        }) {
                            Ok(_) => Ok(()),
                            Err(_) => {
                                println!("There was an error executing this writing operation!");
                                Err(AdapterError::new(AdapterErrorType::StateWrite))
                            }
                        }
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for writing State on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::StateWrite))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for writing State on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::StateWrite))
            }
        }
    }

    fn tag_list_all(&self) -> Vec<Tag> {
        let mut tags: Vec<Tag> = Vec::new();

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {
                        let expression = "SELECT * FROM tags";
                        let mut stmt_select = lock.prepare(expression).unwrap();

                        let iter = stmt_select.query_map([], |row| {
                            Ok(Tag {
                                adapter: self.get_name(),
                                name: row.get(0).unwrap(),
                                color: row.get(1).unwrap(),
                                color_text: row.get(2).unwrap()
                            })
                        }).unwrap();
                
                        for row in iter {
                            let tag = row.unwrap();
                            tags.push(tag)
                        };
                
                    },
                    Err(e) => println!("Wasn't able to lock for listing Tags on local, {}", e)
                }
            },
            Err(e) => println!("Wasn't able to lock Database, {}", e)
        }
        tags
    }

    fn tag_write(&self, tag: &Tag) -> Result<(), AdapterError> {
        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        let expression = [
                            "REPLACE INTO tags",  
                            "(name, color, color_text)", 
                            "VALUES (:name, :color, :color_text);"].join("");

                        let mut stmt_write = lock.prepare(expression.as_str()).unwrap();

                        match stmt_write.execute(rusqlite::named_params! {
                            ":name": tag.name,
                            ":color": tag.color,
                            ":color_text": tag.color_text
                        }) {
                            Ok(_) => Ok(()),
                            Err(_) => {
                                println!("There was an error executing this writing operation!");
                                Err(AdapterError::new(AdapterErrorType::TagWrite))
                            }
                        }
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for writing Tag on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::TagWrite))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for writing Tag on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::TagWrite))
            }
        }
    }

    fn filter_list_all(&self) -> Vec<Filter> {
        let mut filters: Vec<Filter> = Vec::new();

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {
                        let expression = "SELECT * FROM filters";
                        let mut stmt_select = lock.prepare(expression).unwrap();

                        let iter = stmt_select.query_map([], |row| {
                            Ok(Filter::default()
                                .with_adapter(self)
                                .with_details(row.get(0).unwrap(), row.get(1).unwrap())
                                .with_type(FilterType::User)
                            )
                        }).unwrap();
                
                        for row in iter {
                            let filter = row.unwrap();
                            filters.push(filter)
                        };
                
                    },
                    Err(e) => println!("Wasn't able to lock for listing Filters on local, {}", e)
                }
            },
            Err(e) => println!("Wasn't able to lock Database, {}", e)
        }

        filters.append(&mut self.list_builtin_filters());

        filters
    }

    fn filter_list(&self, filter_name: String) -> Option<Filter> {

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {
                        let expression = "SELECT * FROM filters WHERE name = :name";
                        let mut stmt_select = lock.prepare(expression).unwrap();

                        let mut iter = stmt_select.query_map(rusqlite::named_params! {
                            ":name": filter_name
                        }, |row| {
                            Ok(Filter::default()
                                .with_adapter(self)
                                .with_details(row.get(0).unwrap(), row.get(1).unwrap())
                                .with_type(FilterType::User)
                            )
                        }).unwrap();
                
                        iter.next().map(|row| row.unwrap())
                
                    },
                    Err(e) => {
                        println!("Wasn't able to lock for listing Filters on local, {}", e);
                        None
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database, {}", e);
                None
            }
        }
    }

    fn filter_expression_validate(&self, filter: &String) -> Result<(), Vec<(String, String)>> {
        let mut validation_errors: Vec<(String, String)> = Vec::default();

        //Try and tokenize the operation
        let mut interpreter = AdapterInterpreter::default();
        interpreter.setup_environment(self.config.clone());

        match interpreter.try_tokenize(filter.to_string()) {
            Ok(_) => (),
            Err(token_error) => {
                validation_errors.push(("operation".to_string(), token_error.to_string()));
            },
        }

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(validation_errors)
        }
    }

    fn filter_write(&self, filter: &Filter) -> Result<(), AdapterError> {

        if !matches!(filter.filter_type, FilterType::User) {
            println!("{} is a builtin Filter, so it is readonly!", filter.identifier.name);
            return Err(AdapterError::new(AdapterErrorType::FilterWrite))
        }

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        let expression = [
                            "REPLACE INTO filters",  
                            "(name, operation)", 
                            "VALUES (:name, :operation);"].join("");

                        let mut stmt_write = lock.prepare(expression.as_str()).unwrap();

                        match stmt_write.execute(rusqlite::named_params! {
                            ":name": filter.identifier.name,
                            ":operation": filter.operation
                        }) {
                            Ok(_) => Ok(()),
                            Err(_) => {
                                println!("There was an error executing this writing operation!");
                                Err(AdapterError::new(AdapterErrorType::FilterWrite))
                            }
                        }
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for writing Filter on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::FilterWrite))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for writing Filter on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::FilterWrite))
            }
        }
    }

    fn ticket_list_unique(&self, id: i64) -> Option<Ticket> {
        let mut ticket_option: Option<Ticket> = None;

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {
                        let expression = "SELECT * FROM tickets WHERE tickets.id = :id;";
                        let mut stmt_select = lock.prepare(expression).unwrap();

                        let iter = stmt_select.query_map(rusqlite::named_params! {
                            ":id": id
                        }, |row| {
                            Ok(Ticket {
                                adapter: self.get_name(),
                                id: row.get(0).unwrap(),
                                bucket_id: row.get(1).unwrap(),
                                title: row.get(2).unwrap(),
                                state_name: row.get(3).unwrap(),
                                description: row.get(4).unwrap(),
                                created_at: row.get(5).unwrap(),
                                due_at: row.get(6).unwrap(),
                                assigned_to: row.get(7).unwrap(),
                                tags: vec![],
                                additional_id: id.to_string()
                            })
                        }).unwrap();

                        for ticket_result in iter {
                            match ticket_result {
                                Ok(ticket) => ticket_option = Some(ticket),
                                Err(_) => ticket_option = None,
                            }
                        }

                        ticket_option = match ticket_option {
                            Some(mut ticket) => {

                                let mut tags: Vec<String> = Vec::new();
                                {
                                    let tags_expression = [
                                        "SELECT tag_name FROM ticket_tags ",
                                        "WHERE ticket_tags.ticket_id = :id;"
                                    ].join("");
    
                                    let mut stmt_tags_join = lock.prepare(tags_expression.as_str()).unwrap();
    
                                    let tags_iter = stmt_tags_join.query_map(rusqlite::named_params! {
                                        ":id": ticket.id
                                    }, |tags_row| {
                                        Ok(tags_row.get(0).unwrap())
                                    }).unwrap();
    
                                    for tags_row in tags_iter {
                                        let tag = tags_row.unwrap();
                                        tags.push(tag)
                                    };
                                }
    
                                ticket.tags.append(&mut tags);
                                Some(ticket)
                            },
                            None => None,
                        };
                    },
                    Err(e) => println!("Wasn't able to lock for listing a Ticket on local, {}", e)
                }
            },
            Err(e) => println!("Wasn't able to lock Database, {}", e)
        }

        ticket_option
    }

    fn bucket_list_unique(&self, id: u64) -> Option<Bucket> {
        let mut bucket_option: Option<Bucket> = None;

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {
                        let expression = "SELECT * FROM buckets WHERE buckets.id = :id;";
                        let mut stmt_select = lock.prepare(expression).unwrap();

                        let iter = stmt_select.query_map(rusqlite::named_params! {
                            ":id": id
                        }, |row| {
                            Ok(Bucket {
                                identifier: BucketIdentifier {
                                    adapter: self.get_name(),
                                    id: row.get(0).unwrap()
                                },
                                name: row.get(1).unwrap(),
                                last_change: row.get(2).unwrap(),
                            })
                        }).unwrap();

                        for bucket_result in iter {
                            match bucket_result {
                                Ok(bucket) => bucket_option = Some(bucket),
                                Err(_) => bucket_option = None,
                            }
                        }
                    },
                    Err(e) => println!("Wasn't able to lock for listing a Bucket on local, {}", e)
                }
            },
            Err(e) => println!("Wasn't able to lock Database, {}", e)
        }

        bucket_option
    }

    fn ticket_list_all(&self) -> Vec<Ticket> {

        let mut tickets: Vec<Ticket> = Vec::new();

        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        // Get tickets first, with empty tags vector
                        let expression = "SELECT * FROM tickets";
                        let mut stmt_select = lock.prepare(expression).unwrap();
                
                        let iter = stmt_select.query_map([], |row| {
                            Ok(Ticket {
                                adapter: self.get_name(),
                                id: row.get(0).unwrap(),
                                bucket_id: row.get(1).unwrap(),
                                title: row.get(2).unwrap(),
                                state_name: row.get(3).unwrap(),
                                description: row.get(4).unwrap(),
                                created_at: row.get(5).unwrap(),
                                due_at: row.get(6).unwrap(),
                                assigned_to: row.get(7).unwrap(),
                                tags: vec![],
                                additional_id: row.get::<_, i64>(0).unwrap().to_string()
                            })
                        }).unwrap();
                
                        for row in iter {
                            let mut ticket = row.unwrap();

                            // get tags vector content, and fill ticket with it
                            let mut tags: Vec<String> = Vec::new();
                            {
                                let tags_expression = [
                                    "SELECT tag_name FROM ticket_tags ",
                                    "WHERE ticket_tags.ticket_id = :id;"
                                ].join("");

                                let mut stmt_tags_join = lock.prepare(tags_expression.as_str()).unwrap();

                                let tags_iter = stmt_tags_join.query_map(rusqlite::named_params! {
                                    ":id": ticket.id
                                }, |tags_row| {
                                    Ok(tags_row.get(0).unwrap())
                                }).unwrap();

                                for tags_row in tags_iter {
                                    let tag = tags_row.unwrap();
                                    tags.push(tag)
                                };
                            }

                            ticket.tags.append(&mut tags);
                            tickets.push(ticket)
                        };

                    },
                    Err(e) => println!("Wasn't able to lock for listing Tickets on local, {}", e)
                }
            },
            Err(e) => println!("Wasn't able to lock Database, {}", e)
        }
        tickets
    }

    fn ticket_list(&self, expression: &str) -> Result<Vec<Ticket>, AdapterError> {

        let mut tickets: Vec<Ticket> = Vec::new();
        let mut interpreter = AdapterInterpreter::default();
        interpreter.setup_environment(self.config.clone());

        let error: Option<AdapterError> = match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        // Get tickets first, with empty tags vector
                        if let Err(error) = interpreter.try_tokenize(expression.to_string()) {
                            println!("TokenizationError: {}", error);
                            return Err(AdapterError::new(AdapterErrorType::Expression(error.to_string())));
                        }

                        let mut compiled_expr = String::default();

                        // Return early on Err(..)
                        match interpreter.construct_sql() {
                            Ok(expr) => {
                                compiled_expr = expr;
                                Ok(())
                            },
                            Err(err) => {
                                println!("SqlExpressionError: {}", err);
                                Err(AdapterError::new(AdapterErrorType::Expression(err.to_string())))
                            }
                        }?;

                        let mut stmt_select = lock.prepare(compiled_expr.as_str()).unwrap();
                
                        let iter = stmt_select.query_map([], |row| {
                            Ok(Ticket {
                                adapter: self.get_name(),
                                id: row.get(0).unwrap(),
                                bucket_id: row.get(1).unwrap(),
                                title: row.get(2).unwrap(),
                                state_name: row.get(3).unwrap(),
                                description: row.get(4).unwrap(),
                                created_at: row.get(5).unwrap(),
                                due_at: row.get(6).unwrap(),
                                assigned_to: row.get(7).unwrap(),
                                tags: vec![],
                                additional_id: row.get::<_, i64>(0).unwrap().to_string()
                            })
                        }).unwrap();
                
                        for row in iter {
                            let mut ticket = row.unwrap();

                            // get tags vector content, and fill ticket with it
                            let mut tags: Vec<String> = Vec::new();
                            {
                                let tags_expression = [
                                    "SELECT tag_name FROM ticket_tags ",
                                    "WHERE ticket_tags.ticket_id = :id;"
                                ].join("");

                                let mut stmt_tags_join = lock.prepare(tags_expression.as_str()).unwrap();

                                let tags_iter = stmt_tags_join.query_map(rusqlite::named_params! {
                                    ":id": ticket.id
                                }, |tags_row| {
                                    Ok(tags_row.get(0).unwrap())
                                }).unwrap();

                                for tags_row in tags_iter {
                                    let tag = tags_row.unwrap();
                                    tags.push(tag)
                                };
                            }

                            ticket.tags.append(&mut tags);
                            tickets.push(ticket)
                        };
                        None
                    },
                    Err(e) => {
                        println!("Wasn't able to lock for listing Tickets on local, {}", e);
                        Some(AdapterError::new(AdapterErrorType::Access))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database, {}", e);
                Some(AdapterError::new(AdapterErrorType::Access))
            }
        };
        
        match error {
            None => Ok(tickets),
            Some(err) => Err(err)
        }
    }

    fn ticket_write(&self, ticket: &Ticket) -> Result<(), AdapterError> {
        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        let mut ticket = ticket.clone();

                        // Replace Ticket (or add with new id, if id is 0)
                        if ticket.id != 0 {

                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("REPLACE INTO tickets ");
                            expression.push("(id, bucket_id, title, state_name, description, created_at, due_at, assigned_to) "); 
                            expression.push("VALUES (?, ?, ?, ?, ?, ?, ?, ?); ");
                            parameters.push(Value::Integer(ticket.id));
                            parameters.push(Value::Integer(ticket.bucket_id as i64));
                            parameters.push(Value::Text(ticket.title.clone()));
                            parameters.push(Value::Text(ticket.state_name.clone()));
                            parameters.push(Value::Text(ticket.description.clone()));
                            parameters.push(Value::Integer(ticket.created_at));
                            parameters.push(Value::Integer(ticket.due_at));
                            parameters.push(Value::Text(ticket.assigned_to.clone()));

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            if let Err(err) = stmt_write.execute(rusqlite::params_from_iter(parameters)) {
                                println!("There was an error executing this replace ticket operation! Reason: {}", err);
                            };

                        } else {
                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("INSERT INTO tickets ");
                            expression.push("(bucket_id, title, state_name, description, created_at, due_at, assigned_to) "); 
                            expression.push("VALUES (?, ?, ?, ?, ?, ?, ?) returning id; ");
                            parameters.push(Value::Integer(ticket.bucket_id as i64));
                            parameters.push(Value::Text(ticket.title.clone()));
                            parameters.push(Value::Text(ticket.state_name.clone()));
                            parameters.push(Value::Text(ticket.description.clone()));
                            parameters.push(Value::Integer(ticket.created_at));
                            parameters.push(Value::Integer(ticket.due_at));
                            parameters.push(Value::Text(ticket.assigned_to.clone()));

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            match stmt_write.query(rusqlite::params_from_iter(parameters)) {
                                Err(err) => {
                                    println!("There was an error executing this insert ticket operation! Reason: {}", err);
                                },
                                Ok(mut rows) => {
                                    
                                    let ticket_ref = ticket.clone();
                                    let mut id = 0;

                                    while let Some(row) = rows.next().unwrap() {
                                        id = row.get(0).unwrap();
                                    }

                                    ticket = ticket.with_details(id, ticket_ref.title, ticket_ref.description);
                                },
                            };
                        }

                        // Delete old tag References of ticket
                        {
                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("DELETE FROM ticket_tags WHERE ticket_id = ?; ");
                            parameters.push(Value::Integer(ticket.id));

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            if let Err(err) = stmt_write.execute(rusqlite::params_from_iter(parameters)) {
                                println!("There was an error executing delete tag refs operation! Reason: {}", err);
                            };
                        }

                        // Add new tag references
                        if !ticket.tags.is_empty() {
                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("INSERT INTO ticket_tags VALUES ");
                            if let Some((last_tag, tags)) = ticket.tags.split_last() {
                                for tag in tags {
                                    expression.push("(?, ?), ");
                                    parameters.push(Value::Integer(ticket.id));
                                    parameters.push(Value::Text(tag.to_string()));
                                }

                                expression.push("(?, ?); ");
                                parameters.push(Value::Integer(ticket.id));
                                parameters.push(Value::Text(last_tag.to_string()));
                            }

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            if let Err(err) = stmt_write.execute(rusqlite::params_from_iter(parameters)) {
                                println!("There was an error executing insert tags operation! Reason: {}", err);
                            };
                        }

                        Ok(())
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for writing Ticket on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::TicketWrite))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for writing Ticket on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::TicketWrite))
            }
        }
    }

    fn ticket_drop(&self, ticket: &Ticket) -> Result<(), AdapterError> {
        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {
                        // Delete old tag References of ticket
                        {
                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("DELETE FROM ticket_tags WHERE ticket_id = ?; ");
                            parameters.push(Value::Integer(ticket.id));

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            if let Err(err) = stmt_write.execute(rusqlite::params_from_iter(parameters)) {
                                println!("There was an error executing delete tag refs operation! Reason: {}", err);
                                return Err(AdapterError::new(AdapterErrorType::TicketDelete));
                            };
                        }

                        //Delete ticket itself
                        {
                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("DELETE FROM tickets WHERE id = ?; ");
                            parameters.push(Value::Integer(ticket.id));

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            if let Err(err) = stmt_write.execute(rusqlite::params_from_iter(parameters)) {
                                println!("There was an error executing delete ticket operation! Reason: {}", err);
                                return Err(AdapterError::new(AdapterErrorType::TicketDelete));
                            };
                        }

                        Ok(())
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for deleting Ticket on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::TicketDelete))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for deleting Ticket on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::TicketDelete))
            }
        }
    }

    fn filter_drop(&self, filter: &Filter) -> Result<(), AdapterError> {
        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        //Delete filter
                        {
                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("DELETE FROM filters WHERE name = ?; ");
                            parameters.push(Value::Text(filter.identifier.name.clone()));

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            if let Err(err) = stmt_write.execute(rusqlite::params_from_iter(parameters)) {
                                println!("There was an error executing delete Filter operation! Reason: {}", err);
                                return Err(AdapterError::new(AdapterErrorType::FilterDelete));
                            };
                        }

                        Ok(())
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for deleting Filter on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::FilterDelete))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for deleting Filter on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::FilterDelete))
            }
        }
    }

    fn tag_drop(&self, tag: &Tag) -> Result<(), AdapterError> {
        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        //Delete tag
                        {
                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("DELETE FROM tags WHERE name = ?; ");
                            parameters.push(Value::Text(tag.name.clone()));

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            if let Err(err) = stmt_write.execute(rusqlite::params_from_iter(parameters)) {
                                println!("There was an error executing delete Tag operation! Reason: {}", err);
                                return Err(AdapterError::new(AdapterErrorType::TagDelete));
                            };
                        }

                        Ok(())
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for deleting Tag on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::TagDelete))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for deleting Tag on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::TagDelete))
            }
        }
    }

    fn bucket_drop(&self, bucket: &Bucket) -> Result<(), AdapterError> {
        match self.database.lock() {
            Ok(db_lock) => {
                match db_lock.connection.lock() {
                    Ok(lock) => {

                        //Delete bucket
                        {
                            let mut expression: Vec<&str> = vec![];
                            let mut parameters: Vec<rusqlite::types::Value> = vec![];

                            expression.push("DELETE FROM buckets WHERE id = ?; ");
                            parameters.push(Value::Integer(bucket.identifier.id as i64));

                            let mut stmt_write = lock.prepare(expression.join("").as_str()).unwrap();
                            if let Err(err) = stmt_write.execute(rusqlite::params_from_iter(parameters)) {
                                println!("There was an error executing delete Bucket operation! Reason: {}", err);
                                return Err(AdapterError::new(AdapterErrorType::BucketDelete));
                            };
                        }

                        Ok(())
                    },
                    Err(e) => {
                        println!("Wasn't able to lock Connection for deleting Bucket on local, {}", e);
                        Err(AdapterError::new(AdapterErrorType::BucketDelete))
                    }
                }
            },
            Err(e) => {
                println!("Wasn't able to lock Database for deleting Bucket on local, {}", e);
                Err(AdapterError::new(AdapterErrorType::BucketDelete))
            }
        }
    }
}