use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

use crate::utils::{GrammarError, GrammarValidator, NoopValidator, Result};

/// Represents an element in the grammar, either a terminal or a non-terminal
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// A terminal symbol (literal text)
    Terminal(String),
    /// A non-terminal symbol (reference to another rule)
    NonTerminal(String),
}

/// Represents a production rule in the grammar
#[derive(Debug, Clone)]
pub struct Production {
    /// The sequence of elements in this production
    pub elements: Vec<Element>,
}

/// Configuration options for grammar behavior
#[derive(Debug, Clone)]
pub struct GrammarConfig {
    /// Whether to add spaces between elements automatically
    pub auto_spacing: bool,
    /// Whether to trim whitespace from output
    pub trim_output: bool,
    /// Maximum recursion depth for expansion (to prevent infinite recursion)
    pub max_recursion_depth: usize,
}

impl Default for GrammarConfig {
    fn default() -> Self {
        GrammarConfig {
            auto_spacing: true,
            trim_output: true,
            max_recursion_depth: 100,
        }
    }
}

/// The complete grammar with rules for generating text
#[derive(Debug, Clone)]
pub struct Grammar {
    /// The rules mapping non-terminals to productions
    rules: HashMap<String, Vec<Production>>,
    /// The starting symbol for generation
    start_symbol: String,
    /// Configuration options
    config: GrammarConfig,
    /// Optional validator for post-processing generated text
    validator: Box<dyn GrammarValidator>,
}

/// Token types for the grammar parser
#[derive(Debug, Clone, PartialEq)]
enum Token {
    NonTerminal(String),
    Terminal(String),
    RuleSeparator, // ::=
    ListStart,     // [
    ListEnd,       // ]
    Comma,         // ,
    EndOfFile,
}

/// Tokenizer for the grammar parser
struct Tokenizer<'a> {
    chars: Peekable<Chars<'a>>,
    current_line: String,
    line_number: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Tokenizer {
            chars: input.chars().peekable(),
            current_line: String::new(),
            line_number: 1,
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        loop {
            self.skip_empty_lines();
            self.skip_whitespace();
            self.skip_comments();

            if self.chars.peek().is_none() {
                return Ok(Token::EndOfFile);
            }

            if let Some(&c) = self.chars.peek() {
                if c.is_whitespace() || c == '\n' {
                    self.chars.next();
                    continue;
                }
            }

            break;
        }

        match self.chars.peek() {
            Some(&'<') => self.parse_non_terminal(),
            Some(&'[') => {
                self.chars.next();
                Ok(Token::ListStart)
            }
            Some(&']') => {
                self.chars.next();
                Ok(Token::ListEnd)
            }
            Some(&',') => {
                self.chars.next();
                Ok(Token::Comma)
            }
            Some(&':') => self.parse_rule_separator(),
            Some(&_) => self.parse_terminal(),
            None => Ok(Token::EndOfFile),
        }
    }

    fn skip_empty_lines(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c != '\n' {
                break;
            }
            self.chars.next();
            self.line_number += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() || c == '\n' {
                self.chars.next();
            } else {
                break;
            }
        }
    }

    fn skip_to_end_of_line(&mut self) {
        while let Some(&c) = self.chars.peek() {
            self.chars.next();

            if c == '\n' {
                self.line_number += 1;
                break;
            }
        }
    }

    fn skip_comments(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == '#' {
                self.skip_to_end_of_line();
            } else {
                break;
            }
        }
    }

    fn parse_non_terminal(&mut self) -> Result<Token> {
        self.chars.next(); // Consume '<'
        let mut name = String::new();

        while let Some(&c) = self.chars.peek() {
            if c == '>' {
                self.chars.next();
                return Ok(Token::NonTerminal(name));
            }
            name.push(c);
            self.current_line.push(c);
            self.chars.next();
        }

        Err(GrammarError::Parse(format!(
            "Unclosed non-terminal at line {}: {}",
            self.line_number, self.current_line
        )))
    }

    fn parse_terminal(&mut self) -> Result<Token> {
        let mut value = String::new();
        let mut in_quotes = false;
        let mut quote_char = None;
        let mut is_escaped = false;

        // Check if we're starting with a quote
        if let Some(&c) = self.chars.peek() {
            if c == '"' || c == '\'' {
                in_quotes = true;
                quote_char = Some(c);
                self.current_line.push(c);
                self.chars.next();
            }
        }

        while let Some(&c) = self.chars.peek() {
            match c {
                c if is_escaped => {
                    // Add the escaped character literally
                    value.push(c);
                    self.current_line.push(c);
                    self.chars.next();
                    is_escaped = false;
                }
                '\\' => {
                    // Next character is escaped
                    is_escaped = true;
                    self.current_line.push(c);
                    self.chars.next();
                }
                c if in_quotes && Some(c) == quote_char => {
                    self.chars.next();
                    return Ok(Token::Terminal(value));
                }
                c if in_quotes => {
                    value.push(c);
                    self.current_line.push(c);
                    self.chars.next();
                }
                c if !c.is_whitespace() && c != ',' && c != ']' && c != '>' => {
                    value.push(c);
                    self.current_line.push(c);
                    self.chars.next();
                }
                _ => break,
            }
        }

        if in_quotes {
            Err(GrammarError::Parse(format!(
                "Unclosed quotes at line {}: {}",
                self.line_number, self.current_line
            )))
        } else {
            Ok(Token::Terminal(value))
        }
    }

    fn parse_rule_separator(&mut self) -> Result<Token> {
        let mut chars = String::new();
        for _ in 0..3 {
            if let Some(&c) = self.chars.peek() {
                chars.push(c);
                self.current_line.push(c);
                self.chars.next();
            }
        }

        if chars == "::=" {
            Ok(Token::RuleSeparator)
        } else {
            Err(GrammarError::Parse(format!(
                "Invalid rule separator at line {}: {}",
                self.line_number, self.current_line
            )))
        }
    }
}

