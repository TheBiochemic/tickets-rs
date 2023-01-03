#[cfg(test)]
mod tests {
    use crate::local_ticket_adapter::{
        interpreter::{
            AdapterInterpreter, 
            TokenizationError, 
            NewTokenizationError
        }, 
        interpreter_instructions::{
            WithStateInstruction, 
            WithTagInstruction, 
            InBucketInstruction, 
            AssignedToInstruction, 
            DescriptionContainsInstruction,
            DueInDaysInstruction, 
            VerifiableInstruction
        },
        interpreter_parameters::{
            Parameter as Param, 
            Variable as Var, 
            Literal as Lit, 
            VerifiableData
        }};

    #[test]
    fn test_interpreter_to_sql() {
        let mut interpreter: AdapterInterpreter = AdapterInterpreter::default();
        interpreter.set_variable("me", "biochemist");
        let _ = interpreter.try_tokenize([
            "in_bucket(default.bucket)",
            "with_tag(documentation);;",
            "in_bucket(empty.bucket)",
            "with_tag(documentation)"
        ].join("\n"));

        assert_eq!(interpreter.get_last_error(), None);

        let result = interpreter.construct_sql();

        assert_eq!(result.unwrap(), [

            "SELECT tickets.* ",
                "FROM ((tickets) ",
                    "JOIN buckets ON tickets.bucket_id = buckets.id) ",
                    "JOIN ticket_tags ON tickets.id = ticket_tags.ticket_id ",
                "WHERE buckets.name = 'default.bucket' ",
                    "AND ticket_tags.tag_name = 'documentation' ",
            "UNION SELECT tickets.* ",
                "FROM ((tickets) ",
                    "JOIN buckets ON tickets.bucket_id = buckets.id) ",
                    "JOIN ticket_tags ON tickets.id = ticket_tags.ticket_id ",
                "WHERE buckets.name = 'empty.bucket' ",
                    "AND ticket_tags.tag_name = 'documentation';"

            ].join(""));
    }

    #[test]
    fn test_tokenizer_ingest() {
        let mut interpreter: AdapterInterpreter = AdapterInterpreter::default();
        interpreter.set_variable("me", "biochemist");
        let _ = interpreter.try_tokenize([
            "in_bucket(default.bucket)",
            "with_tag(documentation);;",
            "in_bucket(empty.bucket)",
            "with_tag(documentation)"
        ].join("\n"));

        assert_eq!(interpreter.get_last_error(), None);
        assert_eq!(interpreter.to_string(), [
            "in_bucket(default.bucket)",
            "with_tag(documentation)",
            ";;",
            "in_bucket(empty.bucket)",
            "with_tag(documentation)"
        ].join(" \n"));
    }

    #[test]
    fn test_single_tokens() {
        let mut interpreter: AdapterInterpreter = AdapterInterpreter::default();
        interpreter.set_variable("me", "biochemist");
        interpreter.set_variable("number", "42");

        // An Empty String on Token level should create an Error
        assert_eq!(
            WithStateInstruction::try_tokenize(&mut interpreter, "".to_string()).err(),
            Some(TokenizationError::new("Expected with_state for Token".to_string())));

        // Since the expression is incomplete, it will complain about a missing Variable/Literal
        assert_eq!(
            WithTagInstruction::try_tokenize(&mut interpreter, "with_tag".to_string()).err(),
            Some(TokenizationError::new("Expected (:: for Variable or\nExpected ( for Literal")));

        // Here it will complain about the variable not existing
        assert_eq!(
            InBucketInstruction::try_tokenize(&mut interpreter, "in_bucket(::nonexistent)".to_string()).err(),
            Some(TokenizationError::new("::nonexistent is not known. or\nNot allowed to interpret :: as Literal")));

        // This should work, since ::me exists and the syntax is correct
        assert_eq!(
            AssignedToInstruction::try_tokenize(&mut interpreter, "assigned_to(::me)".to_string()).ok(),
            Some((AssignedToInstruction{user: Param::Variable(Var::new("me".to_string()))}, "".to_string())));

        // This will work, because the literal is valid and the syntax is correct. Also there was no description_contains before
        assert_eq!(
            DescriptionContainsInstruction::try_tokenize(&mut interpreter, "description_contains(this is part of the text)".to_string()).ok(),
            Some((DescriptionContainsInstruction::new(
                Param::Literal(Lit::new("this is part of the text".to_string()))), "".to_string())));

        // This is supposed to fail, because it would be the second description_contains
        assert_eq!(
            DescriptionContainsInstruction::try_tokenize(&mut interpreter, "description_contains(this is another part of text)".to_string()).err(),
            Some(TokenizationError::new("Can't have more than one description_contains".to_string())));

        // This is supposed to fail because due_in_days requires a number
        assert_eq!(
            DueInDaysInstruction::try_tokenize(&mut interpreter, "due_in_days(not a number)".to_string()).err(),
            Some(TokenizationError::new("Type of Number required for due_in_days".to_string())));

        // This is supposed to work and return the variable
        assert_eq!(
            DueInDaysInstruction::try_tokenize(&mut interpreter, "due_in_days(::number)".to_string()).ok(),
            Some((DueInDaysInstruction{
                days: Param::Variable(Var::new("number".to_string()))}, "".to_string())));

        interpreter.can_have_due_in_days = true;

        // This is supposed to work and return the concrete value 42
        assert_eq!(
            DueInDaysInstruction::try_tokenize(
                &mut interpreter, 
                "due_in_days(::number)".to_string())
                    .unwrap().0.days.get_number(&interpreter).unwrap(),
            42);

        // This should return a String Vector
        assert_eq!(
            AssignedToInstruction::try_tokenize(
                &mut interpreter, 
                "assigned_to(biochemic, user1, user2)".to_string())
                    .unwrap().0.user.get_text_array(&interpreter).unwrap(),
            vec!["biochemic".to_string(), "user1".to_string(), "user2".to_string()]);
        
    }
}