use grammar_gen::utils::SqlNullValidator;
use grammar_gen::{Grammar, GrammarConfig};
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
    assert!(result.text == "Hello world" || result.text == "Hello Rust");

    // Clean up
    fs::remove_file(test_file).unwrap();
}

#[test]
fn test_complex_grammar() {
    // Test a more complex grammar with nested production rules
    // FIXME: It produces infinite recursion and strange output. Inifinite recursion is by design.
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
    println!("result: {}", result.text);
    assert!(!result.text.is_empty());
}

#[test]
fn test_null_handling() {
    // Create a simple SQL grammar with NULL values
    let mut grammar = Grammar::new();
    grammar
        .add_rule("condition", vec!["<column>", "<operator>", "<value>"])
        .unwrap();
    grammar
        .add_rule("condition", vec!["<column>", "IS", "NULL"])
        .unwrap();
    grammar.add_rule("column", vec!["status"]).unwrap();
    grammar.add_rule("column", vec!["name"]).unwrap();
    grammar.add_rule("operator", vec!["="]).unwrap();
    grammar.add_rule("operator", vec!["!="]).unwrap();
    grammar.add_rule("value", vec!["NULL"]).unwrap();
    grammar.add_rule("value", vec!["'active'"]).unwrap();

    // Test SQL NULL validation
    grammar = grammar.with_validator(Box::new(SqlNullValidator));

    // Generate multiple samples to increase the chance of testing NULL handling
    for _ in 0..10 {
        let result = grammar.generate("condition");

        // We should never see "= NULL" in the output, it should be converted to "IS NULL"
        assert!(!result.text.contains("= NULL"));
        assert!(!result.text.contains("!= NULL"));

        // If NULL appears, it should be with IS or IS NOT
        if result.text.contains("NULL") {
            assert!(result.text.contains("IS NULL") || result.text.contains("IS NOT NULL"));
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
    assert_eq!(result.text, "Helloworld");

    // Test with auto spacing
    let mut config = GrammarConfig::default();
    config.auto_spacing = true;
    grammar.set_config(config);

    let result = grammar.generate("test");
    assert_eq!(result.text, "Hello world");
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
    assert!(result.text.contains("<missing>"));
}

#[test]
fn test_complex_joins() {
    // Test complex join scenarios
    let mut grammar = Grammar::new();

    grammar
        .add_rule(
            "query",
            vec!["SELECT * FROM", "<table_name>", "<alias>", "<join_clause>"],
        )
        .unwrap();
    grammar
        .add_rule(
            "join_clause",
            vec!["INNER JOIN", "<table_name>", "<alias>", "ON", "<condition>"],
        )
        .unwrap();
    grammar
        .add_rule(
            "join_clause",
            vec!["LEFT JOIN", "<table_name>", "<alias>", "ON", "<condition>"],
        )
        .unwrap();
    grammar
        .add_rule(
            "condition",
            vec![
                "<alias>",
                ".",
                "<column_name>",
                "=",
                "<alias>",
                ".",
                "<column_name>",
            ],
        )
        .unwrap();
    grammar.add_rule("table_name", vec!["users"]).unwrap();
    grammar.add_rule("table_name", vec!["orders"]).unwrap();
    grammar.add_rule("table_name", vec!["products"]).unwrap();
    grammar.add_rule("alias", vec!["u"]).unwrap();
    grammar.add_rule("alias", vec!["o"]).unwrap();
    grammar.add_rule("alias", vec!["p"]).unwrap();
    grammar.add_rule("column_name", vec!["id"]).unwrap();
    grammar.add_rule("column_name", vec!["user_id"]).unwrap();
    grammar.add_rule("column_name", vec!["product_id"]).unwrap();

    println!("grammar: {:?}", grammar);

    // Generate multiple queries to test different join combinations
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("Generated query: {}", query.text);
        assert!(
            query.text.contains("JOIN"),
            "Query should contain JOIN: {}",
            query.text
        );
        assert!(
            query.text.contains("ON"),
            "Query should contain ON: {}",
            query.text
        );
        assert!(
            query.text.matches("=").count() >= 1,
            "Query should contain at least one =: {}",
            query.text
        );
    }
}

#[test]
fn test_ctes() {
    // Test Common Table Expressions
    let mut grammar = Grammar::new();

    grammar
        .add_rule("query", vec!["WITH", "<cte_list>", "<select_statement>"])
        .unwrap();
    grammar.add_rule("cte_list", vec!["<cte>"]).unwrap();
    grammar
        .add_rule("cte_list", vec!["<cte>", ",", "<cte_list>"])
        .unwrap();
    grammar
        .add_rule(
            "cte",
            vec!["<cte_name>", "AS", "(", "<select_statement>", ")"],
        )
        .unwrap();
    grammar.add_rule("cte_name", vec!["temp1"]).unwrap();
    grammar.add_rule("cte_name", vec!["temp2"]).unwrap();
    grammar
        .add_rule(
            "select_statement",
            vec!["SELECT", "*", "FROM", "<table_name>"],
        )
        .unwrap();
    grammar.add_rule("table_name", vec!["users"]).unwrap();
    grammar.add_rule("table_name", vec!["orders"]).unwrap();

    // Generate multiple CTE queries
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("query: {}", query.text);
        assert!(query.text.starts_with("WITH"));
        assert!(query.text.contains("AS"));
        assert!(query.text.contains("SELECT"));
    }
}