/// Parser for the grammar rules
struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Result<Self> {
        let mut tokenizer = Tokenizer::new(input);
        let current_token = tokenizer.next_token()?;

        Ok(Parser {
            tokenizer,
            current_token,
        })
    }

    fn advance(&mut self) -> Result<()> {
        self.current_token = self.tokenizer.next_token()?;
        Ok(())
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        if self.current_token == expected {
            self.advance()?;
            Ok(())
        } else {
            Err(GrammarError::Parse(format!(
                "Expected {:?}, found {:?}",
                expected, self.current_token
            )))
        }
    }

    fn parse_rule(&mut self) -> Result<(String, Production)> {
        let non_terminal = match &self.current_token {
            Token::NonTerminal(name) => name.clone(),
            _ => return Err(GrammarError::Parse("Expected non-terminal".to_string())),
        };

        self.advance()?;
        self.expect(Token::RuleSeparator)?;
        self.expect(Token::ListStart)?;

        let production = self.parse_production()?;

        self.expect(Token::ListEnd)?;

        Ok((non_terminal, production))
    }

    fn parse_production(&mut self) -> Result<Production> {
        let mut elements = Vec::new();

        loop {
            match &self.current_token {
                Token::NonTerminal(name) => {
                    elements.push(Element::NonTerminal(name.clone()));
                    self.advance()?;
                }
                Token::Terminal(value) => {
                    elements.push(Element::Terminal(value.clone()));
                    self.advance()?;
                }
                Token::Comma => {
                    self.advance()?;
                }
                _ => break, // Allow other tokens to end the production
            }
        }

        if elements.is_empty() {
            return Err(GrammarError::EmptyProduction(
                "Empty production".to_string(),
            ));
        }

        Ok(Production { elements })
    }
}

impl Grammar {
    /// Create a new empty grammar with a specified start symbol
    pub fn new(start_symbol: &str) -> Self {
        Grammar {
            rules: HashMap::new(),
            start_symbol: start_symbol.to_string(),
            config: GrammarConfig::default(),
            validator: Box::new(NoopValidator),
        }
    }

    /// Create a new grammar with custom configuration
    pub fn with_config(start_symbol: &str, config: GrammarConfig) -> Self {
        Grammar {
            rules: HashMap::new(),
            start_symbol: start_symbol.to_string(),
            config,
            validator: Box::new(NoopValidator),
        }
    }

    /// Set a validator for this grammar
    pub fn with_validator(mut self, validator: Box<dyn GrammarValidator>) -> Self {
        self.validator = validator;
        self
    }

