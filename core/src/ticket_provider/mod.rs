use std::{sync::{
    Mutex, 
    Arc
}, time::{SystemTime, UNIX_EPOCH}};

use eframe::egui::epaint::ahash::{HashMap, HashMapExt};

use crate::Config;

use super::{
    AppConfig,
    Bucket, 
    Ticket, 
    Filter, 
    State, 
    Tag,
    AdapterError,
    AdapterErrorType,
    TicketAdapter,
    BucketPanelLocation, 
    BucketPanelLocationType
};

pub type SyncedTicketAdapter = Box<dyn TicketAdapter + Sync + Send>;
pub type AdapterConstructor = fn(Arc<Mutex<AppConfig>>, &Config) -> Result<SyncedTicketAdapter, AdapterError>;
pub type AdapterConfig = fn() -> Config;


pub struct FilterExpression {
    adapter: String,
    expression: String
}

pub struct AdapterType {
    pub name: String,
    pub fancy_name: String,
    new_fn: AdapterConstructor,
    config_fn: AdapterConfig
}

impl AdapterType {
    pub fn new<T: TicketAdapter>() -> Self {
        AdapterType{
            name: T::get_type_name(), 
            fancy_name: T::get_fancy_type_name(), 
            new_fn: T::from_config, 
            config_fn: T::create_config
        }
    }

    pub fn from_config(&self, app_config: Arc<Mutex<AppConfig>>, config: &Config) -> Result<SyncedTicketAdapter, AdapterError> {
        (self.new_fn)(app_config, config)
    }

    pub fn config(&self) -> Config {
        (self.config_fn)()
    }

}

pub struct TicketProvider {
    type_registry: HashMap<String, AdapterType>,
    config: Arc<Mutex<AppConfig>>,
    adapters: Arc<Mutex<Vec<Arc<SyncedTicketAdapter>>>>
}

impl TicketProvider {

    /**
       Setup the Ticket Provider. It is supposed to be a single Provider
       that manages all available Adapters, but i tried to avoid making
       a global instance just for less problems.
     */
    pub fn new(config: Arc<Mutex<AppConfig>>, types_list: Vec<AdapterType>) -> TicketProvider {

        let adapters: Vec<Arc<Box<dyn TicketAdapter + Sync + Send>>> = vec![];
        let mut type_registry: HashMap<String, AdapterType> = HashMap::new();

        for type_entry in types_list {
            type_registry.insert(type_entry.name.clone(), type_entry);
        }

        let ticket_provider = TicketProvider {
            type_registry,
            config,
            adapters: Arc::new(Mutex::new(adapters)),
        };

        ticket_provider.adapters_from_app_config();

        ticket_provider
    }

    pub fn has_adapters(&self) -> bool {
        match self.adapters.lock() {
            Ok(lock) => !lock.is_empty(),
            Err(_) => false,
        }
    }

    /**
       Creates a minimal Configuration, from which custom Adapter Implementation can derive
       from. This Data is needed at a minimum, to make the whole Adapter Detection System work.
     */
    pub fn get_default_config<T: TicketAdapter>() -> Config {
        let mut random_identifier = T::get_type_name() + "_";
        random_identifier += SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string()
            .as_str();

        Config::default()
            .with("type", T::get_type_name(), "readonly_string")
            .with("name", random_identifier, "string")
            .with("display", T::get_fancy_type_name(), "string")
    }

    /**
       Creates a Config of a supplied Type, if the type is in the registry of this Ticket Provider
     */
    pub fn get_type_config(&self, type_name: &String) -> Option<Config> {
        self.type_registry.get(type_name).map(|found_type| found_type.config())
    }

    /**
       Lists all available Adapter Types, that have been registered with this Ticket Provider
     */
    pub fn list_available_adapter_types(&self) -> Vec<&AdapterType> {
        let mut types_vec = Vec::default();

        for type_val in self.type_registry.values() {
            types_vec.push(type_val);
        }

        types_vec
    }

