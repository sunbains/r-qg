use std::fs::File;
use std::io::Write;
use std::fs;
use grammar_gen::utils::SqlNullValidator;
use grammar_gen::{Grammar, GrammarBuilder, GrammarConfig};

#[test]
fn test_load_from_file() {
    // Create a temporary test grammar file
    let test_file = "test_grammar.txt";
    let grammar_content = r#"# Test Grammar
       <start> ::= ["Hello", <subject>]
       <subject> ::= ["world"]
       <subject> ::= ["Rust"]
       "#;

    {
        let mut file = File::create(test_file).unwrap();
        file.write_all(grammar_content.as_bytes()).unwrap();
    }

    // Test loading the grammar
    let grammar = Grammar::from_file(test_file, "start").unwrap();
    
    // Verify the grammar was loaded correctly
    assert_eq!(grammar.start_symbol(), "start");
    assert!(grammar.has_non_terminal("subject"));
    
    // Generate some text
    let result = grammar.generate();
    assert!(result == "Hello world" || result == "Hello Rust");
    
    // Clean up
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_complex_grammar() {
    // Test a more complex grammar with nested production rules
    let mut grammar = Grammar::new("expression");
    
    // Add non-recursive rules first
    grammar.add_rule("expression", vec!["<term>"]).unwrap();
    grammar.add_rule("term", vec!["<factor>"]).unwrap();
    grammar.add_rule("factor", vec!["<number>"]).unwrap();
    
    // Then add recursive rules
    grammar.add_rule("expression", vec!["<term>", "+", "<expression>"]).unwrap();
    grammar.add_rule("term", vec!["<factor>", "*", "<term>"]).unwrap();
    grammar.add_rule("factor", vec!["(", "<expression>", ")"]).unwrap();
    
    // Add terminal values
    grammar.add_rule("number", vec!["0"]).unwrap();
    grammar.add_rule("number", vec!["1"]).unwrap();
    grammar.add_rule("number", vec!["2"]).unwrap();
    
    // Generate text - we won't test exact output since it's random,
    // but we'll ensure it doesn't crash and returns something
    let result = grammar.generate();
    assert!(!result.is_empty());
}

#[test]
fn test_null_handling() {
    // Create a simple SQL grammar with NULL values
    let mut grammar = GrammarBuilder::new("condition")
        .add_rule("condition", &["<column>", "<operator>", "<value>"])
        .add_rule("condition", &["<column>", "IS", "NULL"])
        .add_rule("column", &["status"])
        .add_rule("column", &["name"])
        .add_rule("operator", &["="])
        .add_rule("operator", &["!="])
        .add_rule("value", &["NULL"])
        .add_rule("value", &["'active'"])
        .build();
    
    // Test SQL NULL validation
    grammar = grammar.with_validator(Box::new(SqlNullValidator));
    
    // Generate multiple samples to increase the chance of testing NULL handling
    for _ in 0..10 {
        let result = grammar.generate();
        
        // We should never see "= NULL" in the output, it should be converted to "IS NULL"
        assert!(!result.contains("= NULL"));
        assert!(!result.contains("!= NULL"));
        
        // If NULL appears, it should be with IS or IS NOT
        if result.contains("NULL") {
            assert!(result.contains("IS NULL") || result.contains("IS NOT NULL"));
        }
    }
}

#[test]
fn test_grammar_config() {
    // Test custom configuration
    let mut config = GrammarConfig::default();
    config.auto_spacing = false;
    config.trim_output = false;
    
    let mut grammar = Grammar::with_config("test", config);
    
    grammar.add_rule("test", vec!["Hello", "world"]).unwrap();
    
    let result = grammar.generate();
    
    // Without auto spacing, we should get "Helloworld" (no space)
    assert_eq!(result, "Helloworld");
    
    // Test with auto spacing
    let mut config = GrammarConfig::default();
    config.auto_spacing = true;
    grammar.set_config(config);
    
    let result = grammar.generate();
    assert_eq!(result, "Hello world");
}

#[test]
fn test_empty_production() {
    // Test handling of empty productions
    let result = Grammar::parse_production("");
    assert!(result.is_err());
    
    if let Err(err) = result {
        assert!(format!("{}", err).contains("Empty production"));
    }
}

#[test]
fn test_unknown_nonterminal() {
    // Create a grammar with a reference to a non-existent non-terminal
    let mut grammar = Grammar::new("start");
    
    grammar.add_rule("start", vec!["<missing>"]).unwrap();
    
    // Should return the missing non-terminal as <missing>
    let result = grammar.generate();
    assert!(result.contains("<missing>"));
}

