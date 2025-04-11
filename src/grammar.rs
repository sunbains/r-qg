use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

use crate::utils::{GrammarError, GrammarValidator, NoopValidator, Result};
#[derive(Debug, Clone)]
pub struct QueryAstNode {
    pub element_type: String,
    pub value: String,
    pub children: Vec<QueryAstNode>,
}

impl QueryAstNode {
    /// Convert a node to its string representation
    pub fn to_string(&self) -> String {
        match self.element_type.as_str() {
            "terminal" => self.value.clone(),
            "non_terminal" => {
                let mut result = String::new();
                for child in &self.children {
                    let child_str = child.to_string();
                    if !result.is_empty()
                        && !result.ends_with('(')
                        && !child_str.starts_with(')')
                        && !child_str.starts_with(',')
                        && !result.ends_with(',')
                    {
                        result.push(' ');
                    }
                    result.push_str(&child_str);
                }
                result
            }
            "undefined" => format!("<{}>", self.value),
            "error" => format!("<{}>", self.value),
            _ => format!("<unknown:{}>", self.value),
        }
    }

    /// Get a debug representation showing node types
    pub fn to_debug_string(&self) -> String {
        match self.element_type.as_str() {
            "terminal" => format!("T({})", self.value),
            "non_terminal" => {
                let mut result = format!("NT({})[", self.value);
                for (i, child) in self.children.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&child.to_debug_string());
                }
                result.push(']');
                result
            }
            "undefined" => format!("UNDEF({})", self.value),
            "error" => format!("ERR({})", self.value),
            _ => format!("UNKNOWN({})", self.value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryAst {
    pub text: String,       // The generated text
    pub type_name: String,  // The starting non-terminal
    pub root: QueryAstNode, // Root node of the AST
}

impl QueryAst {
    /// Print the AST in a readable format
    pub fn print(&self) {
        println!("AST for '{}' (type: {}):", self.text, self.type_name);
        self.print_node(&self.root, 0);
    }

    fn print_node(&self, node: &QueryAstNode, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}└─ {} : '{}'", indent, node.element_type, node.value);

        for child in &node.children {
            self.print_node(child, depth + 1);
        }
    }

    /// Helper to get all nodes of a specific type
    pub fn find_nodes(&self, element_type: &str) -> Vec<&QueryAstNode> {
        let mut result = Vec::new();
        self.find_nodes_recursive(&self.root, element_type, &mut result);
        result
    }

    fn find_nodes_recursive<'a>(
        &'a self,
        node: &'a QueryAstNode,
        element_type: &str,
        result: &mut Vec<&'a QueryAstNode>,
    ) {
        if node.element_type == element_type {
            result.push(node);
        }

        for child in &node.children {
            self.find_nodes_recursive(child, element_type, result);
        }
    }

    /// Performs a default transformation by traversing the AST and
    /// returning a string representation of the nodes
    pub fn transform(&self) -> String {
        self.root.to_string()
    }

    /// Transforms the AST using a custom transformation function
    pub fn transform_with<F>(&self, transformer: F) -> String
    where
        F: Fn(&QueryAstNode) -> String,
    {
        transformer(&self.root)
    }

    /// Helper function for recursive transformation with a custom function
    #[allow(dead_code)]
    fn transform_with_helper<F>(node: &QueryAstNode, transformer: F) -> String
    where
        F: Fn(&QueryAstNode) -> String + Copy,
    {
        transformer(node)
    }

    /// Creates a new QueryAst from an existing AST but with a different root
    pub fn with_root(&self, new_root: QueryAstNode) -> Self {
        QueryAst {
            text: new_root.to_string(),
            type_name: self.type_name.clone(),
            root: new_root,
        }
    }

    /// Apply a transformation that modifies the AST structure
    pub fn transform_ast<F>(&self, transformer: F) -> Self
    where
        F: Fn(&QueryAstNode) -> QueryAstNode,
    {
        let new_root = transformer(&self.root);
        self.with_root(new_root)
    }

