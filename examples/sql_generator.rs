use std::error::Error;

use grammar_gen::{utils::sql_validator, utils::SqlCaseFormat, Grammar};

fn main() -> Result<(), Box<dyn Error>> {
    // Load the grammar with SQL NULL validation
    let grammar = Grammar::from_file("examples/sql_grammar.txt")?
        .with_validator(sql_validator(SqlCaseFormat::Uppercase));

    println!("Generated SQL Queries:");

    for i in 1..=10 {
        let query = grammar.generate("query");
        println!("{}. {}", i, query);
    }

    Ok(())
}
