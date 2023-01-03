use std::fmt::{
    Display, 
    Debug, 
    Formatter
};

use std::fmt::Result as ErrorResult;

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub struct TokenizationError {
    error_string: String
}

impl TokenizationError {
    pub fn message(&self) -> String {
        self.error_string.clone()
    }
}

pub trait NewTokenizationError<T> {
    fn new(param: T) -> Self;
}

impl<T> NewTokenizationError<T> for TokenizationError where T: ToString {
    fn new(param: T) -> TokenizationError {
        TokenizationError { error_string: param.to_string() }
    }
}

impl Display for TokenizationError {
    fn fmt(&self, f: &mut Formatter) -> ErrorResult {
        write!(f, "An Error Occurred; {}.", self.error_string)
    }
}

impl Debug for TokenizationError {
    fn fmt(&self, f: &mut Formatter) -> ErrorResult {
        let (file, line) = (file!(), line!());
        write!(f, "{{ file: {file}, line: {line}, message: {} }}", self.error_string)
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Clone)]
pub struct SqlParseError {
    error_string: String
}

impl SqlParseError {
    pub fn _message(&self) -> String {
        self.error_string.clone()
    }
}

pub trait NewSqlParseError<T> {
    fn new(param: T) -> Self;
}

impl<T> NewSqlParseError<T> for SqlParseError where T: ToString {
    fn new(param: T) -> SqlParseError {
        SqlParseError { error_string: param.to_string() }
    }
}

impl Display for SqlParseError {
    fn fmt(&self, f: &mut Formatter) -> ErrorResult {
        write!(f, "An Error Occurred; {}.", self.error_string)
    }
}

impl Debug for SqlParseError {
    fn fmt(&self, f: &mut Formatter) -> ErrorResult {
        let (file, line) = (file!(), line!());
        write!(f, "{{ file: {file}, line: {line}, message: {} }}", self.error_string)
    }
}