#[test]
fn test_fts_queries() {
    // Test Full Text Search queries
    let mut grammar = Grammar::new();

    grammar
        .add_rule(
            "query",
            vec![
                "SELECT",
                "<select_list>",
                "FROM",
                "<table_name>",
                "WHERE",
                "<fts_match>",
                "<fts_options>",
            ],
        )
        .unwrap();
    grammar.add_rule("select_list", vec!["*"]).unwrap();
    grammar.add_rule("table_name", vec!["documents"]).unwrap();
    grammar.add_rule("table_name", vec!["articles"]).unwrap();
    grammar
        .add_rule(
            "fts_match",
            vec![
                "MATCH",
                "(",
                "<column_list>",
                ")",
                "AGAINST",
                "(",
                "<fts_term>",
                "IN BOOLEAN MODE)",
            ],
        )
        .unwrap();
    grammar
        .add_rule("column_list", vec!["title", ",", "content"])
        .unwrap();
    grammar
        .add_rule("fts_term", vec!["'", "<fts_boolean_expr>", "'"])
        .unwrap();
    grammar
        .add_rule("fts_boolean_expr", vec!["+", "<string>", "-", "<string>"])
        .unwrap();
    grammar
        .add_rule("fts_boolean_expr", vec!["<string>", " AND ", "<string>"])
        .unwrap();
    grammar.add_rule("string", vec!["database"]).unwrap();
    grammar.add_rule("string", vec!["mysql"]).unwrap();
    grammar.add_rule("string", vec!["postgres"]).unwrap();
    grammar.add_rule("string", vec!["tidb"]).unwrap();
    grammar.add_rule("fts_options", vec![""]).unwrap();
    grammar
        .add_rule("fts_options", vec!["AS relevance_score"])
        .unwrap();

    // Generate multiple FTS queries
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("query: {}", query.text);
        assert!(query.text.contains("MATCH"));
        assert!(query.text.contains("AGAINST"));
        assert!(query.text.contains("IN BOOLEAN MODE"));
    }
}

#[test]
fn test_complex_fts_with_ctes() {
    // Test combining FTS with CTEs
    let mut grammar = Grammar::new();

    grammar
        .add_rule(
            "query",
            vec![
                "WITH",
                "<cte>",
                "SELECT",
                "<select_list>",
                "FROM",
                "<table_reference>",
                "WHERE",
                "<fts_match>",
            ],
        )
        .unwrap();
    grammar
        .add_rule(
            "cte",
            vec![
                "search_results AS (SELECT * FROM documents WHERE MATCH (content) AGAINST ('",
                "<string>",
                "' IN BOOLEAN MODE))",
            ],
        )
        .unwrap();
    grammar.add_rule("select_list", vec!["*"]).unwrap();
    grammar
        .add_rule("table_reference", vec!["search_results sr"])
        .unwrap();
    grammar
        .add_rule(
            "table_reference",
            vec!["search_results sr INNER JOIN documents d ON sr.id = d.id"],
        )
        .unwrap();
    grammar
        .add_rule(
            "fts_match",
            vec![
                "MATCH (d.content) AGAINST ('",
                "<string>",
                "' IN NATURAL LANGUAGE MODE)",
            ],
        )
        .unwrap();
    grammar.add_rule("string", vec!["database"]).unwrap();
    grammar.add_rule("string", vec!["search"]).unwrap();
    grammar.add_rule("string", vec!["query"]).unwrap();

    // Generate complex queries combining FTS and CTEs
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("query: {}", query.text);
        assert!(query.text.starts_with("WITH"));
        assert!(query.text.contains("MATCH"));
        assert!(query.text.contains("AGAINST"));
        assert!(query.text.contains("SELECT"));
    }
}

#[test]
fn test_fts_with_relevance_scoring() {
    // Test FTS with relevance scoring and complex ordering
    let mut grammar = Grammar::new();

    grammar
        .add_rule(
            "query",
            vec![
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
        .unwrap();
    grammar.add_rule("select_list", vec!["*"]).unwrap();
    grammar.add_rule("table_name", vec!["documents"]).unwrap();
    grammar
        .add_rule(
            "fts_match",
            vec![
                "MATCH (title, content) AGAINST ('",
                "<fts_term>",
                "' IN NATURAL LANGUAGE MODE)",
            ],
        )
        .unwrap();
    grammar.add_rule("fts_term", vec!["<string>"]).unwrap();
    grammar.add_rule("string", vec!["database"]).unwrap();
    grammar.add_rule("string", vec!["search"]).unwrap();
    grammar
        .add_rule(
            "fts_score",
            vec!["MATCH (title, content) AGAINST ('", "<fts_term>", "')"],
        )
        .unwrap();
    grammar
        .add_rule("order_by", vec!["ORDER BY relevance DESC, created_at DESC"])
        .unwrap();

    // Generate FTS queries with relevance scoring
    for _ in 0..5 {
        let query = grammar.generate("query");
        println!("query: {}", query.text);
        assert!(query.text.contains("MATCH"));
        assert!(query.text.contains("AS relevance"));
        assert!(query.text.contains("ORDER BY"));
        assert!(query.text.contains("DESC"));
    }
}
