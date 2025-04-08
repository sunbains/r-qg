use std::fmt;
use std::io;
use std::sync::Arc;
use thiserror::Error;

/// Custom error types for the grammar generator
#[derive(Error, Debug)]
pub enum GrammarError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid grammar: {0}")]
    InvalidGrammar(String),

    #[error("Unknown non-terminal: {0}")]
    UnknownNonTerminal(String),

    #[error("Empty production: {0}")]
    EmptyProduction(String),

    #[error("Grammar validation failed: {0}")]
    ValidationFailed(String),

    #[error("Validator error: {0}")]
    ValidatorError(String),
}

/// Result type for grammar operations
pub type Result<T> = std::result::Result<T, GrammarError>;

/// Trait for grammar validation functions
pub trait GrammarValidator: Send + Sync + fmt::Debug {
    /// Validate and potentially transform generated text
    fn validate(&self, text: &str) -> String;

    /// Get the name of this validator
    fn name(&self) -> &str;

    /// Check if this validator applies to a given text
    fn applies_to(&self, _text: &str) -> bool {
        true // By default, applies to all text
    }

    /// Clone this validator as a box
    fn clone_box(&self) -> Box<dyn GrammarValidator>;
}

impl Clone for Box<dyn GrammarValidator> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Trait for creating validator chains
pub trait ValidatorExt: GrammarValidator + Sized {
    /// Chain this validator with another one
    fn chain<V: GrammarValidator + 'static>(self, other: V) -> ValidatorChain
    where
        Self: 'static,
    {
        ValidatorChain::new(Box::new(self), Box::new(other))
    }
}

// Implement ValidatorExt for all GrammarValidators
impl<T: GrammarValidator + 'static> ValidatorExt for T {}

/// A chain of validators that are applied in sequence
#[derive(Debug)]
pub struct ValidatorChain {
    validators: Vec<Box<dyn GrammarValidator>>,
    name: String,
}

impl ValidatorChain {
    /// Create a new validator chain from two validators
    pub fn new(first: Box<dyn GrammarValidator>, second: Box<dyn GrammarValidator>) -> Self {
        let name = format!("{}+{}", first.name(), second.name());
        let validators = vec![first, second];
        ValidatorChain { validators, name }
    }

    /// Add another validator to the chain
    pub fn add<V: GrammarValidator + 'static>(mut self, validator: V) -> Self {
        self.name = format!("{}+{}", self.name, validator.name());
        self.validators.push(Box::new(validator));
        self
    }
}

impl GrammarValidator for ValidatorChain {
    fn validate(&self, text: &str) -> String {
        let mut result = text.to_string();
        for validator in &self.validators {
            if validator.applies_to(&result) {
                result = validator.validate(&result);
            }
        }
        result
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_box(&self) -> Box<dyn GrammarValidator> {
        let mut cloned = Vec::new();
        for validator in &self.validators {
            cloned.push(validator.clone_box());
        }
        Box::new(ValidatorChain {
            validators: cloned,
            name: self.name.clone(),
        })
    }
}

/// A no-op validator that performs no changes
#[derive(Debug, Clone)]
pub struct NoopValidator;

impl GrammarValidator for NoopValidator {
    fn validate(&self, text: &str) -> String {
        text.to_string()
    }

    fn name(&self) -> &str {
        "noop"
    }

    fn clone_box(&self) -> Box<dyn GrammarValidator> {
        Box::new(self.clone())
    }
}

/// SQL-specific validator that ensures NULL values are properly handled
#[derive(Debug, Clone)]
pub struct SqlNullValidator;

impl GrammarValidator for SqlNullValidator {
    fn validate(&self, sql: &str) -> String {
        // Replace incorrect NULL comparisons with correct IS NULL or IS NOT NULL
        let sql = sql
            .replace(" = NULL", " IS NULL")
            .replace(" = null", " IS NULL")
            .replace(" != NULL", " IS NOT NULL")
            .replace(" != null", " IS NOT NULL")
            .replace(" <> NULL", " IS NOT NULL")
            .replace(" <> null", " IS NOT NULL")
            .replace(" > NULL", " IS NOT NULL")
            .replace(" > null", " IS NOT NULL")
            .replace(" < NULL", " IS NOT NULL")
            .replace(" < null", " IS NOT NULL")
            .replace(" >= NULL", " IS NOT NULL")
            .replace(" >= null", " IS NOT NULL")
            .replace(" <= NULL", " IS NOT NULL")
            .replace(" <= null", " IS NOT NULL");

        sql
    }

