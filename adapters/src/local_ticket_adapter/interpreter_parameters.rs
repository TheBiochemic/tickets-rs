use super::{
    interpreter::AdapterInterpreter, 
    interpreter_errors::{TokenizationError, NewTokenizationError}
};



pub enum VerifiableDataType {
    Text,
    TextArray,
    Number,
    //Boolean
}

impl VerifiableDataType {
    pub fn get_type_name(&self) -> String {
        match self {
            VerifiableDataType::Text => "Text",
            VerifiableDataType::TextArray => "TextArray",
            VerifiableDataType::Number => "Number",
            //VerifiableDataType::Boolean => "Boolean",
        }.to_string()
    }
}

pub trait VerifiableData: Sized {
    fn to_string(&self) -> String;
    fn try_tokenize(interpreter: &mut AdapterInterpreter, code: String) -> Result<(Self, String), TokenizationError>; 

    fn is_valid_type(&self, interpreter: &AdapterInterpreter, v_type: VerifiableDataType) -> bool {
        match v_type {
            VerifiableDataType::Text => self.get_text(interpreter).is_some(),
            VerifiableDataType::TextArray => self.get_text_array(interpreter).is_some(),
            VerifiableDataType::Number => self.get_number(interpreter).is_some(),
            //VerifiableDataType::Boolean => self.get_boolean(interpreter).is_some(),
        }
    }

    fn get_text(&self, interpreter: &AdapterInterpreter) -> Option<String>;
    fn get_text_array(&self, interpreter: &AdapterInterpreter) -> Option<Vec<String>>;
    fn get_number(&self, interpreter: &AdapterInterpreter) -> Option<i32>;
    fn get_boolean(&self, interpreter: &AdapterInterpreter) -> Option<bool>;
}


#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct Variable {
    variable_name: String
}

impl Variable {
    #[cfg(test)] // This is only used in tests so far
    pub fn new (variable_name: String) -> Variable {
        Variable { variable_name }
    }
}

impl VerifiableData for Variable {

    fn to_string(&self) -> String {
        ["(::", self.variable_name.as_str(), ") "].join("")
    }

    fn try_tokenize(interpreter: &mut AdapterInterpreter, code: String) -> Result<(Self, String), TokenizationError> {

        let mut code_internal = code.trim_start();
        if code_internal.starts_with("(::") {

            code_internal = code_internal.split_at(3).1;

            if let Some(pos) = code_internal.find(')') {
                let (variable, split_off_code) = code_internal.split_at(pos);
                code_internal = split_off_code.split_at(1).1;

                let final_variable = variable.trim().to_string();

                if interpreter.has_variable(&final_variable) {
                    Ok((Variable{variable_name: final_variable}, code_internal.to_string()))
                } else {
                    Err(TokenizationError::new([
                        "::", 
                        final_variable.as_str(),
                        " is not known."
                    ].join("") ))
                }

            } else {
                Err(TokenizationError::new("Expected ) for Variable"))
            }

        } else {
            Err(TokenizationError::new("Expected (:: for Variable"))
        }
    }

    fn get_text(&self, interpreter: &AdapterInterpreter) -> Option<String> {
        interpreter.get_variable(&self.variable_name).map(|found_var| found_var.to_owned())
    }

    fn get_text_array(&self, interpreter: &AdapterInterpreter) -> Option<Vec<String>> {
        interpreter.get_variable(&self.variable_name)
            .map(|found_var| found_var
                .split(',')
                .map(|l| l.trim().to_string())
                .collect())
    }

    fn get_number(&self, interpreter: &AdapterInterpreter) -> Option<i32> {
        match interpreter.get_variable(&self.variable_name) {
            Some(found_var) => {
                match found_var.as_str().parse::<i32>() {
                    Ok(num) => Some(num),
                    Err(_) => None
                }
            },
            None => None
        }
        
    }

