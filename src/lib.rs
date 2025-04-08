//! Grammar-Gen is a flexible grammar-based text generator.
//!
//! This library allows you to define grammars where non-terminals map to arrays of
//! terminals and other non-terminals, and then use those grammars to generate
//! random text that follows the syntactic rules.
//!
//! # Example
//!
//! ```rust
//! use grammar_gen::{Grammar, GrammarBuilder};
//!
//! // Create a simple grammar programmatically
//! let mut grammar = GrammarBuilder::new("greeting")
//!     .add_rule("greeting", &["Hello", "<subject>"])
//!     .add_rule("subject", &["world"])
//!     .add_rule("subject", &["Rust", "programmers"])
//!     .build();
//!
//! // Generate a random greeting
//! let text = grammar.generate();
//! assert!(text == "Hello world" || text == "Hello Rust programmers");
//! ```

pub mod common;
pub mod grammar;
pub mod utils;

pub use grammar::{Grammar, GrammarBuilder, GrammarConfig};
pub use utils::{GrammarError, Result, SqlNullValidator};

// Re-export common enums and structs
pub use grammar::{Element, Production};
