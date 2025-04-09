use grammar_gen::utils::SqlNullValidator;
use grammar_gen::{Grammar, GrammarBuilder, GrammarConfig};
use std::fs;
use std::fs::File;
use std::io::Write;

#[test]
fn test_load_from_file() {
    // Create a temporary test grammar file
    let test_file = "test_grammar.txt";

    // Test for comments to be ignored too.
    let grammar_content = r#"
       # Test comment
       <start>   ::= ["Hello", <subject>]
       <subject> ::= ["world"]
       <subject> ::= ["Rust"]
       "#;

    {
        let mut file = File::create(test_file).unwrap();
        file.write_all(grammar_content.as_bytes()).unwrap();
    }

    // Test loading the grammar
    let grammar = Grammar::from_file(test_file).unwrap();

    // Verify the grammar was loaded correctly
    assert!(grammar.has_non_terminal("subject"));

    // Generate some text
    let result = grammar.generate("start");
    assert!(result == "Hello world" || result == "Hello Rust");

    // Clean up
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_complex_grammar() {
    // Test a more complex grammar with nested production rules
    let mut grammar = Grammar::new();

    // Add non-recursive rules first
    grammar.add_rule("expression", vec!["<term>"]).unwrap();
    grammar.add_rule("term", vec!["<factor>"]).unwrap();
    grammar.add_rule("factor", vec!["<number>"]).unwrap();

    // Then add recursive rules
    grammar
        .add_rule("expression", vec!["<term>", "+", "<expression>"])
        .unwrap();
    grammar
        .add_rule("term", vec!["<factor>", "*", "<term>"])
        .unwrap();
    grammar
        .add_rule("factor", vec!["(", "<expression>", ")"])
        .unwrap();

    // Add terminal values
    grammar.add_rule("number", vec!["0"]).unwrap();
    grammar.add_rule("number", vec!["1"]).unwrap();
    grammar.add_rule("number", vec!["2"]).unwrap();

    // Generate text - we won't test exact output since it's random,
    // but we'll ensure it doesn't crash and returns something
    let result = grammar.generate("expression");
    println!("result: {}", result);
    assert!(!result.is_empty());
}

#[test]
fn test_null_handling() {
    // Create a simple SQL grammar with NULL values
    let mut grammar = GrammarBuilder::new()
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
        let result = grammar.generate("condition");

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

    let mut grammar = Grammar::with_config(config);

    grammar.add_rule("test", vec!["Hello", "world"]).unwrap();

    let result = grammar.generate("test");

    // Without auto spacing, we should get "Helloworld" (no space)
    assert_eq!(result, "Helloworld");

    // Test with auto spacing
    let mut config = GrammarConfig::default();
    config.auto_spacing = true;
    grammar.set_config(config);

    let result = grammar.generate("test");
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
    let mut grammar = Grammar::new();

    grammar.add_rule("start", vec!["<missing>"]).unwrap();

    // Should return the missing non-terminal as <missing>
    let result = grammar.generate("start");
    assert!(result.contains("<missing>"));
}

#[test]
fn test_complex_joins() {
    // Test complex join scenarios
    let grammar = GrammarBuilder::new()
        .add_rule(
            "query",
            &["SELECT * FROM", "<table_name>", "<alias>", "<join_clause>"],
        )
        .add_rule(
            "join_clause",
            &["INNER JOIN", "<table_name>", "<alias>", "ON", "<condition>"],
        )
        .add_rule(
            "join_clause",
            &["LEFT JOIN", "<table_name>", "<alias>", "ON", "<condition>"],
        )
        .add_rule(
            "condition",
            &[ "<alias>", ".", "<column_name>", "=", "<alias>", ".", "<column_name>", ],
        )
        .add_rule("table_name", &["users"])
        .add_rule("table_name", &["orders"])
        .add_rule("table_name", &["products"])
        .add_rule("alias", &["u"])
        .add_rule("alias", &["o"])
        .add_rule("alias", &["p"])
        .add_rule("column_name", &["id"])
        .add_rule("column_name", &["user_id"])
        .add_rule("column_name", &["product_id"])
        .build();

    println!("grammar: {:?}", grammar);

    // Generate multiple queries to test different join combinations
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("Generated query: {}", query);
        assert!(
            query.contains("JOIN"),
            "Query should contain JOIN: {}",
            query
        );
        assert!(query.contains("ON"), "Query should contain ON: {}", query);
        assert!(
            query.matches("=").count() >= 1,
            "Query should contain at least one =: {}",
            query
        );
    }
}

