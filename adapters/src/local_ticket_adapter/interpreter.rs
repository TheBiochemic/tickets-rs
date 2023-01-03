use std::collections::{HashMap, VecDeque};
use std::fmt::{
    Display, Result as FmtResult, Formatter
};
use std::sync::{Mutex, Arc};

use tickets_rs_core::AppConfig;

pub use super::interpreter_errors::{
    TokenizationError, 
    NewTokenizationError,
    SqlParseError,
    NewSqlParseError
};
use super::interpreter_instructions::{
    Instruction, 
    VerifiableInstruction
};

pub struct SqlExpression {
    finished_expression: String,
    from_expression: Vec<String>,
    where_expression: Vec<String>
}

impl Default for SqlExpression {
    fn default() -> Self {
        SqlExpression { 
            finished_expression: "".to_string(), 
            from_expression: vec![], 
            where_expression: vec![], 
        }
    }
}

impl SqlExpression {

    pub fn add_to_from(&mut self, expression: String) {

        if !self.from_expression.contains(&expression) {
            self.from_expression.push(expression);
        }
    }

    pub fn add_to_where(&mut self, expression: String) {
        self.where_expression.push(expression);
    }

    pub fn _add_directly(&mut self, expression: String) {
        self.finished_expression += expression.as_str();
    }

    pub fn flush(&mut self) {

        match self.finished_expression.is_empty() {
            true => self.finished_expression += "SELECT tickets.* FROM ",
            false => self.finished_expression += " UNION SELECT tickets.* FROM ",
        }

        let mut join_expression = "tickets".to_string();

        for from in self.from_expression.as_slice() {
            join_expression = [
                "(",
                join_expression.as_str(),
                ") JOIN ",
                from.as_str()
            ].join("")
        }

        self.finished_expression += join_expression.as_str();

        if !self.where_expression.is_empty() {
            self.finished_expression += " WHERE ";
            self.finished_expression += self.where_expression.join(" AND ").as_str();
        }
        
        self.where_expression.clear();
        self.from_expression.clear();

    }

    pub fn is_buffer_empty(&self) -> bool {
        self.finished_expression.is_empty() && self.from_expression.is_empty() && self.where_expression.is_empty()
    }

    pub fn is_accumulator_empty(&self) -> bool {
        self.from_expression.is_empty() && self.where_expression.is_empty()
    }

    pub fn get_final(&mut self) -> &String {
        self.finished_expression.push(';');
        &self.finished_expression
    }
}

pub trait SqlParsable: Sized {
    fn to_sql(&self, interpreter: &AdapterInterpreter, sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError>;
}

pub struct AdapterInterpreter {
    instructions: VecDeque<Instruction>,
    variables: HashMap<String, String>,
    last_error: Option<TokenizationError>,
    pub can_have_title_contains: bool,
    pub can_have_descr_contains: bool,
    pub can_have_due_in_days: bool
}

impl AdapterInterpreter {

    pub fn has_variable(&self, variable_name: &String) -> bool{
        self.variables.contains_key(variable_name)
    }

    pub fn get_variable(&self, variable_name: &String) -> Option<&String> {
        self.variables.get(variable_name)
    }

    pub fn set_variable(&mut self, variable_name: &str, value: &str) {
        self.variables.insert(String::from(variable_name), String::from(value));
    }

    #[cfg(test)]
    pub fn get_last_error(&self) -> Option<TokenizationError>{
        self.last_error.clone()
    }

    pub fn setup_environment(&mut self, config: Arc<Mutex<AppConfig>>) {
        match config.lock() {
            Ok(mut config) => {
                self.set_variable("me", config.get_or_default(
                    "username", "new User", ""
                ).raw().as_str());
            },
            Err(err) => println!("Wasn't able to lock Config. Reason: {}", err),
        }
    }

    pub fn try_tokenize(&mut self, code: String) -> Result<(), TokenizationError> {
        self.instructions.clear();
        let mut code_internal = code.trim_start().to_string();
        while !code_internal.is_empty() {

            if let Some(token_error) = match Instruction::try_tokenize(self, code_internal.clone()) {
                Ok(verified_instr) => {
                    code_internal = verified_instr.1;
                    self.instructions.push_back(verified_instr.0);
                    None
                },
                Err(token_error) => Some(token_error),
            } {
                self.last_error = Some(token_error.clone());
                self.instructions.clear();
                return Err(token_error);
            }

            code_internal = code_internal.trim_start().to_string();

        }

        self.last_error = None;
        Ok(())
    }

    pub fn construct_sql(&mut self) -> Result<String, SqlParseError> {
        if self.last_error.is_some() {
            return Err(SqlParseError::new("Cannot parse Instructions, because there was an Error when Tokenizing."));
        };

        if self.instructions.is_empty() {
            return Err(SqlParseError::new("Cannot parse Instruction, because nothing is in Buffer. Did you forget to tokenize first?"));
        };

        let mut expression: SqlExpression = SqlExpression::default();

        while let Some(instruction) = self.instructions.pop_front() {
            if let Some(parse_error) = match instruction.to_sql(self, expression) {
                Ok(new_expression) => {
                    expression = new_expression;
                    None
                },
                Err(parse_error) => {
                    expression = SqlExpression::default();
                    Some(parse_error)
                }
            } {
                return Err(parse_error);
            }
        };

        expression.flush();
        Ok(expression.get_final().clone())
    }
}

impl Default for AdapterInterpreter {
    fn default() -> Self {
        AdapterInterpreter { 
            instructions: VecDeque::new(), 
            variables: HashMap::new(), 
            can_have_title_contains: true, 
            can_have_descr_contains: true, 
            can_have_due_in_days: true,
            last_error: None
        }
    }
}

impl Display for AdapterInterpreter {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {

        let result = self.instructions
        .iter()
        .map(|instr| instr
            .to_string())
        .collect::<Vec<String>>()
        .join("\n")
        .trim()
        .to_string();

        write!(f, "{}", result)
    }
}