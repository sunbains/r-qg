# Validators in Grammar-Gen

Validators provide a powerful way to transform and validate generated text. This document explains how to use and create custom validators.

## Built-in Validators

Grammar-Gen comes with several built-in validators:

### SQL Validators

- **SqlNullValidator**: Ensures correct NULL handling in SQL queries
  ```rust
  use grammar_gen::utils::SqlNullValidator;
  let grammar = grammar.with_validator(Box::new(SqlNullValidator));
  ```

- **SqlKeywordValidator**: Formats SQL keywords with consistent casing
  ```rust
  use grammar_gen::utils::{SqlKeywordValidator, SqlCaseFormat};
  let grammar = grammar.with_validator(Box::new(SqlKeywordValidator::new(SqlCaseFormat::Uppercase)));
  ```

### Factory Functions

For convenience, factory functions are provided:

```rust
use grammar_gen::utils::{sql_null_validator, sql_keyword_validator, sql_validator, SqlCaseFormat};

// Only NULL handling
let grammar = grammar.with_validator(sql_null_validator());

// Only keyword casing
let grammar = grammar.with_validator(sql_keyword_validator(SqlCaseFormat::Uppercase));

// Combined SQL validation (NULL handling + keyword casing)
let grammar = grammar.with_validator(sql_validator(SqlCaseFormat::Uppercase));
```

## Chaining Validators

The `ValidatorExt` trait allows validators to be chained together:

```rust
use grammar_gen::utils::{SqlNullValidator, SqlKeywordValidator, SqlCaseFormat, ValidatorExt};

let null_validator = SqlNullValidator;
let keyword_validator = SqlKeywordValidator::new(SqlCaseFormat::Uppercase);

// Chain validators
let combined = null_validator.chain(keyword_validator);

// Apply the chain
let grammar = grammar.with_validator(Box::new(combined));
```

## Validator Registry

The `ValidatorRegistry` provides a way to register and retrieve validators by name:

```rust
use grammar_gen::utils::{ValidatorRegistry, SqlNullValidator, SqlKeywordValidator, SqlCaseFormat, ValidatorExt};
use std::sync::Arc;

// Create a registry
let mut registry = ValidatorRegistry::new();

// Register built-in validators
registry.register_defaults();

// Register a custom validator
let custom_chain = SqlNullValidator.chain(SqlKeywordValidator::new(SqlCaseFormat::Capitalize));
registry.register("custom_sql", custom_chain);

// List available validators
let validators = registry.list_validators();
println!("Available validators: {:?}", validators);

// Retrieve a validator
if let Some(validator) = registry.get("custom_sql") {
    let grammar = grammar.with_validator(Arc::try_unwrap(validator).unwrap());
    let text = grammar.generate();
}
```

## Creating Custom Validators

To create a custom validator, implement the `GrammarValidator` trait:

```rust
use grammar_gen::utils::GrammarValidator;
use std::fmt;

#[derive(Debug, Clone)]
struct MyCustomValidator;

impl GrammarValidator for MyCustomValidator {
    fn validate(&self, text: &str) -> String {
        // Your custom validation logic here
        text.replace("old", "new")
    }
    
    fn name(&self) -> &str {
        "my_custom"
    }
    
    fn applies_to(&self, text: &str) -> bool {
        // Optional: determines if this validator should be applied to this text
        text.contains("specific_pattern")
    }
    
    fn clone_box(&self) -> Box<dyn GrammarValidator> {
        Box::new(self.clone())
    }
}
```

### Required Methods

- `validate(&self, text: &str) -> String`: The core validation logic
- `name(&self) -> &str`: A unique name for this validator
- `clone_box(&self) -> Box<dyn GrammarValidator>`: Creates a boxed clone

### Optional Methods

- `applies_to(&self, text: &str) -> bool`: Determines if the validator applies to the given text

## Example: LaTeX Formatter

Here's a more complete example of a custom validator that converts expressions to LaTeX format:

```rust
#[derive(Debug, Clone)]
pub struct LatexValidator;

impl GrammarValidator for LatexValidator {
    fn validate(&self, text: &str) -> String {
        // Convert basic arithmetic expressions to LaTeX format
        let mut latex = text.to_string();
        
        // Convert multiplication from * to \times
        latex = latex.replace("*", " \\times ");
        
        // Properly format parentheses
        latex = latex.replace("(", "\\left(")
                    .replace(")", "\\right)");
        
        // Wrap in LaTeX math delimiters
        format!("${latex}$")
    }
    
    fn name(&self) -> &str {
        "latex"
    }
    
    fn clone_box(&self) -> Box<dyn GrammarValidator> {
        Box::new(self.clone())
    }
}
```

## Best Practices

1. **Specialized Validators**: Create focused validators that do one thing well
2. **Chain Validators**: Use the chain method to combine validators
3. **Selective Application**: Use `applies_to` to determine when a validator should be applied
4. **Descriptive Names**: Use clear, descriptive names for your validators
5. **Handle Edge Cases**: Ensure your validator handles all possible input correctly

