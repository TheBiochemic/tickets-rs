mod adapter;
use std::{collections::BTreeMap, sync::{Arc, Mutex}, time::{Instant, Duration}, thread};

pub use adapter::*;
use octocrab::{Octocrab, models, Page};
use reqwest::header::{HeaderMap, HeaderValue, self};
use tickets_rs_core::{AppConfig, Ticket, Bucket, BucketIdentifier, Filter, FilterType, TicketAdapter, Tag, State};
use tokio::runtime::Handle;

pub struct GithubTicketAdapter {
    name: String,
    display_name: String,
    config: Arc<Mutex<AppConfig>>,
    cached_tickets: BTreeMap<u64, Ticket>,
    cached_buckets: Arc<Mutex<BTreeMap<u64, Bucket>>>,
    cached_tags: Arc<Mutex<BTreeMap<String, Tag>>>,
    cached_states: Arc<Mutex<BTreeMap<String, State>>>,
    octocrab: Arc<Octocrab>,
    auth_token: String,
    last_refresh: Instant,
    owner: String,
}

impl GithubTicketAdapter {
    pub(crate) fn full_refresh_data(&mut self, update_trigger: Arc<Mutex<bool>>) {

        // Try to limit updates, so that the API is not getting spammed all the time
        if self.last_refresh.elapsed() < Duration::from_secs(5 * 60) {
            return;
        }

        self.last_refresh = Instant::now();

        let thread_buckets = self.cached_buckets.clone();
        let thread_tags = self.cached_tags.clone();
        let thread_octocrab = self.octocrab.clone();
        let thread_owner = self.owner.clone();
        let thread_bucket_proto = Bucket::default().with_adapter(self);
        let thread_tag_proto = Tag::default().with_adapter(self);
        let thread_auth_token = self.auth_token.clone();
        let handle = Handle::current();
        let _ = thread::spawn(move || {

            let users_request = thread_octocrab.users(thread_owner.clone());

            // First get all the repos available in the account
            let repos_page = match handle.block_on(users_request.repos().per_page(100).send()) {
                Ok(page) => page,
                Err(err) => {
                    println!("{}", err);
                    return;
                }
            };
            let repos_result = handle.block_on(thread_octocrab.all_pages::<models::Repository>(repos_page));

            let mut local_cached_buckets: BTreeMap<u64, Bucket> = BTreeMap::default();
            let mut local_cached_tags: BTreeMap<String, Tag> = BTreeMap::default();

            match repos_result {
                Ok(repos) => {

                    for repo in repos {

                        local_cached_buckets.insert(repo.id.0, 
                        thread_bucket_proto.clone()
                                .with_details(repo.id.0, repo.name)
                        );

                        
                    };

                    
                },
                Err(err) => {
                    println!("{}", err);
                    return;
                }
            }

            match thread_buckets.lock() {
                Ok(mut lock) => {
                    lock.clear();
                    lock.append(&mut local_cached_buckets.clone());
                },
                Err(_) => (),
            }

            // Now get all labels and map them to tags
            for buckets in local_cached_buckets {

                let mut headers = HeaderMap::new();
                headers.insert("X-GitHub-Api-Version", HeaderValue::from_str("2022-11-28").unwrap());
                headers.insert(header::ACCEPT, HeaderValue::from_str("application/vnd.github+json").unwrap());
                headers.insert(header::USER_AGENT, HeaderValue::from_static("curl/7.54.1"));
                
                let client = reqwest::blocking::Client::new();
                let request = client
                    .get(format!("https://api.github.com/repos/{}/{}/labels", thread_owner, buckets.1.name))
                    .bearer_auth(thread_auth_token.clone())
                    .headers(headers)
                    .build().unwrap();

                let response_result = client.execute(request);

                match response_result {
                    Ok(response) => {
                        let parsed = response.json::<Vec<models::Label>>();
                        match parsed {
                            Ok(parsed_vec) => {
                                for label in parsed_vec {
                                    //println!("{}, {}", label.name, label.color);
                                    let next_tag = thread_tag_proto.clone()
                                        .with_name(label.name.clone())
                                        .with_hex_color(label.color.as_str());
                                    local_cached_tags.insert(label.name, next_tag);
                                }
                            },
                            Err(err) => println!("parse vec: {}", err),
                        }
                    },
                    Err(err) => println!("parse body: {}", err),
                }
                
            };

            match thread_tags.lock() {
                Ok(mut lock) => {
                    lock.clear();
                    lock.append(&mut local_cached_tags);
                    println!("appended {} tags", lock.len());
                },
                Err(_) => (),
            }
            


            if let Ok(mut lock) = update_trigger.lock() {
                *lock = true;
            };

        });

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