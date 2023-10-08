use std::time::{
    SystemTime, 
    UNIX_EPOCH
};

use crate::TicketAdapter;

use super::{
    Bucket, 
    State,
    Tag
};

#[derive(Eq, PartialOrd, Ord, Debug, PartialEq, Clone)]
pub struct Ticket {
    pub adapter: String,
    pub id: i64,
    pub bucket_id: u64,
    pub title: String,
    pub assigned_to: String,
    pub state_name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub due_at: i64
}

impl Default for Ticket {
    fn default() -> Self {
        Ticket { 
            adapter: String::default(),
            id: 0, 
            bucket_id: 0, 
            title: String::default(), 
            assigned_to: String::default(),
            state_name: String::default(), 
            description: String::default(), 
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64, 
            due_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64, 
            tags: vec![]
        }
    }
}

impl Ticket {

    pub fn with_adapter(mut self, adapter: &dyn TicketAdapter) -> Self {
        self.adapter = adapter.get_name();
        self
    }

    pub fn with_details(mut self, id: i64, title: String, description: String) -> Self {
        self.id = id;
        self.title = title;
        self.description = description;
        self
    }

    pub fn with_bucket(mut self, bucket: &Bucket) -> Self {
        self.bucket_id = bucket.identifier.id;
        self
    }

    pub fn with_state(mut self, state: &State) -> Self {
        self.state_name = state.identifier.name.clone();
        self
    }

    pub fn with_tags(mut self, tags: Vec<&Tag>) -> Self {
        for tag in tags {
            self.tags.push(tag.name.clone());
        }

        self
    }

    pub fn with_assignee(mut self, assignee: String) -> Self {
        self.assign_to(assignee);
        self
    }

    pub fn add_tag(&mut self, tag: &Tag) {
        if !self.tags.contains(&tag.name) {
            self.tags.push(tag.name.clone())
        }
    }

    pub fn assign_to(&mut self, assignee: String) {
        self.assigned_to = assignee;
    }
}