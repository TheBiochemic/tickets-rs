
use super::interpreter::{
    AdapterInterpreter, 
    SqlExpression, 
    SqlParsable
};

use super::interpreter_errors::{
    TokenizationError,
    SqlParseError, 
    NewTokenizationError, 
    NewSqlParseError
};

use super::interpreter_parameters::{
    Parameter, 
    VerifiableDataType, 
    VerifiableData
};

pub trait VerifiableInstruction: Sized {
    fn is_valid_after(&self, interpreter: &AdapterInterpreter, instruction: Instruction) -> bool;
    fn to_string(&self) -> String;
    fn try_tokenize(interpreter: &mut AdapterInterpreter, code: String) -> Result<(Self, String), TokenizationError>; 
}

pub enum Instruction {
    WithState(WithStateInstruction),
    WithTag(WithTagInstruction),
    InBucket(InBucketInstruction),
    TitleContains(TitleContainsInstruction),
    DescriptionContains(DescriptionContainsInstruction),
    AssignedTo(AssignedToInstruction),
    DueInDays(DueInDaysInstruction),
    Join(JoinInstruction)
}

impl VerifiableInstruction for Instruction {
    fn is_valid_after(&self, interpreter: &AdapterInterpreter, instruction: Instruction) -> bool {
        match self {
            Instruction::WithState(instr) => instr.is_valid_after(interpreter, instruction),
            Instruction::WithTag(instr) => instr.is_valid_after(interpreter, instruction),
            Instruction::InBucket(instr) => instr.is_valid_after(interpreter, instruction),
            Instruction::TitleContains(instr) => instr.is_valid_after(interpreter, instruction),
            Instruction::DescriptionContains(instr) => instr.is_valid_after(interpreter, instruction),
            Instruction::AssignedTo(instr) => instr.is_valid_after(interpreter, instruction),
            Instruction::DueInDays(instr) => instr.is_valid_after(interpreter, instruction),
            Instruction::Join(instr) => instr.is_valid_after(interpreter, instruction),
        }
    }

    fn to_string(&self) -> String {
        match self {
            Instruction::WithState(instr) => instr.to_string(),
            Instruction::WithTag(instr) => instr.to_string(),
            Instruction::InBucket(instr) => instr.to_string(),
            Instruction::TitleContains(instr) => instr.to_string(),
            Instruction::DescriptionContains(instr) => instr.to_string(),
            Instruction::AssignedTo(instr) => instr.to_string(),
            Instruction::DueInDays(instr) => instr.to_string(),
            Instruction::Join(instr) => instr.to_string(),
        }
    }