    /**
       Drops an adapter from the enabled Adapters List within this Ticket Adapter
       Returns true, if successfully removed an adapter. If drop_from_config is true,
       the function will also remove any left over configuration from the adapter,
       when removing the adapter itself was successfull.
     */
    pub fn drop_adapter(&self, adapter_name: String, drop_from_config: bool) -> bool {
        match self.adapters.lock() {
            Ok(mut lock) => {

                let mut found_index: Option<usize> = None;

                for adapter in lock.iter().enumerate() {
                    if adapter.1.get_name() == adapter_name {
                        found_index = Some(adapter.0);
                        break;
                    }
                }

                if let Some(index) = found_index {
                    lock.remove(index);
                    if drop_from_config {
                        match self.config.lock() {
                            Ok(mut lock) => {
                                let corrected_name = "adapters:".to_string() + adapter_name.as_str();
                                lock.drop_sub_config(corrected_name.as_str());

                                if let Some(mut adapters_vec) = lock.get_or_default("adapters", "", "").get::<Vec<String>>() {

                                    let mut found_index: Option<usize> = None;
                                    for adapter_entry in adapters_vec.iter().enumerate() {
                                        if adapter_name == *adapter_entry.1 {
                                            found_index = Some(adapter_entry.0);
                                        }
                                    }

                                    if let Some(found_index) = found_index {
                                        adapters_vec.remove(found_index);
                                        lock.put("adapters", adapters_vec, "");
                                    };
                                }
                            },
                            Err(err) => {
                                println!("Unable to lock App Config due to {err}");
                            },
                        }
                    }
                    true
                } else {
                    false
                }

            },
            Err(err) => {
                println!("Unable to lock Adapters List due to {err}");
                false
            },
        }
    }

    /**
       Tries to create an Adapter from Config and adds it to the adapters list
       within this Ticket Provider. If something fials creating the Adapter,
       it will create an AdapterError.
       If it was successfull and write_to_app_config is true, then it will write
       the supplied config into the app config
     */
    pub fn adapter_from_config(&self, config: &Config, write_to_app_config: bool) -> Result<(), AdapterError> {
        let type_name = match config.get("type") {
            Some(option) => match option.get::<String>() {
                Some(type_name) => type_name,
                None => {
                    println!("Given Adapter Config has no valid type!");
                    return Err(AdapterError::new(AdapterErrorType::Instantiation));
                },
            },
            None => {
                println!("Given Adapter Config has no valid type!");
                return Err(AdapterError::new(AdapterErrorType::Instantiation));
            },
        };

        let constructor = match self.type_registry.get(&type_name) {
            Some(found_type) => {
                found_type.new_fn
            },
            None => {
                println!("Adapter type \"{type_name}\" does not exist!");
                return Err(AdapterError::new(AdapterErrorType::Instantiation));
            },
        };

        match constructor(self.config.clone(), config) {
            Ok(adapter) => {

                let mut adapters = match self.adapters.lock() {
                    Ok(lock) => lock,
                    Err(err) => {
                        println!("Unable to lock Adapters List due to {err}");
                        return Err(AdapterError::new(AdapterErrorType::Access));
                    },
                };
                let adapter_name = adapter.get_name();
                adapters.push(Arc::new(adapter));

                if write_to_app_config {
                    match self.config.lock() {
                        Ok(mut config_lock) => {
                            config_lock.put_sub_config(config, ["adapters", adapter_name.as_str()].join(":").as_str());
                            let mut old_adapters_list = config_lock.get_or_default("adapters", "", "").raw().clone();

                            if old_adapters_list.is_empty() {
                                old_adapters_list += adapter_name.as_str();
                            } else {
                                old_adapters_list += "|||";
                                old_adapters_list += adapter_name.as_str();
                            }

                            config_lock.put("adapters", old_adapters_list, "");
                        },
                        Err(err) => {
                            println!("Unable to lock Config due to {err}");
                            return Err(AdapterError::new(AdapterErrorType::Access));
                        },
                    }
                }

                Ok(())
            },
            Err(err) => {
                println!("Found Adapter type \"{type_name}\", but there was a problem instantiating it. Maybe a broken Config?");
                Err(err)
            },
        }
    }


