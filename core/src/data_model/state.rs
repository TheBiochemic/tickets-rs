use crate::TicketAdapter;

#[derive(Default, PartialEq, Clone, Eq, Hash)]
pub struct StateIdentifier {
    pub adapter: String,
    pub name: String,
}

impl StateIdentifier {
    pub fn new(adapter: &String, name: &String) -> Self {
        StateIdentifier { adapter: adapter.clone(), name: name.clone() }
    }
}

#[derive(Default, PartialEq, Clone)]
pub struct State {
    pub identifier: StateIdentifier,
    pub description: String,
    pub sorting_order: i64
}

impl State {

    pub fn with_name(mut self, name: String) -> Self {
        self.identifier.name = name;
        self
    }

    pub fn with_order(mut self, sorting_order: i64) -> Self {
        self.sorting_order = sorting_order;
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_adapter(mut self, adapter: &dyn TicketAdapter ) -> Self {
        self.identifier.adapter = adapter.get_name();
        self
    }
}