    fn try_tokenize(interpreter: &mut AdapterInterpreter, code: String) -> Result<(Self, String), TokenizationError> {
        let mut results: Vec<Result<(Instruction, String), TokenizationError>> = vec![];
        let mut error_messages: Vec<String> = vec![];

        
        match WithStateInstruction::try_tokenize(interpreter, code.clone()) {
            Ok(result) => results.push(Ok((Instruction::WithState(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        match WithTagInstruction::try_tokenize(interpreter, code.clone()) {
            Ok(result) => results.push(Ok((Instruction::WithTag(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        match InBucketInstruction::try_tokenize(interpreter, code.clone()) {
            Ok(result) => results.push(Ok((Instruction::InBucket(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        match TitleContainsInstruction::try_tokenize(interpreter, code.clone()) {
            Ok(result) => results.push(Ok((Instruction::TitleContains(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        match DescriptionContainsInstruction::try_tokenize(interpreter, code.clone()) {
            Ok(result) => results.push(Ok((Instruction::DescriptionContains(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        match AssignedToInstruction::try_tokenize(interpreter, code.clone()) {
            Ok(result) => results.push(Ok((Instruction::AssignedTo(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        match DueInDaysInstruction::try_tokenize(interpreter, code.clone()) {
            Ok(result) => results.push(Ok((Instruction::DueInDays(result.0), result.1))),
            Err(err) => results.push(Err(err)),
        };

        match JoinInstruction::try_tokenize(interpreter, code) {
            Ok(result) => results.push(Ok((Instruction::Join(result.0), result.1))),
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
}

impl SqlParsable for Instruction {
    fn to_sql(&self, interpreter: &AdapterInterpreter, sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        match self {
            Instruction::WithState(instr) => instr.to_sql(interpreter, sql_expression),
            Instruction::WithTag(instr) => instr.to_sql(interpreter, sql_expression),
            Instruction::InBucket(instr) => instr.to_sql(interpreter, sql_expression),
            Instruction::TitleContains(instr) => instr.to_sql(interpreter, sql_expression),
            Instruction::DescriptionContains(instr) => instr.to_sql(interpreter, sql_expression),
            Instruction::AssignedTo(instr) => instr.to_sql(interpreter, sql_expression),
            Instruction::DueInDays(instr) => instr.to_sql(interpreter, sql_expression),
            Instruction::Join(instr) => instr.to_sql(interpreter, sql_expression),
        }
    }
}

trait FunctionTypeInstruction {

    /**
       Returns it's single parameter
     */
    fn get_content(&self) -> &Parameter;

    /**
       Returns the type of the parameter.
       Text is the broadest type and usually works
     */
    fn required_parameter_type() -> VerifiableDataType;

    /**
       Returns the function's name
     */
    fn get_function_name() -> String;

    /**
       returns an instance with the parameter filled
     */
    fn get_instance(param: Parameter) -> Self;

    /**
       returns wether the Instruction is valid to use (based on it's uniqueness)
       also updates whatever value is determining it's uniqueness
     */
    fn initiate_unique(interpreter: &mut AdapterInterpreter) -> bool;
}

impl<T> VerifiableInstruction for T where T: FunctionTypeInstruction {
    fn is_valid_after(&self, _interpreter: &AdapterInterpreter, _instruction: Instruction) -> bool {
        true
    }

    fn to_string(&self) -> String {
        let mut expression: String = Self::get_function_name();

        expression.push_str(self.get_content().to_string().as_str());

        expression
    }

    fn try_tokenize(interpreter: &mut AdapterInterpreter, code: String) -> Result<(Self, String), TokenizationError> {
        let mut code_internal = code.trim_start();
        if code_internal.starts_with(Self::get_function_name().as_str()) {
            code_internal = code_internal.split_at(Self::get_function_name().len()).1;
            match Parameter::try_tokenize(interpreter, code_internal.to_string()) {
                Ok(result) => {
                    if result.0.is_valid_type(interpreter, Self::required_parameter_type()) {
                        if Self::initiate_unique(interpreter) {
                            let instruction = Self::get_instance(result.0);
                            Ok((instruction, result.1))
                        } else {
                            Err(TokenizationError::new(["Can't have more than one ", Self::get_function_name().as_str()].join("")))
                        }
                    } else {
                        Err(TokenizationError::new([
                            "Type of ", 
                            Self::required_parameter_type().get_type_name().as_str(), 
                            " required for ", 
                            Self::get_function_name().as_str()].join("")))
                    }
                    
                },
                Err(err) => Err(err)
            }
        } else {
            Err(TokenizationError::new(["Expected ", Self::get_function_name().as_str(), " for Token"].join("")))
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct WithStateInstruction {
    state: Parameter
}

impl FunctionTypeInstruction for WithStateInstruction {
    fn get_content(&self) -> &Parameter { &self.state }
    fn get_function_name() -> String { "with_state".to_string() }
    fn get_instance(param: Parameter) -> Self {WithStateInstruction { state: param }}
    fn initiate_unique(_interpreter: &mut AdapterInterpreter) -> bool { true }
    fn required_parameter_type() -> VerifiableDataType {VerifiableDataType::Text}
}

impl SqlParsable for WithStateInstruction {
    fn to_sql(&self, interpreter: &AdapterInterpreter, mut sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        
        let state_option = self.state.get_text(interpreter);

        if let Some(state) = state_option {
            sql_expression.add_to_where(
                ["tickets.state_name = '", state.as_str(), "'"]
                .join("")
            );
            Ok(sql_expression)
        } else {
            Err(SqlParseError::new("Wasn't able to parse with_state because of wrong Parameter Type"))
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct WithTagInstruction {
    tag: Parameter
}

impl FunctionTypeInstruction for WithTagInstruction {
    fn get_content(&self) -> &Parameter {&self.tag}
    fn get_function_name() -> String {"with_tag".to_string()}
    fn get_instance(param: Parameter) -> Self {WithTagInstruction { tag: param }}
    fn initiate_unique(_interpreter: &mut AdapterInterpreter) -> bool { true }
    fn required_parameter_type() -> VerifiableDataType {VerifiableDataType::Text}
}

impl SqlParsable for WithTagInstruction {
    fn to_sql(&self, interpreter: &AdapterInterpreter, mut sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        let tag_option = self.tag.get_text(interpreter);

        if let Some(tag) = tag_option {

            sql_expression.add_to_where(
                ["ticket_tags.tag_name = '", tag.as_str(), "'"]
                .join("")
            );

            sql_expression.add_to_from("ticket_tags ON tickets.id = ticket_tags.ticket_id".to_string());

            Ok(sql_expression)
        } else {
            Err(SqlParseError::new("Wasn't able to parse with_tag because of wrong Parameter Type"))
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct InBucketInstruction {
    bucket: Parameter
}

impl FunctionTypeInstruction for InBucketInstruction {
    fn get_content(&self) -> &Parameter {&self.bucket}
    fn get_function_name() -> String {"in_bucket".to_string()}
    fn get_instance(param: Parameter) -> Self {InBucketInstruction { bucket: param }}
    fn initiate_unique(_interpreter: &mut AdapterInterpreter) -> bool { true }
    fn required_parameter_type() -> VerifiableDataType {VerifiableDataType::Text}
}

impl SqlParsable for InBucketInstruction {
    fn to_sql(&self, interpreter: &AdapterInterpreter, mut sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        let bucket_option = self.bucket.get_text(interpreter);

        if let Some(bucket) = bucket_option {

            sql_expression.add_to_where(
                ["buckets.name = '", bucket.as_str(), "'"]
                .join("")
            );

            sql_expression.add_to_from("buckets ON tickets.bucket_id = buckets.id".to_string());

            Ok(sql_expression)
        } else {
            Err(SqlParseError::new("Wasn't able to parse in_bucket because of wrong Parameter Type"))
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct TitleContainsInstruction {
    title: Parameter
}

impl FunctionTypeInstruction for TitleContainsInstruction {
    fn get_content(&self) -> &Parameter {&self.title}
    fn get_function_name() -> String {"title_contains".to_string()}
    fn get_instance(param: Parameter) -> Self {TitleContainsInstruction { title: param }}
    fn initiate_unique(interpreter: &mut AdapterInterpreter) -> bool {
        let unique = interpreter.can_have_title_contains;
        interpreter.can_have_title_contains = false;
        unique
    }
    fn required_parameter_type() -> VerifiableDataType {VerifiableDataType::Text}
}

impl SqlParsable for TitleContainsInstruction {
    fn to_sql(&self, interpreter: &AdapterInterpreter, mut sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        let title_option = self.title.get_text(interpreter);

        if let Some(title) = title_option {

            sql_expression.add_to_where(
                ["tickets.title LIKE '%", title.as_str(), "%'"]
                .join("")
            );

            Ok(sql_expression)
        } else {
            Err(SqlParseError::new("Wasn't able to parse title_contains because of wrong Parameter Type"))
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct DescriptionContainsInstruction {
    description: Parameter
}

impl DescriptionContainsInstruction {
    #[cfg(test)] //It's so far only used in tests
    pub fn new(description: Parameter) -> DescriptionContainsInstruction {
        DescriptionContainsInstruction { description }
    }
}

impl FunctionTypeInstruction for DescriptionContainsInstruction {
    fn get_content(&self) -> &Parameter {&self.description}
    fn get_function_name() -> String {"description_contains".to_string()}
    fn get_instance(param: Parameter) -> Self {DescriptionContainsInstruction { description: param }}
    fn initiate_unique(interpreter: &mut AdapterInterpreter) -> bool {
        let unique = interpreter.can_have_descr_contains;
        interpreter.can_have_descr_contains = false;
        unique
    }
    fn required_parameter_type() -> VerifiableDataType {VerifiableDataType::Text}
}

impl SqlParsable for DescriptionContainsInstruction {
    fn to_sql(&self, interpreter: &AdapterInterpreter, mut sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        let desc_option = self.description.get_text(interpreter);

        if let Some(desc) = desc_option {

            sql_expression.add_to_where(
                ["tickets.description LIKE '%", desc.as_str(), "%'"]
                .join("")
            );

            Ok(sql_expression)
        } else {
            Err(SqlParseError::new("Wasn't able to parse description_contains because of wrong Parameter Type"))
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct AssignedToInstruction {
    pub user: Parameter
}

impl FunctionTypeInstruction for AssignedToInstruction {
    fn get_content(&self) -> &Parameter {&self.user}
    fn get_function_name() -> String {"assigned_to".to_string()}
    fn get_instance(param: Parameter) -> Self {AssignedToInstruction { user: param }}
    fn initiate_unique(_interpreter: &mut AdapterInterpreter) -> bool { true }
    fn required_parameter_type() -> VerifiableDataType {VerifiableDataType::TextArray}
}

impl SqlParsable for AssignedToInstruction {
    fn to_sql(&self, interpreter: &AdapterInterpreter, mut sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        let users_option = self.user.get_text_array(interpreter);

        if let Some(users) = users_option {

            let user_comps: Vec<String> = users.iter()
                 .map(|username| ["tickets.assigned_to = '", username.as_str(), "'"].join(""))
                 .collect();
            
            sql_expression.add_to_where(
                ["(", user_comps.join(" OR ").as_str(), ")"]
                .join("")
            );

            Ok(sql_expression)
        } else {
            Err(SqlParseError::new("Wasn't able to parse assigned_to because of wrong Parameter Type"))
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct DueInDaysInstruction {
    pub days: Parameter
}

impl FunctionTypeInstruction for DueInDaysInstruction {
    fn get_content(&self) -> &Parameter {&self.days}
    fn get_function_name() -> String {"due_in_days".to_string()}
    fn get_instance(param: Parameter) -> Self {DueInDaysInstruction { days: param }}
    fn initiate_unique(interpreter: &mut AdapterInterpreter) -> bool {
        let unique = interpreter.can_have_due_in_days;
        interpreter.can_have_due_in_days = false;
        unique
    }
    fn required_parameter_type() -> VerifiableDataType {VerifiableDataType::Number}
}

impl SqlParsable for DueInDaysInstruction {
    fn to_sql(&self, interpreter: &AdapterInterpreter, mut sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        let days_options = self.days.get_number(interpreter);

        if let Some(days) = days_options {
            sql_expression.add_to_where(
                ["tickets.due_at < (SELECT unixepoch('now','start of day','+", days.to_string().as_str() , " day'))"]
                .join("")
            );
            Ok(sql_expression)
        } else {
            Err(SqlParseError::new("Wasn't able to parse due_in_days because of wrong Parameter Type"))
        }
    }
}

#[derive(Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct JoinInstruction {}

impl VerifiableInstruction for JoinInstruction {
    fn is_valid_after(&self, _interpreter: &AdapterInterpreter, instruction: Instruction) -> bool {
        !matches!(instruction, Instruction::Join(_))
    }

    fn to_string(&self) -> String {
        ";; ".to_string()
    }

    fn try_tokenize(_interpreter: &mut AdapterInterpreter, code: String) -> Result<(Self, String), TokenizationError> {
        let code_internal = code.trim_start();

        if code_internal.starts_with(";;") {
            let code_final = code_internal.split_at(2).1;
            Ok((JoinInstruction{}, code_final.to_string()))
        } else {
            Err(TokenizationError::new("Expected ;; for Token"))
        }
    }
}

impl SqlParsable for JoinInstruction {
    fn to_sql(&self, _interpreter: &AdapterInterpreter, mut sql_expression: SqlExpression) -> Result<SqlExpression, SqlParseError> {
        if sql_expression.is_buffer_empty() {
            return Err(SqlParseError::new("Join instruction in the beginning is not allowed."));
        }

        if sql_expression.is_accumulator_empty() {
            return Err(SqlParseError::new("Can't have more than one ;; after eachother."));
        }

        sql_expression.flush();
        Ok(sql_expression)
    }
}