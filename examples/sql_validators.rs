use grammar_gen::utils::{
    SqlCaseFormat, SqlKeywordValidator, SqlNullValidator, ValidatorExt, ValidatorRegistry,
};
use grammar_gen::Grammar;
use std::error::Error;
use std::path::Path;

/// Example of using custom SQL validators with Grammar-Gen
fn main() -> Result<(), Box<dyn Error>> {
    // Use the pre-created SQL grammar file
    let grammar_path = "examples/sql_grammar.txt";
    if !Path::new(grammar_path).exists() {
        return Err(
            "SQL grammar file not found. Please ensure examples/sql_grammar.txt exists.".into(),
        );
    }

    // 1. Demonstrate SQL NULL value handling
    println!("=== Load the SQL grammar ===");
    let grammar = Grammar::from_file(grammar_path)?.with_validator(Box::new(SqlNullValidator));

    println!("=== SQL Create Handling ===");
    for i in 1..=3 {
        let query = grammar.generate("create_table");
        println!("{}. {}", i, query);
    }

    println!("=== SQL NULL Value Handling ===");
    for i in 1..=3 {
        let query = grammar.generate("query");
        println!("{}. {}", i, query);
    }

    // 2. Demonstrate different SQL case formatting options
    println!("\n=== SQL Keyword Case Formatting ===");
    test_case_formatting(grammar_path)?;

    // 3. Demonstrate chaining validators together
    println!("\n=== Chained Validators ===");
    let null_validator = SqlNullValidator;
    let keyword_validator = SqlKeywordValidator::new(SqlCaseFormat::Uppercase);
    let chained_validator = null_validator.chain(keyword_validator);

    let grammar = Grammar::from_file(grammar_path)?.with_validator(Box::new(chained_validator));

    for i in 1..=3 {
        let query = grammar.generate("query");
        println!("{}. {}", i, query);
    }

    // 4. Demonstrate using the validator registry
    println!("\n=== Validator Registry ===");
    let mut registry = ValidatorRegistry::new();
    registry.register_defaults();

    // Register a custom validator chain
    let custom_chain = SqlNullValidator.chain(SqlKeywordValidator::new(SqlCaseFormat::Capitalize));
    registry.register("custom_sql", custom_chain);

    println!("Available validators:");
    for name in registry.list_validators() {
        println!("- {}", name);
    }

    println!("\nUsing registry validators:");
    if let Some(validator) = registry.get("custom_sql") {
        let validator = (*validator).clone();
        let grammar = Grammar::from_file(grammar_path)?.with_validator(validator);

        for i in 1..=3 {
            let query = grammar.generate("query");
            println!("{}. {}", i, query);
        }
    }

    Ok(())
}

/// Test different SQL case formatting validators
fn test_case_formatting(grammar_path: &str) -> Result<(), Box<dyn Error>> {
    // Test uppercase formatting
    let uppercase_grammar = Grammar::from_file(grammar_path)?
        .with_validator(Box::new(SqlKeywordValidator::new(SqlCaseFormat::Uppercase)));
    println!("UPPERCASE keywords:");
    let query = uppercase_grammar.generate("query");
    println!("{}", query);

    // Test lowercase formatting
    let lowercase_grammar = Grammar::from_file(grammar_path)?
        .with_validator(Box::new(SqlKeywordValidator::new(SqlCaseFormat::Lowercase)));
    println!("\nlowercase keywords:");
    let query = lowercase_grammar.generate("query");
    println!("{}", query);

    // Test capitalized formatting
    let capitalize_grammar = Grammar::from_file(grammar_path)?.with_validator(Box::new(
        SqlKeywordValidator::new(SqlCaseFormat::Capitalize),
    ));

    println!("\nCapitalized keywords:");
    let query = capitalize_grammar.generate("query");
    println!("{}", query);

    Ok(())
}
