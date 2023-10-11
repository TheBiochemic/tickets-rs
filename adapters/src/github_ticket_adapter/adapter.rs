use std::{sync::{Arc, Mutex}, time::{Instant, Duration, SystemTime}, collections::BTreeMap, thread};

use octocrab::{Octocrab, models};
use tickets_rs_core::{TicketAdapter, TicketProvider, AppConfig, Config, AdapterError, AdapterErrorType, Filter, Ticket};
use tokio::runtime::Handle;

use crate::GithubTicketAdapter;


impl TicketAdapter for GithubTicketAdapter {

    fn get_type_name() -> String where Self: Sized {
        "github".to_string()
    }

    fn get_fancy_type_name() -> String where Self: Sized {
        "Github Issues".to_string()
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn create_config() -> tickets_rs_core::Config where Self: Sized {
        TicketProvider::get_default_config::<Self>()
            .with("personal_auth_token", "", "string")
            .with("repo_owner", "", "string")
    }

    fn from_config(app_config: Arc<Mutex<AppConfig>>, config: &Config, finished: Arc<Mutex<bool>>) -> Result<Box<dyn TicketAdapter + Send + Sync>, AdapterError> where Self: Sized {
        
        let mut auth_token = "".to_string();

        let octocrab = match config.get("personal_auth_token") {
            Some(config_option) => {

                match config_option.get::<String>() {
                    Some(config_string) => {
                        if !config_string.is_empty() {
                            auth_token = config_string.clone();
                            match Octocrab::builder().personal_token(config_string).build() {
                                Ok(instance) => {
                                    Arc::new(instance)
                                },
                                Err(err) => {
                                    println!("{}", err);
                                    return Err(AdapterError { error_type: AdapterErrorType::Instantiation })
                                },
                            }
                        } else {
                            octocrab::instance()
                        }
                    },
                    None => {
                        octocrab::instance()
                    },
                }

            },
            None => {
                octocrab::instance()
            },
        };

        let owner: String = match config.get("repo_owner") {
            Some(option) => match option.get() {
                Some(result) => result,
                None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
            },
            None => return Err(AdapterError::new(AdapterErrorType::Instantiation)),
        };

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

        let instant = match Instant::now().checked_sub(Duration::from_secs(600)) {
            Some(instant) => instant,
            None => Instant::now(),
        };

        let mut adapter = GithubTicketAdapter{
            name,
            display_name,
            config: app_config,
            cached_tickets: Default::default(),
            cached_buckets: Default::default(),
            cached_tags: Default::default(),
            cached_states: Default::default(),
            octocrab,
            auth_token,
            last_refresh: instant,
            owner,
            update_trigger: Arc::new(Mutex::new(false))
        };

        adapter.full_refresh_data(finished);

        Ok(Box::new(adapter))

    }

    fn get_icon(&self) -> &std::path::Path {
        std::path::Path::new("assets/adapters/github.png")
    }

    fn get_fancy_name(&self) -> String {
        self.display_name.clone()
    }

    fn is_read_only(&self) -> bool {
        true //TODO!
    }

    fn bucket_list_all(&self) -> Vec<tickets_rs_core::Bucket> {
        if let Ok(lock) = self.cached_buckets.lock() {
            return lock.values().cloned().collect();
        };

        vec![]
    }

    fn bucket_list_unique(&self, id: u64) -> Option<tickets_rs_core::Bucket> {
        if let Ok(lock) = self.cached_buckets.lock() {
            return lock.get(&id).cloned();
        }

        None
    }

    fn bucket_drop(&self, bucket: &tickets_rs_core::Bucket) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError::new(tickets_rs_core::AdapterErrorType::BucketDelete)) //TODO!
    }

    fn bucket_write(&self, bucket: &mut tickets_rs_core::Bucket) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError::new(tickets_rs_core::AdapterErrorType::BucketWrite)) //TODO!
    }

    fn ticket_list_all(&self) -> Vec<tickets_rs_core::Ticket> {

        let buckets = if let Ok(lock) = self.cached_buckets.lock() {

            lock.values().cloned().collect()

        } else {
            vec![]
        };

        let mut tickets: Vec<tickets_rs_core::Ticket> = vec![];

        for bucket in buckets {
            match self.ticket_list(&Self::filter_expr_from_bucket(&bucket)) {
                Ok(mut new_tickets) => tickets.append(&mut new_tickets),
                Err(_) => (),
            }
        }

        tickets
    }

    fn ticket_list_unique(&self, id: i64) -> Option<tickets_rs_core::Ticket> {
        
        let mut local_ticket_opt = None;

        // Check if the ticket even exists locally
        if let Ok(tickets_lock) = self.cached_tickets.lock() {
            local_ticket_opt = tickets_lock.get(&(id as u64)).cloned()
        }

        // If there was some Ticket found, get the bucket name as repo name and query the ticket
        if let Some(local_ticket) = local_ticket_opt {

            if let Some(local_bucket) = self.bucket_list_unique(local_ticket.bucket_id) {

                let thread_octocrab = self.octocrab.clone();
                let thread_owner = self.owner.clone();
                let thread_id = {
                    let add_id_str = local_ticket.additional_id.clone();
                    //let add_id: &str = &add_id_str;
                    let add_id_opt = add_id_str.split_once("::");
                    let num = add_id_opt.unwrap().0.to_string();
                    num
                };
                let thread_repo = local_bucket.name.clone();
                let thread_repo_id = local_bucket.identifier.clone();
                let thread_ticket_proto = Ticket::default().with_adapter(self);
                let handle = Handle::current();
                let thread_result = thread::spawn(move || {

                    if let Ok(issue) = handle.block_on(thread_octocrab.issues(thread_owner, thread_repo.clone()).get(u64::from_str_radix(&thread_id, 10).unwrap())) {
                        Self::map_issues_to_tickets(vec![issue], thread_ticket_proto, thread_repo_id.id, &thread_repo).pop_first().map(|elem| elem.1)
                    } else {
                        println!("wasnt able to get issue by id {}", thread_id);
                        None
                    }

                }).join();

                if let Ok(thread_inner_data) = thread_result {
                    if let Some(ticket) = &thread_inner_data {

                        if let Ok(mut tickets_lock) = self.cached_tickets.lock() {
                            tickets_lock.insert(ticket.id as u64, ticket.clone());
                        };

                    };

                    thread_inner_data
                } else {
                    println!("thread didnt exit correctly");
                    None
                }
            } else {
                println!("didnt find bucket");
                None
            }
        } else {
            println!("didnt find locally stored ticket");
            None
        }

    }

    fn ticket_list(&self, expression: &str) -> Result<Vec<tickets_rs_core::Ticket>, tickets_rs_core::AdapterError> {

        let split_expression: Vec<&str> = expression.split(" ||| ").collect();
        let repo = split_expression.get(0).unwrap().to_string();
        let id = u64::from_str_radix(split_expression.get(1).unwrap(), 10).unwrap();

        let mut loaded = true;

        if let Ok(mut lock) = self.cached_buckets.lock() {
            if let Some(result) = lock.get_mut(&id) {
                match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                    Ok(duration) => {
                        let last_change_ts = result.last_change as u64;
                        let diff = duration.as_secs() - last_change_ts;

                        if diff > 5 * 60 {
                            loaded = false;
                            result.last_change = duration.as_secs() as i64;
                        }
                    },
                    Err(_err) => (),
                };
            }
        }

        if !loaded {

            let handle = Handle::current();

            let thread_octocrab = self.octocrab.clone();
            let thread_owner = self.owner.clone();
            let thread_ticket_proto = Ticket::default().with_adapter(self);
            let thread_cached_tickets = self.cached_tickets.clone();
            let final_result = thread::spawn(move || {

                let result_page = handle.block_on(thread_octocrab.issues(thread_owner.clone(), repo.clone()).list().state(octocrab::params::State::All).per_page(100).send());
                match result_page {
                    Ok(found_page) => {
                        let issues_result = handle.block_on(thread_octocrab.all_pages::<models::issues::Issue>(found_page));

                        if let Ok(issues) = issues_result {
                            let tickets: BTreeMap<u64, Ticket> = Self::map_issues_to_tickets(issues, thread_ticket_proto, id, &repo);

                            if let Ok(mut lock) = thread_cached_tickets.lock() {
                                lock.retain(|key, ticket| id.ne(&ticket.bucket_id) );
                                lock.append(&mut tickets.clone());
                                let local_tickets: Vec<tickets_rs_core::Ticket> = tickets.into_iter().map(|ticket| ticket.1).collect();
                                return Some(local_tickets)
                            } else {
                                return None
                            }

                        }
                    },
                    Err(_) => (),
                }

                return None


            }).join();

            match final_result {
                Ok(tickets_vec) => {
                    match tickets_vec {
                        Some(inner_vec) => Ok(inner_vec),
                        None => Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::Access}),
                    }
                },
                Err(_) => Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::Access}),
            }

            
        } else {
            if let Ok(lock) = self.cached_tickets.lock() {

                let local_tickets: Vec<tickets_rs_core::Ticket> = lock
                    .iter()
                    .filter(|ticket| {ticket.1.bucket_id.eq(&id)})
                    .map(|ticket| ticket.1)
                    .cloned()
                    .collect();

                return Ok(local_tickets)
            }
            Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::Access})
        }

        
    }

    fn ticket_write(&self, ticket: &tickets_rs_core::Ticket) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::TicketWrite})
    }

    fn ticket_drop(&self, ticket: &tickets_rs_core::Ticket) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::TicketDelete})
    }

    fn state_list_all(&self) -> Vec<tickets_rs_core::State> {
        if let Ok(lock) = self.cached_states.lock() {
            return lock.values().cloned().collect();
        };

        vec![]
    }

    fn state_write(&self, state: &tickets_rs_core::State) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::StateWrite})
    }

    fn tag_list_all(&self) -> Vec<tickets_rs_core::Tag> {
        if let Ok(lock) = self.cached_tags.lock() {
            return lock.values().cloned().collect();
        };

        vec![]
    }

    fn tag_write(&self, state: &tickets_rs_core::Tag) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::TagWrite})
    }

    fn tag_drop(&self, state: &tickets_rs_core::Tag) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::TagDelete})
    }

    fn filter_list_all(&self) -> Vec<tickets_rs_core::Filter> {

        let mut filters: Vec<Filter> = Vec::new();

        filters.append(&mut self.list_builtin_filters());

        filters
    }

    fn filter_list(&self, filter_name: String) -> Option<tickets_rs_core::Filter> {
        None
    }

    fn filter_write(&self, filter: &tickets_rs_core::Filter) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::FilterWrite})
    }

    fn filter_drop(&self, filter: &tickets_rs_core::Filter) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError{error_type: AdapterErrorType::FilterDelete})
    }

    fn filter_expression_validate(&self, expression: &String) -> Result<(), Vec<(String, String)>> {
        let split_expression: Vec<&str> = expression.split(" ||| ").collect();
        if split_expression.len() == 2 {

            match u64::from_str_radix(split_expression.get(1).unwrap(), 10) {
                Ok(_) => Ok(()),
                Err(_) => Err(vec![("operation".to_string(), "Expression needs to be in the form of \"repo_name ||| repo_id\". repo_id needs to be a number".to_string())]),
            }

        } else {
            Err(vec![("operation".to_string(), "Expression needs to be in the form of \"repo_name ||| repo_id\"".to_string())])
        }
    }

}