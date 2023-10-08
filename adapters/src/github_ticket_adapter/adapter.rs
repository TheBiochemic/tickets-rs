use std::{sync::{Arc, Mutex}, time::{Instant, Duration}};

use octocrab::Octocrab;
use tickets_rs_core::{TicketAdapter, TicketProvider, AppConfig, Config, AdapterError, AdapterErrorType, Filter};

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

    fn from_config(app_config: Arc<Mutex<AppConfig>>, config: &Config) -> Result<Box<dyn TicketAdapter + Send + Sync>, AdapterError> where Self: Sized {
        

        let octocrab = match config.get("personal_auth_token") {
            Some(config_option) => {

                match config_option.get::<String>() {
                    Some(config_string) => {
                        if !config_string.is_empty() {
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
            last_refresh: instant,
            owner,
        };

        adapter.refresh_data();

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
        self.cached_buckets.values().cloned().collect()
    }

    fn bucket_list_unique(&self, id: u64) -> Option<tickets_rs_core::Bucket> {
        self.cached_buckets.get(&id).cloned()
    }

    fn bucket_drop(&self, bucket: &tickets_rs_core::Bucket) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError::new(tickets_rs_core::AdapterErrorType::BucketDelete)) //TODO!
    }

    fn bucket_write(&self, bucket: &mut tickets_rs_core::Bucket) -> Result<(), tickets_rs_core::AdapterError> {
        Err(tickets_rs_core::AdapterError::new(tickets_rs_core::AdapterErrorType::BucketWrite)) //TODO!
    }

    fn ticket_list_all(&self) -> Vec<tickets_rs_core::Ticket> {
        todo!()
    }

    fn ticket_list_unique(&self, id: i64) -> Option<tickets_rs_core::Ticket> {
        todo!()
    }

    fn ticket_list(&self, expression: &str) -> Result<Vec<tickets_rs_core::Ticket>, tickets_rs_core::AdapterError> {
        todo!()
    }

    fn ticket_write(&self, ticket: &tickets_rs_core::Ticket) -> Result<(), tickets_rs_core::AdapterError> {
        todo!()
    }

    fn ticket_drop(&self, ticket: &tickets_rs_core::Ticket) -> Result<(), tickets_rs_core::AdapterError> {
        todo!()
    }

    fn state_list_all(&self) -> Vec<tickets_rs_core::State> {
        vec![] //TODO"
    }

    fn state_write(&self, state: &tickets_rs_core::State) -> Result<(), tickets_rs_core::AdapterError> {
        todo!()
    }

    fn tag_list_all(&self) -> Vec<tickets_rs_core::Tag> {
        vec![] //TODO"
    }

    fn tag_write(&self, state: &tickets_rs_core::Tag) -> Result<(), tickets_rs_core::AdapterError> {
        todo!()
    }

    fn tag_drop(&self, state: &tickets_rs_core::Tag) -> Result<(), tickets_rs_core::AdapterError> {
        todo!()
    }

    fn filter_list_all(&self) -> Vec<tickets_rs_core::Filter> {

        let mut filters: Vec<Filter> = Vec::new();

        filters.append(&mut self.list_builtin_filters());

        filters
    }

    fn filter_list(&self, filter_name: String) -> Option<tickets_rs_core::Filter> {
        todo!()
    }

    fn filter_write(&self, filter: &tickets_rs_core::Filter) -> Result<(), tickets_rs_core::AdapterError> {
        todo!()
    }

    fn filter_drop(&self, filter: &tickets_rs_core::Filter) -> Result<(), tickets_rs_core::AdapterError> {
        todo!()
    }

    fn filter_expression_validate(&self, expression: &String) -> Result<(), Vec<(String, String)>> {
        todo!()
    }

}