    /// Parse a grammar from a file
    pub fn from_file<P: AsRef<Path>>(path: P, start_symbol: &str) -> Result<Self> {
        let file = File::open(path).map_err(GrammarError::Io)?;
        let reader = io::BufReader::new(file);
        let mut grammar = Grammar::new(start_symbol);

        let mut current_content = String::new();
        for line in reader.lines() {
            let line = line.map_err(GrammarError::Io)?;
            current_content.push_str(&line);
            current_content.push('\n');
        }

        let mut parser = Parser::new(&current_content)?;

        while parser.current_token != Token::EndOfFile {
            let (non_terminal, production) = parser.parse_rule()?;

            if grammar.rules.contains_key(&non_terminal) {
                if let Some(productions) = grammar.rules.get_mut(&non_terminal) {
                    productions.push(production);
                }
            } else {
                grammar.rules.insert(non_terminal, vec![production]);
            }
        }

        // Validate that the start symbol exists
        if !grammar.rules.contains_key(&grammar.start_symbol) {
            return Err(GrammarError::UnknownNonTerminal(
                grammar.start_symbol.clone(),
            ));
        }

        Ok(grammar)
    }

    /// Parse a production rule from a string
    pub fn parse_production(elements_str: &str) -> Result<Production> {
        let mut parser = Parser::new(elements_str)?;
        parser.parse_production()
    }

    /// Add a rule to the grammar
    pub fn add_rule(&mut self, non_terminal: &str, elements: Vec<&str>) -> Result<&mut Self> {
        let production = self.parse_elements(elements)?;

        if self.rules.contains_key(non_terminal) {
            if let Some(productions) = self.rules.get_mut(non_terminal) {
                productions.push(production);
            }
        } else {
            self.rules
                .insert(non_terminal.to_string(), vec![production]);
        }

        Ok(self)
    }

    /// Parse a vector of strings into a Production
    fn parse_elements(&self, elements: Vec<&str>) -> Result<Production> {
        let mut parsed_elements = Vec::new();

        for element in elements {
            if element.starts_with('<') && element.ends_with('>') {
                // This is a non-terminal
                let name = element[1..element.len() - 1].to_string();
                parsed_elements.push(Element::NonTerminal(name));
            } else {
                // This is a terminal
                parsed_elements.push(Element::Terminal(element.to_string()));
            }
        }

        if parsed_elements.is_empty() {
            return Err(GrammarError::EmptyProduction(
                "Empty elements vector".to_string(),
            ));
        }

        Ok(Production {
            elements: parsed_elements,
        })
    }

    /// Generate random text based on the grammar rules
    pub fn generate(&self) -> String {
        // Start generation from the start symbol, with initial depth 0
        let result = self.expand_non_terminal(&self.start_symbol, 0);

        // Apply validation/post-processing
        let result = self.validator.validate(&result);

        // Apply final trimming if configured
        if self.config.trim_output {
            result.trim().to_string()
        } else {
            result
        }
    }

    /// Recursively expand a non-terminal symbol
    fn expand_non_terminal(&self, symbol: &str, depth: usize) -> String {
        // Prevent excessive recursion
        if depth >= self.config.max_recursion_depth {
            return format!("<recursion_limit_exceeded:{}>", symbol);
        }

        let mut rng = rand::thread_rng();

        if let Some(productions) = self.rules.get(symbol) {
            // Randomly select one of the productions
            let production_idx = rng.gen_range(0..productions.len());
            let production = &productions[production_idx];

            // Expand all elements in the selected production
            let mut result = String::new();
            for element in &production.elements {
                match element {
                    Element::Terminal(text) => {
                        // Add space if needed
                        if self.config.auto_spacing
                            && !result.is_empty()
                            && !result.ends_with(' ')
                            && !text.starts_with(' ')
                        {
                            result.push(' ');
                        }
                        result.push_str(text);
                    }
                    Element::NonTerminal(name) => {
                        let expanded = self.expand_non_terminal(name, depth + 1);
                        // Add space if needed
                        if self.config.auto_spacing
                            && !result.is_empty()
                            && !result.ends_with(' ')
                            && !expanded.starts_with(' ')
                        {
                            result.push(' ');
                        }
                        result.push_str(&expanded);
                    }
                }
            }
            result
        } else {
            // Unknown non-terminal, return it as is (for debugging)
            format!("<{}>", symbol)
        }
    }