    /**
       Clears the currently installed Adapters and reloads them from the 
       config, that has been supplied on instantiation of the Ticket provider.
     */
    pub fn adapters_from_app_config(&self) {

        match self.adapters.lock() {
            Ok(mut lock) => lock.clear(),
            Err(err) => {
                println!("Unable to lock Adapters List due to {err}");
                return;
            },
        };

        let mut config = match self.config.lock() {
            Ok(lock) => lock,
            Err(err) => {
                println!("Unable to lock Config due to {err}");
                return;
            },
        };

        let adapters_vec: Vec<String> = match config.get_or_default("adapters", Vec::<String>::default(), "").get() {
            Some(adapters_vec) => adapters_vec,
            None => {
                println!("No Adapters Vector in Config found, or invalid format.");
                return;
            },
        };

        for adapter_name in adapters_vec {

            if adapter_name.is_empty() {
                println!("Adapter name is empty, ignoring.");
                continue;
            }

            let adapter_config = config.get_sub_config(["adapters", adapter_name.as_str()].join(":").as_str());

            match self.adapter_from_config(&adapter_config, false) {
                Ok(_) => (),
                Err(err) => println!("Failed creating {adapter_name}. Reason: {err}"),
            };
        };
    }

    /**
       Adds an Adapter for providing tickets and other things to the ticket management tool.
       Be aware that this will consume the Adapter
     */
    pub fn add_adapter(&mut self, adapter: Arc<Box<dyn TicketAdapter + Sync + Send>>) {
        match self.adapters.lock() {
            Ok(mut lock) => {
                lock.push(adapter);
            },
            Err(err) => println!("Wasn't able to add the \"{}\" adapter to the System due to {}", adapter.get_name(), err)
        }
    }

    /**
       Returns a list of fancy names from all added adapters as a vector
     */
    pub fn list_fancy_adapter_names(&self) -> Vec<String> {
        match self.adapters.lock() {
            Ok(lock) => {
                let mut found_names: Vec<String> = vec![];
                for adapter in lock.iter() {
                    found_names.push(adapter.get_fancy_name())
                }
                found_names
            },
            Err(_) => {
                println!("Can't lock adapters for listing adapter Names.");
                vec![]
            }
        }
    } 

    /**
     * Returns a list of the internal names of the available adapters as a vector
     */
    pub fn list_adapter_names(&self) -> Vec<String> {
        match self.adapters.lock() {
            Ok(lock) => {
                let mut found_names: Vec<String> = vec![];
                for adapter in lock.iter() {
                    found_names.push(adapter.get_name())
                }
                found_names
            },
            Err(_) => {
                println!("Can't lock adapters for listing adapter Names.");
                vec![]
            }
        }
    }

    /**
     * Returns the list of name pairs of the available adapters as a vector.
     * The pairs consist of the internal name, followed by the fancy name
     */
    pub fn list_adapter_name_pairs(&self) -> Vec<(String, String)> {
        match self.adapters.lock() {
            Ok(lock) => {
                let mut found_names: Vec<(String, String)> = vec![];
                for adapter in lock.iter() {
                    found_names.push((adapter.get_name(), adapter.get_fancy_name()))
                }
                found_names
            },
            Err(_) => {
                println!("Can't lock adapters for listing adapter Names.");
                vec![]
            }
        }
    }

