mod adapter;
use std::{collections::BTreeMap, sync::{Arc, Mutex}, time::{Instant, Duration}};

pub use adapter::*;
use octocrab::Octocrab;
use tickets_rs_core::{AppConfig, Ticket, Bucket, BucketIdentifier, Filter, FilterType, TicketAdapter, Tag, State};

pub struct GithubTicketAdapter {
    name: String,
    display_name: String,
    config: Arc<Mutex<AppConfig>>,
    cached_tickets: BTreeMap<u64, Ticket>,
    cached_buckets: BTreeMap<u64, Bucket>,
    cached_tags: BTreeMap<String, Tag>,
    cached_states: BTreeMap<String, State>,
    octocrab: Arc<Octocrab>,
    last_refresh: Instant,
    owner: String,
}

impl GithubTicketAdapter {
    pub(crate) fn refresh_data(&mut self) {

        // Try to limit updates, so that the API is not getting spammed all the time
        if self.last_refresh.elapsed() < Duration::from_secs(5 * 60) {
            return;
        }

        self.last_refresh = Instant::now();

        let users_request = self.octocrab.users(self.owner.clone());
        let repos_result = futures::executor::block_on(users_request.repos().per_page(100).send());

        match repos_result {
            Ok(repos) => {
                self.cached_buckets.clear();

                for repo in repos.items {

                    self.cached_buckets.insert(repo.id.0, 
                    Bucket::default()
                            .with_details(repo.id.0, repo.name)
                            .with_adapter(self)
                    );

                    
                };

                
            },
            Err(err) => {
                println!("{}", err);
                return;
            }
        }

    }

    fn list_builtin_filters(&self) -> Vec<Filter> {

        let buckets = self.bucket_list_all();

        buckets.iter().map(|bucket| {
            Filter::default()
                .with_details(
                    bucket.name.clone(), 
                    Filter::filter_expression(self.get_name(), &format!("in_bucket({})", bucket.name)))
                .with_type(FilterType::Bucket(bucket.identifier.id))
                .with_adapter(self)
        }).collect::<Vec<Filter>>()
    }
}