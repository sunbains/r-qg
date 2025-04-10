# Grammar-Gen

A complete Rust crate is now ready for a GitHub repository. Here's what's included:

## Project Structure

```
grammar-gen/
├── Cargo.toml           # Crate configuration and dependencies
├── .gitignore           # Files to be ignored by Git
├── README.md            # Documentation and usage examples
├── LICENSE              # MIT License
├── src/
│   ├── main.rs          # CLI application
│   ├── lib.rs           # Library entry point
│   ├── grammar.rs       # Core grammar implementation
│   └── utils.rs         # Utility functions and error types
├── examples/
│   ├── sql_generator.rs # SQL generation example
│   └── custom_grammar.rs # Custom grammar example
└── tests/
    └── integration_tests.rs # Integration tests
```

## Key Features

1. **Flexible Grammar Definition**:
   - Define grammars with non-terminals mapping to arrays of elements
   - Support for nested non-terminals and complex structures
   - Special handling for SQL NULL values

2. **Multiple Interfaces**:
   - Command-line tool for quick generation
   - Library interface for integration in other projects
   - Builder pattern for programmatic grammar creation

3. **Extensive Testing**:
   - Unit tests for core functionality
   - Integration tests for end-to-end verification
   - Example applications for real-world use cases

4. **Well-Documented**:
   - Comprehensive API documentation
   - Example grammars for SQL and JSON
   - Step-by-step usage examples

## Setup Instructions

1. Clone this repository
2. Build the crate: `cargo build --release`
3. Run examples: `cargo run --example sql_generator`
4. Use as a binary: `cargo install --path .`
5. Add as a dependency in your Rust project

## Getting Started

The most basic usage:

```bash
# Create an example grammar
grammar-gen example sql my_grammar.txt

# Generate 5 random samples
grammar-gen my_grammar.txt query 5
```

Or in your Rust code:

```rust
use grammar_gen::{Grammar, GrammarBuilder};

// Create a grammar
let grammar = GrammarBuilder::new("greeting")
    .add_rule("greeting", &["Hello", "<subject>"])
    .add_rule("subject", &["world"])
    .add_rule("subject", &["Rust"])
    .build();

// Generate text
let text = grammar.generate();
println!("Generated: {}", text);
```