    /**
     * Returns a list of references to Adapters as a vector
     */
    pub fn list_adapter_refs(&self) -> Vec<Arc<Box<dyn TicketAdapter + Sync + Send>>> {
        let mut ref_list: Vec<Arc<Box<dyn TicketAdapter + Sync + Send>>> = vec![];

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    ref_list.push(adapter.clone());
                }
            },
            Err(err) => println!("Wasn't able to lock adapters list, due to {err}"),
        };

        ref_list
    }

    /**
       Returns a Vector of all Buckets from all Adapters. Doesn't fail.
     */
    pub fn bucket_list_all(&self) -> Vec<Bucket> {

        let mut buckets: Vec<Bucket> = vec![];

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    buckets.append(&mut adapter.bucket_list_all());
                }
            },
            Err(err) => println!("Wasn't able to list buckets from adapters due to {}", err)
        };

        buckets
    }

    /**
       Writes a bucket to it's corresponding adapter, if not readonly
       Throws an error, if the write failed. Other reasons depend on
       the used adapters
     */
    pub fn bucket_write(&self, bucket: &mut Bucket) -> Result<(), AdapterError> {
        let bucket_adapter = bucket.identifier.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name() == bucket_adapter {
                        return adapter.bucket_write(bucket);
                    }
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::BucketWrite))
        }

        Ok(())
    }

    /**
       Check if, the bucket can be inserted as new and is guaranteed
       to keep integrity with it's attributes
     */

    pub fn bucket_validate(&self, bucket: &Bucket) -> Result<(), AdapterError> {
        let mut validation_errors: Vec<(String, String)> = Vec::default();

        if bucket.name.is_empty() {
            validation_errors.push(("name".to_string(), "The name of the Bucket is not supposed to be empty!".to_string()));
        };

        match self.adapters.lock() {
            Ok(lock) => {
                let mut adapter_matched = false;

                for adapter in lock.iter() {
                    if adapter.get_name() == bucket.identifier.adapter {
                        adapter_matched = true;

                        let buckets = adapter.bucket_list_all();
                        let mut bucket_matched = false;
                        for found_bucket in buckets {
                            if found_bucket.name == bucket.name {
                                bucket_matched = true;
                                break;
                            }
                        }

                        if bucket_matched {
                            validation_errors.push(("name".to_string(), format!("The Bucket with name {} does already exist in this adapter.", bucket.name)));
                        }

                        break;
                    }
                };

                if !adapter_matched {
                    let adapter = &bucket.identifier.adapter;
                    validation_errors.push(("adapter".to_string(), format!("The Adapter \"{adapter}\" does not match any of the available adapters.")));
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::Access))
        };

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(AdapterError::new(AdapterErrorType::Validate(validation_errors, "Bucket".to_string())))
        }
    }

    /**
       Checks, if the ticket can be inserted as new and is guaranteed
       to keep integrity with it's attributes
     */
    pub fn ticket_validate(&self, ticket: &Ticket) -> Result<(), AdapterError> {
        let mut validation_errors: Vec<(String, String)> = Vec::default();
        
        if ticket.title.is_empty() {
            validation_errors.push(("title".to_string(), "The title of the Ticket is not supposed to be empty!".to_string()));
        };

        match self.adapters.lock() {
            Ok(lock) => {
                let mut adapter_matched = false;

                for adapter in lock.iter() {
                    if adapter.get_name() == ticket.adapter {
                        adapter_matched = true;

                        let buckets = adapter.bucket_list_all();
                        let mut bucket_matched = false;
                        for bucket in buckets {
                            if bucket.identifier.id == ticket.bucket_id {
                                bucket_matched = true;
                                break;
                            }
                        }

                        if !bucket_matched {
                            let bucket = ticket.bucket_id;
                            validation_errors.push(("bucket".to_string(), format!("The Bucket with id {bucket} does not exist in this adapter.")));
                        }

                        let states = adapter.state_list_all();
                        let mut state_matched = false;

                        for state in states {
                            if state.identifier.name == ticket.state_name {
                                state_matched = true;
                                break;
                            }
                        }

                        if !state_matched {
                            let state = &ticket.state_name;
                            validation_errors.push(("state".to_string(), format!("The State \"{state}\" does not exist in this adapter.")));
                        }

                        break;
                    }
                };

                if !adapter_matched {
                    let adapter = &ticket.adapter;
                    validation_errors.push(("adapter".to_string(), format!("The Adapter \"{adapter}\" does not match any of the available adapters.")));
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::Access))
        };

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(AdapterError::new(AdapterErrorType::Validate(validation_errors, "Ticket".to_string())))
        }
    }

    /**
       Check if the State can be inserted as new and keeps integrity with it's attributes
     */
    pub fn state_validate(&self, state: &State) -> Result<(), AdapterError> {
        let mut validation_errors: Vec<(String, String)> = Vec::default();

        if state.identifier.name.is_empty() {
            validation_errors.push(("name".to_string(), "The Name of the State is not supposed to be empty!".to_string()));
        };

        if state.description.len() > 1000 {
            validation_errors.push(("description".to_string(), "The arbitrary Limit of the State's Description of 1k chars is reached. Make sure the description is as short and concise as possible.".to_string()));
        };

        match self.adapters.lock() {
            Ok(lock) => {
                let mut adapter_matched = false;

                for adapter in lock.iter() {
                    if adapter.get_name() == state.identifier.adapter {
                        adapter_matched = true;
                    };
                };

                if !adapter_matched {
                    let adapter = &state.identifier.adapter;
                    validation_errors.push(("adapter".to_string(), format!("The Adapter \"{adapter}\" does not match any of the available adapters.")));
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::Access))
        };

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(AdapterError::new(AdapterErrorType::Validate(validation_errors, "State".to_string())))
        }
    }

    /**
       Checks, if the Tag can be inserted as new and is guaranteed to keep
       integrity with it's attributes
     */
    pub fn tag_validate(&self, tag: &Tag) -> Result<(), AdapterError> {
        let mut validation_errors: Vec<(String, String)> = Vec::default();
        
        if tag.name.is_empty() {
            validation_errors.push(("name".to_string(), "The Name of the Tag is not supposed to be empty!".to_string()));
        };

        if tag.color.to_ascii_lowercase() == tag.color_text.to_ascii_lowercase() {
            validation_errors.push(("color".to_string(), "The colors of the font and the background are the same!".to_string()));
        }

        match self.adapters.lock() {
            Ok(lock) => {
                let mut adapter_matched = false;

                for adapter in lock.iter() {
                    if adapter.get_name() == tag.adapter {
                        adapter_matched = true;
                    };
                };

                if !adapter_matched {
                    let adapter = &tag.adapter;
                    validation_errors.push(("adapter".to_string(), format!("The Adapter \"{adapter}\" does not match any of the available adapters.")));
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::Access))
        };

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(AdapterError::new(AdapterErrorType::Validate(validation_errors, "Tag".to_string())))
        }
    }

    /**
       Lists all Tickets from all Adapters. Returns a vector of all 
       found Tickets. Doesn't fail.
     */
    pub fn ticket_list_all(&self) -> Vec<Ticket> {
        let mut tickets: Vec<Ticket> = vec![];

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    tickets.append(&mut adapter.ticket_list_all());
                }
            },
            Err(err) => println!("Wasn't able to list states from adapters due to {}", err)
        };

        tickets
    }

    /**
       Lists a singular Ticket. Not recommended to be used in Loops, it exists to get
       the most up to date version of a ticket, to make changes on.
     */
    pub fn ticket_list_unique(&self, id: i64, adapter_name: &String) -> Option<Ticket> {
        let mut ticket: Option<Ticket> = None;

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name().eq(adapter_name) {

                        ticket = adapter.ticket_list_unique(id);
                    }
                }
            },
            Err(err) => println!("Wasn't able to list tickets from adapters due to {}", err)
        };

        ticket
    }

    /**
       Lists a singular Bucket. Not recommended to be used in Loops, it exists to get
       the most up to date version of a bucket, to make changes on.
     */
    pub fn bucket_list_unique(&self, id: i64, adapter_name: &String) -> Option<Bucket> {
        let mut bucket: Option<Bucket> = None;

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name().eq(adapter_name) {

                        bucket = adapter.bucket_list_unique(id);
                    }
                }
            },
            Err(err) => println!("Wasn't able to list buckets from adapters due to {}", err)
        };

        bucket
    }

    /**
       Lists all Tickets from one adapter
     */
    pub fn tickets_list_adapter(&self, adapter_name: &String) -> Vec<Ticket> {
        let mut tickets: Vec<Ticket> = vec![];

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name().eq(adapter_name) {
                        tickets.append(&mut adapter.ticket_list_all());
                    }
                }
            },
            Err(err) => println!("Wasn't able to list states from adapters due to {}", err)
        };

        tickets
    }

    pub fn split_filter_expression(&self, filter_expression: String) -> Result<Vec<FilterExpression>, AdapterError> {

        let mut expression = filter_expression.as_str().trim();

        // Vector of Tuples with Adapter Name + Expression
        let mut found_expressions: Vec<FilterExpression> = vec![];

        while expression.contains("[[") {

            // Split off the beginning
            let cleaned_expression_beginning = expression.split_at(2).1;

            // Split off the end and write advanced String to iterated expression
            let expression_result = match cleaned_expression_beginning.find("]]") {
                Some(found_pos) => {
                    let (cleaned_expression, rest_of_string) = cleaned_expression_beginning.split_at(found_pos);
                    let leftover_expression = rest_of_string.split_at(2).1;
                    expression = leftover_expression.trim();
                    Ok(cleaned_expression)
                },
                None => Err(AdapterError{ error_type: AdapterErrorType::Expression("Expression doesn't end with ]]".to_string()) })
            };

            // If successfully found content between brackets, isolate parameters,
            // Otherwise return with error
            if let Ok(cleaned_expression) = expression_result {
                let filter_expression = match cleaned_expression.find(':') {
                    Some(found_pos) => {
                        let (adapter_name, adapter_op) = cleaned_expression.split_at(found_pos);
                        Ok(FilterExpression{
                            adapter: String::from(adapter_name.trim()), 
                            expression: String::from(adapter_op.split_at(1).1.trim()) //remove the :
                        })
                    },
                    None => Err(AdapterError{ error_type: AdapterErrorType::Expression("Expression doesn't follow adapter:expression between the square brackets".to_string()) })
                };

                if let Ok(isolated_expr) = filter_expression {
                    found_expressions.push(isolated_expr);
                } else {
                    println!("Expression adapter and operation is not split off with : !");
                    return Err(filter_expression.err().unwrap());
                }
                
            } else {
                println!("Expression needs to end with ]] !");
                return Err(expression_result.err().unwrap());
            }
        };

        // If the found expressions list is empty, error out, because there was 
        // either nothing or a wrongly formatted expression in the filter string
        if found_expressions.is_empty() {
            println!("Expression needs start with [[ ! Is the expression empty?");
            return Err(AdapterError{ error_type: AdapterErrorType::Expression("Expression needs start with [[ ! Is the expression empty?".to_string()) });
        }

        Ok(found_expressions)
    }


    /**
       Lists all Tickets, that satisfy expressions specific to the adapters
       a valid filter can have the form ```[[adapter: expression]]```.
       You can even chain mutliple filter after eachother to collect the
       results from multiple adapters such as ```[[ad1: expr1]][[ad2: expr2]]...```
       the adapter in this case is the name of the adapter (not the fancy name)
       and the expression is specific to the adapter, so you have to look
       it up on the adapters documentation.
     */
    pub fn ticket_list(&self, filter: &Filter) -> Result<Vec<Ticket>, AdapterError> {

        let found_expressions = match self.split_filter_expression(filter.operation.clone()) {
            Ok(expressions) => expressions,
            Err(err) => return Err(err),
        };
        
        let mut tickets: Vec<Ticket> = vec![];

        // Find adapters with corresponding name and execute operation on them.
        // then collect their results into final list
        let local_adapters = self.adapters.clone();
        if let Ok(lock) = local_adapters.lock() {
            
            for found_expression in found_expressions {
                let expression_adapter = found_expression.adapter.clone();
                let found_adapter = lock.iter().by_ref().into_iter().find(move |adapter| adapter.get_name() == expression_adapter);
                if let Some(adapter) = found_adapter {
                    if let Err(error) = match adapter.ticket_list(&found_expression.expression) {
                        Ok(mut new_tickets) => {
                            tickets.append(&mut new_tickets);
                            Ok(())
                        },
                        Err(err) => Err(err),
                    } {
                        println!("Adapter \"{}\" failed to evaluate expression!", found_expression.adapter);
                        return Err(error);
                    }

                } else {
                    println!("Adapter \"{}\" has not been found in Adapterlist!", found_expression.adapter);
                    return Err(AdapterError{ error_type: AdapterErrorType::Expression(format!("Adapter \"{}\" has not been found in Adapterlist!", found_expression.adapter)) });
                }
            }

        } else {
            println!("Wasn't able to Lock Adapterlist");
            return Err(AdapterError{ error_type: AdapterErrorType::TicketWrite });
        }

        // If everything went fine, return the final ticket List
        Ok(tickets)

    }

    /**
       Write a given Ticket to it's corresponding Adapter. Throws an 
       Error, if the write failed. Other reasons depend on used adapters.
     */
    pub fn ticket_write(&self, ticket: &Ticket) -> Result<(), AdapterError> {
        let ticket_adapter = ticket.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name() == ticket_adapter {
                        return adapter.ticket_write(ticket);
                    }
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::TicketWrite))
        }

        Ok(())
    }

    /**
       Tries to delete a given Ticket from it's corresponding Adapter. Throws
       an error, if the delete fails for whatever reason.
     */
    pub fn ticket_drop(&self, ticket: &Ticket) -> Result<(), AdapterError> {
        let ticket_adapter = ticket.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name() == ticket_adapter {
                        return adapter.ticket_drop(ticket);
                    }
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::TicketDelete))
        }

        Ok(())
    }

    /**
       Lists all states from all Adapters. Returns a vector of all
       found states. Doesn't fail.
     */
    pub fn state_list_all(&self) -> Vec<State> {
        let mut states: Vec<State> = vec![];

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    states.append(&mut adapter.state_list_all());
                }
            },
            Err(err) => println!("Wasn't able to list states from adapters due to {}", err)
        };

        states
    }

    /**
       Write a state to it's corresponding Adapter. If the write goes wrong,
       an Error gets thrown. Other reasons of an error can be found at the
       corresponding adapter documentation
     */
    pub fn state_write(&self, state: &State) -> Result<(), AdapterError> {
        let state_adapter = state.identifier.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name() == state_adapter {
                        return adapter.state_write(state);
                    }
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::StateWrite))
        }

        Ok(())
    }

    /**
       Lists all available tags from all Adapters. Can't fail, gives an empty
       vector instead.
     */
    pub fn tag_list_all(&self) -> Vec<Tag> {
        let mut tags: Vec<Tag> = vec![];

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    tags.append(&mut adapter.tag_list_all());
                }
            },
            Err(err) => println!("Wasn't able to list tags from adapters due to {}", err)
        };

        tags
    }

    /**
       Writes a given Tag to it's corresponding Adapter. If the write fails,
       it throws an Error. Other reasons might be available on the Adapters
       documentation
     */
    pub fn tag_write(&self, tag: &Tag) -> Result<(), AdapterError> {
        let tag_adapter = tag.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name() == tag_adapter {
                        return adapter.tag_write(tag);
                    }
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::TagWrite))
        }

        Ok(())
    }

    /**
       Tries to delete a given Tag from it's corresponding Adapter. Throws
       an error, if the delete fails for whatever reason.
     */
    pub fn tag_drop(&self, tag: &Tag) -> Result<(), AdapterError> {
        let tag_adapter = tag.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name() == tag_adapter {
                        return adapter.tag_drop(tag);
                    }
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::TagDelete))
        }

        Ok(())
    }

    /**
       Tries to delete a given Bucket from it's corresponding Adapter. Throws
       an error, if the delete fails for whatever reason.
     */
    pub fn bucket_drop(&self, bucket: &Bucket) -> Result<(), AdapterError> {
        let bucket_adapter = bucket.identifier.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name() == bucket_adapter {
                        return adapter.bucket_drop(bucket);
                    }
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::BucketDelete))
        }

        Ok(())
    }

    /**
       Lists all available filters in this adapter as a vector,
       returns an empty vector, if no filters have been found
     */
    pub fn filter_list_all(&self) -> Vec<Filter> {
        let mut filters: Vec<Filter> = vec![];

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    filters.append(&mut adapter.filter_list_all());
                }
            },
            Err(err) => println!("Wasn't able to list filters from adapters due to {}", err)
        };

        filters
    }

    /**
       Tries to delete a given Filter from it's corresponding Adapter. Throws
       an error, if the delete fails for whatever reason.
     */
    pub fn filter_drop(&self, filter: &Filter) -> Result<(), AdapterError> {
        let adapter_name = filter.identifier.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                if let Some(adapter) = lock.iter().find(|adapter| {
                    adapter.get_name() == adapter_name
                }) {
                    return adapter.filter_drop(filter);
                } else {
                    return Err(AdapterError::new(AdapterErrorType::FilterDelete))
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::FilterDelete))
        }

        Ok(())
    }

    pub fn ticket_list_from_selection(&self, locations: &Vec<BucketPanelLocation>) -> Option<Vec<Ticket>> {

        // If there are no locations in there, don't do anything
        if locations.is_empty() {
            return None;
        }

        // Gather all Filters first
        let mut filters: Vec<&String> = vec![];
        let mut tickets: Vec<Ticket> = vec![];

        for location in locations {
            match location.entry_type {
                BucketPanelLocationType::Reset => break,
                BucketPanelLocationType::All => {
                    tickets.append(&mut self.ticket_list_all());
                    break
                },
                BucketPanelLocationType::Adapter => {
                    tickets.append(&mut self.tickets_list_adapter(&location.adapter))
                },
                BucketPanelLocationType::Entry => {
                    if !locations.iter().any(|loc| {
                        loc.entry_type == BucketPanelLocationType::Adapter &&
                        loc.adapter == location.adapter
                    }) {
                        filters.push(&location.section);
                    }
                },
                BucketPanelLocationType::Filter => {
                    filters.push(&location.section)
                },
            };
        }

        // Execute all Filters
        let concrete_filters = self.filter_list_by_name(filters);
        for filter in concrete_filters {
            match self.ticket_list(&filter) {
                Ok(mut filtered_tickets) => tickets.append(&mut filtered_tickets),
                Err(err) => println!("Failed to apply Filter. reason: {}", err),
            }
        }
        
        tickets.sort_by(|first, second| first.id.cmp(&second.id));

        // Remove duplicate Tickets
        tickets.dedup_by(|first, second| {
            first.adapter == second.adapter &&
            first.id == second.id
        });

        Some(tickets)
    }

    /*
       Get all filters from the supplied names, that have the same name as the 
       supplied strings. If no Filter is found, an empty vector is returned.
     */
    pub fn filter_list_by_name(&self, filter_names: Vec<&String>) -> Vec<Filter> {
        // TODO: optimize this function
        let filters = self.filter_list_all();
        let mut matching: Vec<Filter> = vec![];

        for filter in filters {
            if filter_names.contains(&&filter.identifier.name) {
                matching.push(filter);
            };
        };

        matching
    }

    /**
       Tries to get a Filter with a specific name in a specific Adapter. If no
       Filter with these attributes is found, None is returned
     */
    pub fn filter_list_unique(&self, filter_name: &str, adapter_name: &String) -> Option<Filter> {
        let mut filter: Option<Filter> = None;

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name().eq(adapter_name) {
                        filter = adapter.filter_list(filter_name.to_string())
                    }
                }
            },
            Err(err) => println!("Wasn't able to list filters from adapters due to {}", err)
        };

        filter
    }

    pub fn filter_validate(&self, filter: &Filter) -> Result<(), AdapterError> {
        let mut validation_errors: Vec<(String, String)> = Vec::default();

        if filter.identifier.name.is_empty() {
            validation_errors.push(("name".to_string(), "The Name of the Filter is not supposed to be empty!".to_string()));
        }

        if match filter.filter_type {
            crate::FilterType::User => false,
            crate::FilterType::Builtin => true,
            crate::FilterType::Bucket(_) => true,
            crate::FilterType::Tag => true,
            crate::FilterType::Other => false,
        } {
            validation_errors.push(("filter_type".to_string(), "The Type of the Filter does not allow it to be modified".to_string()));
        }

        let found_expressions = match self.split_filter_expression(filter.operation.clone()) {
            Ok(found_expressions) => found_expressions,
            Err(err) => {
                validation_errors.push(("operation".to_string(), err.get_text()));
                vec![]
            }
        };

        match self.adapters.lock() {
            Ok(adapters_lock) => {



                for (index, expression) in found_expressions.iter().enumerate() {

                    let mut adapter_matched = false;

                    for adapter in adapters_lock.iter() {
                        if adapter.get_name() == filter.identifier.adapter {
                            adapter_matched = true;
                            break;

                        };
                    };

                    if !adapter_matched {
                        let adapter = &filter.identifier.adapter;
                        validation_errors.push(("adapter".to_string(), format!("The Adapter \"{adapter}\" does not match any of the available adapters.")));
                    }

                    for adapter in adapters_lock.iter() {
                        if adapter.get_name() == expression.adapter {
    
                            //validate filter via the adapter aswell
                            if let Err(mut error) = adapter.filter_expression_validate(&expression.expression) {
                                validation_errors.push(("operation".to_string(), format!("Error in filter expression No {}, ({})", index + 1, &expression.expression)));
                                validation_errors.append(&mut error);
                            };
    
                        };
                    };
                }
            },
            Err(err) => {
                validation_errors.push(("operation".to_string(), format!("Wasn't able to lock Adapters due to {err}")));
            }
        }

        if validation_errors.is_empty() {
            Ok(())
        } else {
            Err(AdapterError::new(AdapterErrorType::Validate(validation_errors, "Filter".to_string())))
        }
    }

    /*
       Attempts to write a filter to the adapter. If there is an Error,
       an AdapterError gets thrown.
     */
    pub fn filter_write(&self, filter: &Filter) -> Result<(), AdapterError> {
        let filter_adapter = filter.identifier.adapter.clone();

        match self.adapters.lock() {
            Ok(lock) => {
                for adapter in lock.iter() {
                    if adapter.get_name() == filter_adapter {
                        return adapter.filter_write(filter);
                    }
                }
            },
            Err(_) => return Err(AdapterError::new(AdapterErrorType::FilterWrite))
        }

        Ok(())
    }



}