    fn get_boolean(&self, interpreter: &AdapterInterpreter) -> Option<bool> {
        match interpreter.get_variable(&self.variable_name) {
            Some(found_var) => {
                match found_var.trim().to_lowercase().as_str() {
                    "yes" => Some(true),
                    "no" => Some(false),
                    "true" => Some(true),
                    "false" => Some(false),
                    "1" => Some(true),
                    "0" => Some(false),
                    _ => None
                }
            },
            None => None
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct Literal {
    literal: String
}

impl Literal {
    #[cfg(test)]
    pub fn new(literal: String) -> Literal {
        Literal { literal }
    }
}

impl VerifiableData for Literal {

    fn to_string(&self) -> String {
        ["(", self.literal.as_str(), ") "].join("")
    }

    fn try_tokenize(_interpreter: &mut AdapterInterpreter, code: String) -> Result<(Self, String), TokenizationError> {

        let mut code_internal = code.trim_start();
        if code_internal.starts_with('(') {

            if code_internal.starts_with("(::") {
                return Err(TokenizationError::new("Not allowed to interpret :: as Literal"));
            };

            code_internal = code_internal.split_at(1).1;

            if let Some(pos) = code_internal.find(')') {

                let (literal, split_off_code) = code_internal.split_at(pos);
                code_internal = split_off_code.split_at(1).1;
                let final_literal = literal.trim().to_string();

                if !final_literal.is_empty() {
                    Ok((Literal{literal: final_literal}, code_internal.to_string()))
                } else {
                    Err(TokenizationError::new("(...) cannot be empty!"))
                }

            } else {
                Err(TokenizationError::new("Expected ) for Literal"))
            }

        } else {
            Err(TokenizationError::new("Expected ( for Literal"))
        }
    }

    fn get_text(&self, _interpreter: &AdapterInterpreter) -> Option<String> {
        Some(self.literal.to_owned())
    }

    fn get_text_array(&self, _interpreter: &AdapterInterpreter) -> Option<Vec<String>> {
        Some(self.literal
            .split(',')
            .map(|l| l.trim().to_string())
            .collect())
    }

    fn get_number(&self, _interpreter: &AdapterInterpreter) -> Option<i32> {
        match self.literal.as_str().parse::<i32>() {
            Ok(num) => Some(num),
            Err(_) => None
        }
    }

    fn get_boolean(&self, _interpreter: &AdapterInterpreter) -> Option<bool> {
        match self.literal.trim().to_lowercase().as_str() {
            "yes" => Some(true),
            "no" => Some(false),
            "true" => Some(true),
            "false" => Some(false),
            "1" => Some(true),
            "0" => Some(false),
            _ => None
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub enum Parameter {
    Variable(Variable),
    Literal(Literal)
}

impl VerifiableData for Parameter {

    fn to_string(&self) -> String {
        match self {
            Parameter::Variable(variable) => variable.to_string(),
            Parameter::Literal(literal) => literal.to_string(),
        }
    }

    fn try_tokenize(interpreter: &mut AdapterInterpreter, code: String) -> Result<(Self, String), TokenizationError> {
        let mut results: Vec<Result<(Parameter, String), TokenizationError>> = vec![];
        let mut error_messages: Vec<String> = vec![];

        match Variable::try_tokenize(interpreter, code.clone()) {
            Ok(result) => results.push(Ok((Parameter::Variable(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        match Literal::try_tokenize(interpreter, code) {
            Ok(result) => results.push(Ok((Parameter::Literal(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        for result in results {
            if let Some(result) = match result {
                Ok(result) => Some(result),
                Err(error) => {
                    error_messages.push(error.message());
                    None
                },
            } {
                return Ok(result);
            }
        };
        let con_error_messages: Vec<&str> = error_messages.iter().map(|msg| msg.as_str()).collect();
        Err(TokenizationError::new(con_error_messages.join(" or\n")))
    }

    fn get_text(&self, interpreter: &AdapterInterpreter) -> Option<String> {
        match self {
            Parameter::Variable(variable) => variable.get_text(interpreter),
            Parameter::Literal(literal) => literal.get_text(interpreter)
        }
    }

    fn get_text_array(&self, interpreter: &AdapterInterpreter) -> Option<Vec<String>> {
        match self {
            Parameter::Variable(variable) => variable.get_text_array(interpreter),
            Parameter::Literal(literal) => literal.get_text_array(interpreter)
        }
    }

    fn get_number(&self, interpreter: &AdapterInterpreter) -> Option<i32> {
        match self {
            Parameter::Variable(variable) => variable.get_number(interpreter),
            Parameter::Literal(literal) => literal.get_number(interpreter)
        }
    }

    fn get_boolean(&self, interpreter: &AdapterInterpreter) -> Option<bool> {
        match self {
            Parameter::Variable(variable) => variable.get_boolean(interpreter),
            Parameter::Literal(literal) => literal.get_boolean(interpreter)
        }
    }
}