    fn name(&self) -> &str {
        "sql_null"
    }

    fn applies_to(&self, text: &str) -> bool {
        // Only apply to SQL-like statements
        let upper = text.to_uppercase();
        upper.contains("SELECT")
            || upper.contains("WHERE")
            || upper.contains("FROM")
            || upper.contains("NULL")
    }

    fn clone_box(&self) -> Box<dyn GrammarValidator> {
        Box::new(self.clone())
    }
}

/// SQL keyword case formatting validator
#[derive(Debug, Clone)]
pub enum SqlCaseFormat {
    Uppercase,
    Lowercase,
    Capitalize,
}

/// SQL keyword validator that ensures consistent casing
#[derive(Debug, Clone)]
pub struct SqlKeywordValidator {
    format: SqlCaseFormat,
    keywords: Vec<String>,
}

impl SqlKeywordValidator {
    /// Create a new SQL keyword validator with the specified case format
    pub fn new(format: SqlCaseFormat) -> Self {
        // Common SQL keywords
        let keywords = vec![
            "SELECT",
            "FROM",
            "WHERE",
            "GROUP BY",
            "ORDER BY",
            "HAVING",
            "JOIN",
            "LEFT JOIN",
            "RIGHT JOIN",
            "INNER JOIN",
            "OUTER JOIN",
            "ON",
            "AND",
            "OR",
            "NOT",
            "IN",
            "BETWEEN",
            "LIKE",
            "IS NULL",
            "IS NOT NULL",
            "AS",
            "DISTINCT",
            "UNION",
            "ALL",
            "INSERT",
            "UPDATE",
            "DELETE",
            "CREATE",
            "ALTER",
            "DROP",
            "TABLE",
            "VIEW",
            "INDEX",
            "CONSTRAINT",
            "PRIMARY KEY",
            "FOREIGN KEY",
            "REFERENCES",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        SqlKeywordValidator { format, keywords }
    }
}

impl GrammarValidator for SqlKeywordValidator {
    fn validate(&self, sql: &str) -> String {
        let mut result = sql.to_string();

        for keyword in &self.keywords {
            let replacement = match self.format {
                SqlCaseFormat::Uppercase => keyword.to_uppercase(),
                SqlCaseFormat::Lowercase => keyword.to_lowercase(),
                SqlCaseFormat::Capitalize => {
                    let mut c = keyword.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => {
                            f.to_uppercase().collect::<String>()
                                + c.as_str().to_lowercase().as_str()
                        }
                    }
                }
            };

            // Case-insensitive replacement
            let keyword_upper = keyword.to_uppercase();
            let pattern = format!(r"(?i)\b{}\b", regex::escape(&keyword_upper));

            if let Ok(re) = regex::Regex::new(&pattern) {
                result = re.replace_all(&result, replacement.as_str()).to_string();
            }
        }

        result
    }

    fn name(&self) -> &str {
        match self.format {
            SqlCaseFormat::Uppercase => "sql_uppercase",
            SqlCaseFormat::Lowercase => "sql_lowercase",
            SqlCaseFormat::Capitalize => "sql_capitalize",
        }
    }

    fn applies_to(&self, text: &str) -> bool {
        // Only apply to SQL-like statements
        text.to_uppercase().contains("SELECT") || text.to_uppercase().contains("FROM")
    }

    fn clone_box(&self) -> Box<dyn GrammarValidator> {
        Box::new(self.clone())
    }
}

/// Factory functions for creating common validators

/// Create an SQL NULL handling validator
pub fn sql_null_validator() -> Box<dyn GrammarValidator> {
    Box::new(SqlNullValidator)
}

/// Create an SQL keyword validator with consistent case formatting
pub fn sql_keyword_validator(format: SqlCaseFormat) -> Box<dyn GrammarValidator> {
    Box::new(SqlKeywordValidator::new(format))
}

/// Create a comprehensive SQL validator that handles NULL values and keyword casing
pub fn sql_validator(format: SqlCaseFormat) -> Box<dyn GrammarValidator> {
    let null_validator = SqlNullValidator;
    let keyword_validator = SqlKeywordValidator::new(format);
    Box::new(null_validator.chain(keyword_validator))
}

/// Create a no-op validator that doesn't change the input
pub fn noop_validator() -> Box<dyn GrammarValidator> {
    Box::new(NoopValidator)
}

/// Registry for managing and retrieving validators
#[derive(Debug, Clone, Default)]
pub struct ValidatorRegistry {
    validators: Vec<(String, Arc<Box<dyn GrammarValidator>>)>,
}

impl ValidatorRegistry {
    /// Create a new empty validator registry
    pub fn new() -> Self {
        ValidatorRegistry {
            validators: Vec::new(),
        }
    }