    /// Check if the grammar contains a specific non-terminal
    pub fn has_non_terminal(&self, name: &str) -> bool {
        self.rules.contains_key(name)
    }

    /// Get a reference to the grammar's rules
    pub fn rules(&self) -> &HashMap<String, Vec<Production>> {
        &self.rules
    }

    /// Get the start symbol
    pub fn start_symbol(&self) -> &str {
        &self.start_symbol
    }

    /// Get a reference to the grammar's configuration
    pub fn config(&self) -> &GrammarConfig {
        &self.config
    }

    /// Set a new configuration
    pub fn set_config(&mut self, config: GrammarConfig) {
        self.config = config;
    }

    /// Set the maximum recursion depth
    pub fn set_recursion_depth(&mut self, depth: usize) {
        self.config.max_recursion_depth = depth;
    }
}

/// Builder for constructing Grammar instances
pub struct GrammarBuilder {
    grammar: Grammar,
}

impl GrammarBuilder {
    /// Create a new grammar builder with default config
    pub fn new(start_symbol: &str) -> Self {
        GrammarBuilder {
            grammar: Grammar::new(start_symbol),
        }
    }

    /// Set the configuration
    pub fn config(mut self, config: GrammarConfig) -> Self {
        self.grammar.config = config;
        self
    }

    /// Add a rule to the grammar
    pub fn add_rule(mut self, non_terminal: &str, elements: &[&str]) -> Self {
        // Properly handle errors in builder pattern
        if let Err(e) = self.grammar.add_rule(non_terminal, elements.to_vec()) {
            panic!(
                "Failed to add rule for non-terminal '{}': {}",
                non_terminal, e
            );
        }
        self
    }

    /// Set a validator
    pub fn validator(mut self, validator: Box<dyn GrammarValidator>) -> Self {
        self.grammar.validator = validator;
        self
    }

    /// Build the grammar
    pub fn build(self) -> Grammar {
        self.grammar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::noop_validator;

    #[test]
    fn test_parse_production() {
        let input = r#""SELECT", <column_ref>, "FROM", <table_name>"#;
        let production = Grammar::parse_production(input).unwrap();

        assert_eq!(production.elements.len(), 4);

        match &production.elements[0] {
            Element::Terminal(s) => assert_eq!(s, "SELECT"),
            _ => panic!("Expected Terminal"),
        }

        match &production.elements[1] {
            Element::NonTerminal(s) => assert_eq!(s, "column_ref"),
            _ => panic!("Expected NonTerminal"),
        }

        match &production.elements[2] {
            Element::Terminal(s) => assert_eq!(s, "FROM"),
            _ => panic!("Expected Terminal"),
        }

        match &production.elements[3] {
            Element::NonTerminal(s) => assert_eq!(s, "table_name"),
            _ => panic!("Expected NonTerminal"),
        }
    }

    #[test]
    fn test_grammar_generate() {
        let mut grammar = Grammar::new("start");

        // Define a simple grammar
        grammar
            .add_rule("start", vec!["Hello", "<subject>"])
            .unwrap();
        grammar.add_rule("subject", vec!["world"]).unwrap();
        grammar.add_rule("subject", vec!["Rust"]).unwrap();

        // Generate some text
        let result = grammar.generate();
        assert!(result == "Hello world" || result == "Hello Rust");
    }

    #[test]
    fn test_grammar_builder() {
        let grammar = GrammarBuilder::new("greeting")
            .add_rule("greeting", &["Hello", "<subject>"])
            .add_rule("subject", &["world"])
            .add_rule("subject", &["Rust", "programmer"])
            .validator(noop_validator())
            .build();

        let result = grammar.generate();
        assert!(result == "Hello world" || result == "Hello Rust programmer");
    }

    #[test]
    fn test_recursion_limit() {
        let mut grammar = Grammar::new("recursive");
        let mut config = GrammarConfig::default();
        config.max_recursion_depth = 5;
        grammar.set_config(config);

        // Create a recursive grammar
        grammar
            .add_rule("recursive", vec!["<recursive>", "loop"])
            .unwrap();

        // Should hit recursion limit
        let result = grammar.generate();
        assert!(result.contains("recursion_limit_exceeded"));
    }
}
