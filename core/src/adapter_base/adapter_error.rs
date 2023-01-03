use std::fmt::{
    Display, 
    Formatter,
    Debug,
    Result
};

type Location = String;
type Message = String;
type ErrorsVec = Vec<(Location, Message)>;

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub enum AdapterErrorType {
    TicketWrite,
    TicketDelete,
    BucketWrite,
    BucketDelete,
    TagWrite,
    TagDelete,
    StateWrite,
    FilterWrite,
    FilterDelete,
    Access,
    Validate(ErrorsVec, String),
    Expression(String),
    Instantiation
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub struct AdapterError {
    pub error_type: AdapterErrorType
}

impl AdapterError {
    pub fn new(error_type: AdapterErrorType) -> Self {
        AdapterError { error_type }
    }

    pub fn get_text(&self) -> String {

        let mut message = String::default();

        match &self.error_type {
            AdapterErrorType::TicketDelete => message += "Failed to delete Ticket",
            AdapterErrorType::TicketWrite => message += "Failed to write Ticket",
            AdapterErrorType::Validate(_, name) => message += ("Failed to validate ".to_owned() + name).as_str(),
            AdapterErrorType::BucketWrite => message += "Failed to write Bucket",
            AdapterErrorType::BucketDelete => message += "Failed to delete Bucket",
            AdapterErrorType::FilterWrite => message += "Failed to write custom Filter",
            AdapterErrorType::FilterDelete => message += "Failed to delete custom Filter",
            AdapterErrorType::TagWrite => message += "Failed to write Tag",
            AdapterErrorType::TagDelete => message += "Failed to delete Tag",
            AdapterErrorType::StateWrite => message += "Failed to write State",
            AdapterErrorType::Access => message += "Failed access Adapter Data",
            AdapterErrorType::Expression(text) => message += ("Failed to execute Expression correctly. Reason: ".to_string() + text.as_str()).as_str(),
            AdapterErrorType::Instantiation => message += "Failed to instantiate Adapter"
        }

        message
    }
}

impl Display for AdapterError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "An Error Occurred; {}.", self.get_text())
    }
}

impl Debug for AdapterError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let (file, line) = (file!(), line!());
        write!(f, "{{ file: {file}, line: {line}, message: {} }}", self.get_text())
    }
}