    /// Register a validator with a name
    pub fn register<V: GrammarValidator + 'static>(
        &mut self,
        name: &str,
        validator: V,
    ) -> &mut Self {
        self.validators
            .push((name.to_string(), Arc::new(Box::new(validator))));
        self
    }

    /// Get a validator by name
    pub fn get(&self, name: &str) -> Option<Arc<Box<dyn GrammarValidator>>> {
        self.validators
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| Arc::clone(v))
    }

    /// Get a list of all registered validator names
    pub fn list_validators(&self) -> Vec<String> {
        self.validators
            .iter()
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Register common built-in validators
    pub fn register_defaults(&mut self) -> &mut Self {
        self.register("noop", NoopValidator)
            .register("sql_null", SqlNullValidator)
            .register(
                "sql_uppercase",
                SqlKeywordValidator::new(SqlCaseFormat::Uppercase),
            )
            .register(
                "sql_lowercase",
                SqlKeywordValidator::new(SqlCaseFormat::Lowercase),
            )
            .register(
                "sql_capitalize",
                SqlKeywordValidator::new(SqlCaseFormat::Capitalize),
            )
    }
}

/// Create a default validator registry with common validators
pub fn default_validator_registry() -> ValidatorRegistry {
    let mut registry = ValidatorRegistry::new();
    registry.register_defaults();
    registry
}

/// Trait extension for Option<T> to convert to GrammarError
pub trait OptionExt<T> {
    fn ok_or_grammar_err<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_grammar_err<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.ok_or_else(|| GrammarError::InvalidGrammar(f()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_null_validator() {
        let validator = SqlNullValidator;

        assert_eq!(
            validator.validate("SELECT * FROM users WHERE status = NULL"),
            "SELECT * FROM users WHERE status IS NULL"
        );

        assert_eq!(
            validator.validate("SELECT * FROM users WHERE status != NULL"),
            "SELECT * FROM users WHERE status IS NOT NULL"
        );
    }

    #[test]
    fn test_sql_keyword_validator() {
        let uppercase_validator = SqlKeywordValidator::new(SqlCaseFormat::Uppercase);
        let lowercase_validator = SqlKeywordValidator::new(SqlCaseFormat::Lowercase);
        let capitalize_validator = SqlKeywordValidator::new(SqlCaseFormat::Capitalize);

        let sample = "select * from users where id = 1";

        assert_eq!(
            uppercase_validator.validate(sample),
            "SELECT * FROM users WHERE id = 1"
        );

        assert_eq!(
            lowercase_validator.validate("SELECT * FROM users WHERE id = 1"),
            "select * from users where id = 1"
        );

        assert_eq!(
            capitalize_validator.validate("SELECT * FROM users WHERE id = 1"),
            "Select * From users Where id = 1"
        );
    }

    #[test]
    fn test_validator_chain() {
        let null_validator = SqlNullValidator;
        let case_validator = SqlKeywordValidator::new(SqlCaseFormat::Uppercase);

        let chain = null_validator.chain(case_validator);

        let result = chain.validate("select * from users where status = null");
        assert_eq!(result, "SELECT * FROM users WHERE status IS NULL");
    }

    #[test]
    fn test_validator_registry() {
        let mut registry = ValidatorRegistry::new();
        registry.register(
            "test_uppercase",
            SqlKeywordValidator::new(SqlCaseFormat::Uppercase),
        );

        let validator = registry.get("test_uppercase").unwrap();
        let result = validator.validate("select * from users");
        assert_eq!(result, "SELECT * FROM users");

        let validators = registry.list_validators();
        assert_eq!(validators, vec!["test_uppercase"]);
    }
}