    /// Returns a debug representation of the AST showing node types
    pub fn to_debug_string(&self) -> String {
        self.root.to_debug_string()
    }
}

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
    Quote,         // '
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
            Some(&'\\') => {
                self.chars.next();
                self.chars.next();
                Ok(Token::Quote)
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
            if c == '\n' {
                self.line_number += 1;
            }
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
                break;
            }
        }
    }

    fn skip_comments(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == '#' {
                self.skip_to_end_of_line();
                self.line_number += 1;
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
                Token::Quote => {
                    elements.push(Element::Terminal("'".to_string()));
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
    pub fn new() -> Self {
        Grammar {
            rules: HashMap::new(),
            config: GrammarConfig::default(),
            validator: Box::new(NoopValidator),
        }
    }

    /// Create a new grammar with custom configuration
    pub fn with_config(config: GrammarConfig) -> Self {
        Grammar {
            rules: HashMap::new(),
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path).map_err(GrammarError::Io)?;
        let reader = io::BufReader::new(file);
        let mut grammar = Grammar::new();

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
    pub fn generate(&self, start_symbol: &str) -> QueryAst {
        // Create root node for the AST
        let root_node = QueryAstNode {
            element_type: "non_terminal".to_string(),
            value: start_symbol.to_string(),
            children: Vec::new(),
        };

        // Start generation from the start symbol
        let (result, ast_root) = self.expand_non_terminal(start_symbol, root_node);

        // Apply validation/post-processing
        let result = self.validator.validate(&result);

        // Apply final trimming if configured
        let text = if self.config.trim_output {
            result.trim().to_string()
        } else {
            result
        };

        QueryAst {
            text,
            type_name: start_symbol.to_string(),
            root: ast_root,
        }
    }

    /// Recursively expand a non-terminal symbol, now returning both the text and AST node
    fn expand_non_terminal(&self, symbol: &str, ast_node: QueryAstNode) -> (String, QueryAstNode) {
        let mut depth = 0;
        let mut rng = rand::thread_rng();
        let mut tokens = Vec::new();

        // Create a stack for production elements and AST nodes
        // Each stack frame contains (Element, AST_Node, Parent_Index)
        // where Parent_Index is the index in ast_nodes of the parent node
        let mut stack: Vec<(Element, usize)> = Vec::new();
        let mut ast_nodes: Vec<QueryAstNode> = Vec::new();

        // Start with the root node
        ast_nodes.push(ast_node);
        let root_idx = 0;

        // Push the initial element with the root as parent
        stack.push((Element::NonTerminal(symbol.to_string()), root_idx));

        while let Some((element, parent_idx)) = stack.pop() {
            match element {
                Element::Terminal(text) => {
                    tokens.push(text.clone());

                    // Create terminal node
                    let terminal_node = QueryAstNode {
                        element_type: "terminal".to_string(),
                        value: text,
                        children: Vec::new(),
                    };

                    // Add it as a child to its parent
                    ast_nodes[parent_idx].children.push(terminal_node);
                }
                Element::NonTerminal(name) => {
                    if let Some(productions) = self.rules.get(&name) {
                        // Choose a random production
                        let production_idx = rng.gen_range(0..productions.len());
                        let production = &productions[production_idx];

                        // Create a non-terminal node
                        let non_terminal_node = QueryAstNode {
                            element_type: "non_terminal".to_string(),
                            value: name.clone(),
                            children: Vec::new(),
                        };

                        // Add node to the tree and get its index
                        ast_nodes.push(non_terminal_node);
                        let node_idx = ast_nodes.len() - 1;

                        // Split the vector to get non-overlapping mutable references
                        let (left, right) = ast_nodes.split_at_mut(parent_idx + 1);
                        left[parent_idx].children.push(right[0].clone());

                        // Push elements and corresponding AST nodes in reverse order
                        if depth < self.config.max_recursion_depth {
                            for element in production.elements.iter().rev() {
                                stack.push((element.clone(), node_idx));
                                depth += 1;
                            }
                        }
                    } else {
                        // Handle unknown non-terminals
                        tokens.push(format!("<{}>", name));

                        // Create an "undefined" node
                        let undefined_node = QueryAstNode {
                            element_type: "undefined".to_string(),
                            value: name,
                            children: Vec::new(),
                        };

                        // Add it as a child to its parent
                        ast_nodes[parent_idx].children.push(undefined_node);
                    }
                }
            }
        }

        // Handle recursion limit if reached
        if depth >= self.config.max_recursion_depth {
            tokens.push(format!("<recursion_limit_exceeded>"));

            // Add a recursion limit node to the root
            let limit_node = QueryAstNode {
                element_type: "error".to_string(),
                value: "recursion_limit_exceeded".to_string(),
                children: Vec::new(),
            };
            ast_nodes[root_idx].children.push(limit_node);
        }

        // Construct the final string
        let mut result = String::new();
        let mut in_quotes = false;

        for (i, token) in tokens.iter().enumerate() {
            if i > 0 {
                let prev = &tokens[i - 1];
                if !in_quotes
                    && !prev.ends_with('(')
                    && !token.starts_with(')')
                    && !token.starts_with(',')
                    && !prev.ends_with(',')
                {
                    result.push(' ');
                }
            }
            if token == "\"" || token == "'" {
                in_quotes = !in_quotes;
            }
            result.push_str(token);
            if token == "," && !in_quotes {
                result.push(' ');
            }
        }

        // Return the generated string and the root AST node
        (result.trim().to_string(), ast_nodes[root_idx].clone())
    }

    /// Check if the grammar contains a specific non-terminal
    pub fn has_non_terminal(&self, name: &str) -> bool {
        self.rules.contains_key(name)
    }

    /// Get a reference to the grammar's rules
    pub fn rules(&self) -> &HashMap<String, Vec<Production>> {
        &self.rules
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

    /// Print the grammar as a DOT graph
    pub fn print_graph(&self) {
        println!("digraph Grammar {{");
        println!("  rankdir=LR;");
        println!("  node [shape=box];");

        // Print all non-terminals as nodes
        for non_terminal in self.rules.keys() {
            println!("  \"{}\" [label=\"{}\"];", non_terminal, non_terminal);
        }

        // Print all productions as edges
        for (non_terminal, productions) in &self.rules {
            for production in productions {
                let mut label = String::new();
                for element in &production.elements {
                    match element {
                        Element::Terminal(text) => label.push_str(&format!("{} ", text)),
                        Element::NonTerminal(name) => {
                            println!(
                                "  \"{}\" -> \"{}\" [label=\"{}\"];",
                                non_terminal,
                                name,
                                label.trim()
                            );
                            label.clear();
                        }
                    }
                }
                if !label.is_empty() {
                    println!(
                        "  \"{}\" -> \"{}\" [label=\"{}\"];",
                        non_terminal,
                        "END",
                        label.trim()
                    );
                }
            }
        }

        println!("}}");
    }
}
