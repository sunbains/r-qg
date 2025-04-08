use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use rand::Rng;
use regex::Regex;

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
        
        // Regular expressions for parsing
        let rule_regex = Regex::new(r"^\s*<([^>]+)>\s*::=\s*\[(.*)\]\s*$").unwrap();
        
        // Current rule name and productions
        let mut current_rule = String::new();
        let mut current_productions = Vec::new();
        let mut in_multiline_rule = false;
        let mut current_line_buffer = String::new();
        
        for line in reader.lines() {
            let line = line.map_err(GrammarError::Io)?;
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Handle multiline rules
            if in_multiline_rule {
                current_line_buffer.push_str(trimmed);
                
                if trimmed.ends_with(']') {
                    in_multiline_rule = false;
                    
                    // Process the complete rule
                    if let Some(elements_str) = current_line_buffer.split('[').nth(1) {
                        let elements_str = elements_str.trim_end_matches(']');
                        let production = Self::parse_production(elements_str)?;
                        current_productions.push(production);
                        
                        // Store the rule
                        grammar.rules.insert(current_rule.clone(), current_productions);
                        
                        // Reset for the next rule
                        current_rule = String::new();
                        current_productions = Vec::new();
                        current_line_buffer = String::new();
                    }
                    continue;
                }
                continue;
            }
            
            // Start of a new rule
            if let Some(captures) = rule_regex.captures(trimmed) {
                let rule_name = captures.get(1).unwrap().as_str().to_string();
                let elements_str = captures.get(2).unwrap().as_str();
                
                // Check if this rule continues on another line
                if !trimmed.ends_with(']') {
                    in_multiline_rule = true;
                    current_rule = rule_name;
                    current_line_buffer = format!("<{}> ::= [{}",
                       current_rule, elements_str);
                    continue;
                }
                
                // Process the single-line rule
                let production = Self::parse_production(elements_str)?;
                
                if grammar.rules.contains_key(&rule_name) {
                    if let Some(productions) = grammar.rules.get_mut(&rule_name) {
                        productions.push(production);
                    }
                } else {
                    grammar.rules.insert(rule_name, vec![production]);
                }
            } else if trimmed.contains("::=") && trimmed.contains('[') {
                // Alternative rule for an existing non-terminal
                let parts: Vec<&str> = trimmed.split("::=").collect();
                if parts.len() == 2 {
                    let rule_name_part = parts[0].trim();
                    if rule_name_part.starts_with('|') {
                        // This is an alternative production for the previous rule
                        if !current_rule.is_empty() {
                            let elements_str = parts[1].trim().trim_start_matches('[').trim_end_matches(']');
                            let production = Self::parse_production(elements_str)?;
                            
                            if let Some(productions) = grammar.rules.get_mut(&current_rule) {
                                productions.push(production);
                            }
                        }
                    }
                }
            }
        }
        
        // Validate that the start symbol exists
        if !grammar.rules.contains_key(&grammar.start_symbol) {
            return Err(GrammarError::UnknownNonTerminal(grammar.start_symbol.clone()));
        }
        
        Ok(grammar)
    }
    
    /// Parse a production rule from a string
    pub fn parse_production(elements_str: &str) -> Result<Production> {
        // let non_terminal_regex = Regex::new(r"<([^>]+)>").unwrap();
        // let quoted_terminal_regex = Regex::new(r#""([^"]*)"|\s*'([^']*)'"#).unwrap();
        
        let mut elements = Vec::new();
        let mut current_pos = 0;
        let chars: Vec<char> = elements_str.chars().collect();
        
        while current_pos < chars.len() {
            // Skip whitespace
            while current_pos < chars.len() && chars[current_pos].is_whitespace() {
                current_pos += 1;
            }
            
            if current_pos >= chars.len() {
                break;
            }
            
            // Check for non-terminal
            if chars[current_pos] == '<' {
                let start_pos = current_pos;
                while current_pos < chars.len() && chars[current_pos] != '>' {
                    current_pos += 1;
                }
                
                if current_pos < chars.len() {
                    current_pos += 1; // Include the '>'
                    let token = elements_str[start_pos..current_pos].to_string();
                    let name = token[1..token.len()-1].to_string(); // Remove < >
                    elements.push(Element::NonTerminal(name));
                }
            }
            // Check for quoted terminal (both single and double quotes)
            else if chars[current_pos] == '"' || chars[current_pos] == '\'' {
                let quote_char = chars[current_pos];
                current_pos += 1; // Skip the opening quote
                let start_pos = current_pos;
                
                while current_pos < chars.len() && chars[current_pos] != quote_char {
                    current_pos += 1;
                }
                
                if current_pos < chars.len() {
                    let terminal = elements_str[start_pos..current_pos].to_string();
                    elements.push(Element::Terminal(terminal));
                    current_pos += 1; // Skip the closing quote
                }
            }
            // Handle comma separators
            else if chars[current_pos] == ',' {
                current_pos += 1;
            }
            // Unquoted terminals (for SQL keywords)
            else {
                let start_pos = current_pos;
                while current_pos < chars.len() && 
                      chars[current_pos] != ',' && 
                      chars[current_pos] != '<' && 
                      !chars[current_pos].is_whitespace() {
                    current_pos += 1;
                }
                
                if current_pos > start_pos {
                    let terminal = elements_str[start_pos..current_pos].to_string();
                    if !terminal.is_empty() {
                        elements.push(Element::Terminal(terminal));
                    }
                }
            }
        }
        
        if elements.is_empty() {
            return Err(GrammarError::EmptyProduction(elements_str.to_string()));
        }
        
        Ok(Production { elements })
    }

    /// Add a rule to the grammar
    pub fn add_rule(&mut self, non_terminal: &str, elements: Vec<&str>) -> Result<&mut Self> {
        let production = self.parse_elements(elements)?;
        
        if self.rules.contains_key(non_terminal) {
            if let Some(productions) = self.rules.get_mut(non_terminal) {
                productions.push(production);
            }
        } else {
            self.rules.insert(non_terminal.to_string(), vec![production]);
        }
        
        Ok(self)
    }
    
    /// Parse a vector of strings into a Production
    fn parse_elements(&self, elements: Vec<&str>) -> Result<Production> {
        let mut parsed_elements = Vec::new();
        let non_terminal_regex = Regex::new(r"^<([^>]+)>$").unwrap();
        
        for element in elements {
            if let Some(captures) = non_terminal_regex.captures(element) {
                let name = captures.get(1).unwrap().as_str();
                parsed_elements.push(Element::NonTerminal(name.to_string()));
            } else {
                parsed_elements.push(Element::Terminal(element.to_string()));
            }
        }
        
        if parsed_elements.is_empty() {
            return Err(GrammarError::EmptyProduction("Empty elements vector".to_string()));
        }
        
        Ok(Production { elements: parsed_elements })
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
                        if self.config.auto_spacing && !result.is_empty() && !result.ends_with(' ') && !text.starts_with(' ') {
                            result.push(' ');
                        }
                        result.push_str(text);
                    },
                    Element::NonTerminal(name) => {
                        let expanded = self.expand_non_terminal(name, depth + 1);
                        // Add space if needed
                        if self.config.auto_spacing && !result.is_empty() && !result.ends_with(' ') && !expanded.starts_with(' ') {
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
        // Ignore errors in builder pattern for simplicity
        let _ = self.grammar.add_rule(non_terminal, elements.to_vec());
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
        grammar.add_rule("start", vec!["Hello", "<subject>"]).unwrap();
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
        grammar.add_rule("recursive", vec!["<recursive>", "loop"]).unwrap();
        
        // Should hit recursion limit
        let result = grammar.generate();
        assert!(result.contains("recursion_limit_exceeded"));
    }
}