#[test]
fn test_ctes() {
    // Test Common Table Expressions
    let grammar = GrammarBuilder::new()
        .add_rule("query", &["WITH", "<cte_list>", "<select_statement>"])
        .add_rule("cte_list", &["<cte>"])
        .add_rule("cte_list", &["<cte>", ",", "<cte_list>"])
        .add_rule("cte", &["<cte_name>", "AS", "(", "<select_statement>", ")"])
        .add_rule("cte_name", &["temp1"])
        .add_rule("cte_name", &["temp2"])
        .add_rule("select_statement", &["SELECT", "*", "FROM", "<table_name>"])
        .add_rule("table_name", &["users"])
        .add_rule("table_name", &["orders"])
        .build();

    // Generate multiple CTE queries
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("query: {}", query);
        assert!(query.starts_with("WITH"));
        assert!(query.contains("AS"));
        assert!(query.contains("SELECT"));
    }
}

#[test]
fn test_fts_queries() {
    // Test Full Text Search queries
    let grammar = GrammarBuilder::new()
        .add_rule(
            "query",
            &[ "SELECT", "<select_list>", "FROM", "<table_name>", "WHERE", "<fts_match>", "<fts_options>", ],
        )
        .add_rule("select_list", &["*"])
        .add_rule("table_name", &["documents"])
        .add_rule("table_name", &["articles"])
        .add_rule(
            "fts_match",
            &[ "MATCH", "(", "<column_list>", ")", "AGAINST", "(", "<fts_term>", "IN BOOLEAN MODE)", ],)
        .add_rule("column_list", &["title", ",", "content"])
        .add_rule("fts_term", &["'", "<fts_boolean_expr>", "'"])
        .add_rule("fts_boolean_expr", &["+", "<string>", "-", "<string>"])
        .add_rule("fts_boolean_expr", &["<string>", "AND", "<string>"])
        .add_rule("string", &["database"])
        .add_rule("string", &["mysql"])
        .add_rule("string", &["postgres"])
        .add_rule("string", &["tidb"])
        .add_rule("fts_options", &["ORDER BY", "<fts_score>", "DESC"])
        .add_rule(
            "fts_score",
            &[ "MATCH", "(", "title", ",", "content", ")", "AGAINST", "(", "<fts_term>", ")", ],
        )
        .build();

    // Generate multiple FTS queries
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("query: {}", query);
        assert!(query.contains("MATCH"));
        assert!(query.contains("AGAINST"));
        assert!(query.contains("IN BOOLEAN MODE"));
        assert!(query.contains("ORDER BY"));
    }
}

#[test]
fn test_complex_fts_with_ctes() {
    // Test combining FTS with CTEs
    let grammar = GrammarBuilder::new()
        .add_rule(
            "query",
            &[ "WITH", "<cte>", "SELECT", "<select_list>", "FROM", "<table_reference>", "WHERE", "<fts_match>", ],
        )
        .add_rule(
            "cte",
            &[
                "search_results AS (SELECT * FROM documents WHERE MATCH (content) AGAINST ('",
                "<string>",
                "' IN BOOLEAN MODE))",
            ],
        )
        .add_rule("select_list", &["*"])
        .add_rule("table_reference", &["search_results sr"])
        .add_rule(
            "table_reference",
            &["search_results sr INNER JOIN documents d ON sr.id = d.id"],
        )
        .add_rule(
            "fts_match",
            &[
                "MATCH (d.content) AGAINST ('",
                "<string>",
                "' IN NATURAL LANGUAGE MODE)",
            ],
        )
        .add_rule("string", &["database"])
        .add_rule("string", &["search"])
        .add_rule("string", &["query"])
        .build();

    // Generate complex queries combining FTS and CTEs
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("query: {}", query);
        assert!(query.starts_with("WITH"));
        assert!(query.contains("MATCH"));
        assert!(query.contains("AGAINST"));
        assert!(query.contains("SELECT"));
    }
}

#[test]
fn test_fts_with_relevance_scoring() {
    // Test FTS with relevance scoring and complex ordering
    let grammar = GrammarBuilder::new()
        .add_rule(
            "query",
            &[
                "SELECT",
                "<select_list>",
                "<fts_score>",
                "AS",
                "relevance",
                "FROM",
                "<table_name>",
                "WHERE",
                "<fts_match>",
                "<order_by>",
            ],
        )
        .add_rule("select_list", &["*"])
        .add_rule("table_name", &["documents"])
        .add_rule(
            "fts_match",
            &[
                "MATCH (title, content) AGAINST ('",
                "<fts_term>",
                "' IN NATURAL LANGUAGE MODE)",
            ],
        )
        .add_rule("fts_term", &["<string>"])
        .add_rule("string", &["database"])
        .add_rule("string", &["search"])
        .add_rule(
            "fts_score",
            &["MATCH (title, content) AGAINST ('", "<fts_term>", "')"],
        )
        .add_rule("order_by", &["ORDER BY relevance DESC, created_at DESC"])
        .build();

    // Generate FTS queries with relevance scoring
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("query: {}", query);
        assert!(query.contains("MATCH"));
        assert!(query.contains("AS relevance"));
        assert!(query.contains("ORDER BY"));
        assert!(query.contains("DESC"));
    }
}
