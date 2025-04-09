# Grammar-Gen

[![Crates.io](https://img.shields.io/crates/v/r-qg.svg)](https://crates.io/crates/grammar-gen)
[![Docs.rs](https://docs.rs/r-qg/badge.svg)](https://docs.rs/r-qg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

r-qg is a flexible grammar-based text generator for Rust that allows you to generate random, syntactically valid text based on grammar rules. It's especially useful for generating test data, example outputs, and exploring language structures.

## Features

- Simple and intuitive grammar definition format
- Support for nested non-terminals and complex grammar rules
- Random selection of production rules for varied output
- Special handling for SQL NULL values and operators
- Command-line interface for easy text generation
- Library interface for integration with other Rust applications

## Installation

### Using as a Binary

```bash
cargo install r-qg
```

### Adding as a Dependency

Add the following to your `Cargo.toml`:

```toml
[dependencies]
r-qg = "0.1.0"
```

## Grammar Format

Grammars are defined in text files with the following syntax:

```
<non-terminal> ::= [element1, element2, ...]
```

Where elements can be:
- Terminals (quoted strings): `"SELECT"`, `"FROM"`, etc.
- Non-terminals (in angle brackets): `<table_name>`, `<column>`, etc.

Multiple productions for a non-terminal are specified by separate rules:

```
<value> ::= ["1"]
<value> ::= ["TRUE"]
<value> ::= ["'text'"]
```

## Usage

### Command Line

```bash
# Generate 5 random SQL queries using the example grammar
r-qg examples/sql_grammar.txt query 5

# Use your own grammar file with a custom start symbol
r-qg path/to/grammar.txt start_symbol [count]
```

### As a Library

```rust
use r-qg::{Grammar, GrammarConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load grammar from a file
    let grammar = Grammar::from_file("examples/sql_grammar.txt", "query")?;
    
    // Generate random text
    let random_text = grammar.generate();
    println!("Generated: {}", random_text);
    
    // Or create a grammar programmatically
    let mut grammar = Grammar::new("greeting");
    grammar.add_rule("greeting", vec!["Hello", "<subject>"])?;
    grammar.add_rule("subject", vec!["world"])?;
    grammar.add_rule("subject", vec!["Rust", "programmer"])?;
    
    let greeting = grammar.generate();
    println!("Greeting: {}", greeting);
    
    Ok(())
}
```

## Example: SQL Generator

One of the included examples is an SQL query generator:

```rust
use grammar_gen::Grammar;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let grammar = Grammar::from_file("examples/sql_grammar.txt", "query")?;
    
    println!("Generated SQL Queries:");
    for i in 1..=5 {
        println!("{}. {}", i, grammar.generate());
    }
    
    Ok(())
}
```

Example output:
```
Generated SQL Queries:
1. SELECT id FROM users WHERE email IS NULL
2. SELECT name, created_at FROM products
3. SELECT * FROM orders WHERE id > 10 AND status = 'pending'
4. SELECT * FROM customers
5. SELECT price FROM inventory WHERE quantity <= 0
```

## Advanced Features

### NULL Value Handling

The library includes special handling for SQL NULL values, ensuring they're used with the correct operators:

```sql
-- Correct (automatically fixed)
WHERE status IS NULL

-- Instead of incorrect
WHERE status = NULL
```

### Custom Validation

You can add custom validation rules to ensure generated text follows specific patterns:

```rust
let grammar = Grammar::new("start")
    .with_validator(Box::new(|text| {
        // Your custom validation logic here
        text.replace("incorrect", "correct")
    }));
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

