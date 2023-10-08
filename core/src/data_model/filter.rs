use crate::TicketAdapter;

/**
   The Filter Type describes, what type a Filter is, essentially which option it can have
   in the frontend for example
 */
#[derive(Default, PartialEq, Clone)]
pub enum FilterType {
    User,
    #[default] Builtin,
    Bucket(u64),
    Tag,
    Other
}

/**
   The Filter Identifier is a little Structure, that contains enough data to describe the
   Filter uniquely.
 */
#[derive(Default, PartialEq, Clone)]
pub struct FilterIdentifier {
    pub adapter: String,
    pub name: String
}

/**
   The filter has it's purpose on limiting the output of tickets.
   the adapter is the adapter the filter is from (not the adapter it is for).
   depending on it's filter_type it will be sorted differently in the user 
   interface. Usually a user generated filter can be created within the frontend
   and has the corresponding menu points in there. if the filter is builtin, it is
   a filter that is made, when the adapter is created, there are some other types,
   such as a bucket filter, and a tag based filter. the operation is an adapter 
   specific string, that determines the tickets that get listed.
 
   The operations structure usually is the following:
   ```[[adaptername: operation]]```
   
   You can chain multiple of these.
 */
#[derive(Default, PartialEq, Clone)]
pub struct Filter {
    pub identifier: FilterIdentifier,
    pub operation: String,
    pub filter_type: FilterType,
}

impl Filter {

    pub fn with_adapter(mut self, adapter: &dyn TicketAdapter) -> Self {
        self.identifier.adapter = adapter.get_name();
        self
    }

    pub fn with_type(mut self, filter_type: FilterType) -> Self {
        self.filter_type = filter_type;
        self
    }

    pub fn with_details(mut self, name: String, operation: String) -> Self {
        self.identifier.name = name;
        self.operation = operation;
        self
    }
}

impl Filter {

    pub fn filter_expression(adapter: String, inner_expression: &str) -> String {
        ["[[", adapter.as_str(), ": ", inner_expression, "]]"].join("")
    }

}