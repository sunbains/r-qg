use grammar_gen::{Grammar, GrammarConfig};
use std::error::Error;

/// Example of creating a custom grammar programmatically
fn main() -> Result<(), Box<dyn Error>> {
    // Example 1: Create a grammar programmatically
    let mut grammar = Grammar::new();
    grammar.add_rule("sentence", vec!["<subject>", "<verb>", "<object>"])?;
    grammar.add_rule("subject", vec!["The", "<adjective>", "<noun>"])?;
    grammar.add_rule("subject", vec!["A", "<adjective>", "<noun>"])?;
    grammar.add_rule("adjective", vec!["quick"])?;
    grammar.add_rule("adjective", vec!["lazy"])?;
    grammar.add_rule("adjective", vec!["clever"])?;
    grammar.add_rule("noun", vec!["fox"])?;
    grammar.add_rule("noun", vec!["dog"])?;
    grammar.add_rule("noun", vec!["programmer"])?;
    grammar.add_rule("verb", vec!["jumps over"])?;
    grammar.add_rule("verb", vec!["runs around"])?;
    grammar.add_rule("verb", vec!["observes"])?;
    grammar.add_rule("object", vec!["the", "<adjective>", "<noun>"])?;

    println!("Generated sentences using builder approach:");
    for i in 1..=5 {
        let result = grammar.generate("sentence");
        println!("{}. {}", i, result.text);
    }

    // Example 2: Create a grammar manually
    let mut custom_config = GrammarConfig::default();
    custom_config.auto_spacing = true;
    custom_config.max_recursion_depth = 30;

    let mut grammar = Grammar::with_config(custom_config);

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
        let result = grammar.generate("greeting");
        println!("{}. {}", i, result.text);
    }

    // Example 3: Create a more complex custom grammar
    // Programming language syntax example
    let mut code_grammar = Grammar::new();

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
    code_grammar.add_rule(
        "statement",
        vec!["if", "(", "<condition>", ")", "{", "<statement>", "}"],
    )?;
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

    code_grammar.add_rule(
        "condition",
        vec!["<expression>", "<comparator>", "<expression>"],
    )?;

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
        let result = code_grammar.generate("program");
        println!("{}. {}", i, result.text);
    }

    Ok(())
}
