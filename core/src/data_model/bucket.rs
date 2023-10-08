use std::time::{
    SystemTime, 
    UNIX_EPOCH
};

use crate::TicketAdapter;

#[derive(Default, PartialEq, Clone, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct BucketIdentifier {
    pub adapter: String,
    pub id: u64,
}

impl BucketIdentifier {
    pub fn new(adapter: &String, id: u64) -> BucketIdentifier {
        BucketIdentifier {
            adapter: adapter.clone(),
            id
        }
    }
}

#[derive(Eq, PartialOrd, Ord, Debug, PartialEq, Clone)]
pub struct Bucket {
    pub identifier: BucketIdentifier,
    pub name: String,
    pub last_change: i64
}

impl Default for Bucket {

    fn default() -> Self {
        Bucket{
            identifier: BucketIdentifier::default(),
            name: String::default(),
            last_change: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
        }
    }
}

impl Bucket {

    pub fn with_adapter(mut self, adapter: &dyn TicketAdapter) -> Self {
        self.identifier.adapter = adapter.get_name();
        self
    }

    pub fn with_details(mut self, id: u64, name: String) -> Self {
        self.identifier.id = id;
        self.name = name;
        self
    }
}

