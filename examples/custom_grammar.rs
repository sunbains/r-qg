use grammar_gen::{Grammar, GrammarBuilder, GrammarConfig};
use std::error::Error;

/// Example of creating a custom grammar programmatically
fn main() -> Result<(), Box<dyn Error>> {
    // Example 1: Create a grammar programmatically using the builder
    let grammar = GrammarBuilder::new("sentence")
        .add_rule("sentence", &["<subject>", "<verb>", "<object>"])
        .add_rule("subject", &["The", "<adjective>", "<noun>"])
        .add_rule("subject", &["A", "<adjective>", "<noun>"])
        .add_rule("adjective", &["quick"])
        .add_rule("adjective", &["lazy"])
        .add_rule("adjective", &["clever"])
        .add_rule("noun", &["fox"])
        .add_rule("noun", &["dog"])
        .add_rule("noun", &["programmer"])
        .add_rule("verb", &["jumps over"])
        .add_rule("verb", &["runs around"])
        .add_rule("verb", &["observes"])
        .add_rule("object", &["the", "<adjective>", "<noun>"])
        .build();
    
    println!("Generated sentences using builder approach:");
    for i in 1..=5 {
        println!("{}. {}", i, grammar.generate());
    }
    
    // Example 2: Create a grammar manually
    let mut custom_config = GrammarConfig::default();
    custom_config.auto_spacing = true;
    custom_config.max_recursion_depth = 30;
    
    let mut grammar = Grammar::with_config("greeting", custom_config);
    
    // Add rules to generate greetings in different languages
    grammar.add_rule("greeting", vec!["<english_greeting>"])?;
    grammar.add_rule("greeting", vec!["<spanish_greeting>"])?;
    grammar.add_rule("greeting", vec!["<french_greeting>"])?;
    grammar.add_rule("greeting", vec!["<german_greeting>"])?;
    
    grammar.add_rule("english_greeting", vec!["Hello", "<person>", "!"])?;
    grammar.add_rule("spanish_greeting", vec!["¡Hola", "<person>", "!"])?;
    grammar.add_rule("french_greeting", vec!["Bonjour", "<person>", "!"])?;
    grammar.add_rule("german_greeting", vec!["Guten Tag", "<person>", "!"])?;
    
    grammar.add_rule("person", vec!["world"])?;
    grammar.add_rule("person", vec!["friend"])?;
    grammar.add_rule("person", vec!["programmer"])?;
    grammar.add_rule("person", vec!["everyone"])?;
    
    println!("\nGenerated greetings in different languages:");
    for i in 1..=5 {
        println!("{}. {}", i, grammar.generate());
    }
    
    // Example 3: Create a more complex custom grammar
    // Programming language syntax example
    let mut code_grammar = Grammar::new("program");
    
    // Set a reasonable recursion limit
    let mut config = GrammarConfig::default();
    config.max_recursion_depth = 5;
    code_grammar.set_config(config);
    
    // Base case for program
    code_grammar.add_rule("program", vec!["<statement>"])?;
    code_grammar.add_rule("program", vec!["<statement>", "<program>"])?;
    
    // Base case for statement_list
    code_grammar.add_rule("statement_list", vec!["<statement>"])?;
    code_grammar.add_rule("statement_list", vec!["<statement>", "<statement>"])?;
    
    code_grammar.add_rule("statement", vec!["<variable>", "=", "<expression>", ";"])?;
    code_grammar.add_rule("statement", vec!["if", "(", "<condition>", ")", "{", "<statement>", "}"])?;
    code_grammar.add_rule("statement", vec!["print", "(", "<expression>", ")", ";"])?;
    
    // Base case for expression
    code_grammar.add_rule("expression", vec!["<term>"])?;
    code_grammar.add_rule("expression", vec!["<term>", "+", "<term>"])?;
    code_grammar.add_rule("expression", vec!["<term>", "-", "<term>"])?;
    
    // Base case for term
    code_grammar.add_rule("term", vec!["<factor>"])?;
    code_grammar.add_rule("term", vec!["<factor>", "*", "<factor>"])?;
    code_grammar.add_rule("term", vec!["<factor>", "/", "<factor>"])?;
    
    code_grammar.add_rule("factor", vec!["<number>"])?;
    code_grammar.add_rule("factor", vec!["<variable>"])?;
    code_grammar.add_rule("factor", vec!["(", "<expression>", ")"])?;
    
    code_grammar.add_rule("condition", vec!["<expression>", "<comparator>", "<expression>"])?;
    
    code_grammar.add_rule("comparator", vec!["=="])?;
    code_grammar.add_rule("comparator", vec![">"])?;
    code_grammar.add_rule("comparator", vec!["<"])?;
    code_grammar.add_rule("comparator", vec![">="])?;
    code_grammar.add_rule("comparator", vec!["<="])?;
    
    code_grammar.add_rule("variable", vec!["x"])?;
    code_grammar.add_rule("variable", vec!["y"])?;
    code_grammar.add_rule("variable", vec!["z"])?;
    code_grammar.add_rule("variable", vec!["count"])?;
    code_grammar.add_rule("variable", vec!["total"])?;
    
    code_grammar.add_rule("number", vec!["0"])?;
    code_grammar.add_rule("number", vec!["1"])?;
    code_grammar.add_rule("number", vec!["42"])?;
    code_grammar.add_rule("number", vec!["100"])?;
    
    println!("\nGenerated code snippets:");
    for i in 1..=3 {
        println!("Code Example {}:\n{}\n", i, code_grammar.generate());
    }
    
    Ok(())
}

