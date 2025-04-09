use grammar_gen::utils::{GrammarValidator, ValidatorExt};
use grammar_gen::GrammarBuilder;
use std::error::Error;

/// Example of creating and using custom validators
fn main() -> Result<(), Box<dyn Error>> {
    // Create a simple grammar
    let grammar = GrammarBuilder::new()
        .add_rule("expression", &["<term>", "+", "<expression>"])
        .add_rule("expression", &["<term>"])
        .add_rule("term", &["<factor>", "*", "<term>"])
        .add_rule("term", &["<factor>"])
        .add_rule("factor", &["(", "<expression>", ")"])
        .add_rule("factor", &["<number>"])
        .add_rule("number", &["1"])
        .add_rule("number", &["2"])
        .add_rule("number", &["3"])
        .add_rule("expression", &["<number>"]) // Base case to terminate recursion
        .build();

    println!("=== Basic Grammar Output ===");
    for i in 1..=5 {
        println!("{}. {}", i, grammar.generate("expression"));
    }

    // Create a custom ParenthesesValidator
    let parens_validator = ParenthesesValidator;

    // Create a custom SpacingValidator
    let spacing_validator = SpacingValidator;

    // Chain the validators
    let chained_validator = parens_validator.chain(spacing_validator);

    // Apply the validators to the grammar
    let validated_grammar = grammar.clone().with_validator(Box::new(chained_validator));

    println!("\n=== With Custom Validators ===");
    for i in 1..=5 {
        println!("{}. {}", i, validated_grammar.generate("expression"));
    }

    // Create and use a LatexValidator
    let latex_validator = LatexValidator;
    let latex_grammar = grammar.clone().with_validator(Box::new(latex_validator));

    println!("\n=== LaTeX Formula Output ===");
    for i in 1..=5 {
        println!("{}. {}", i, latex_grammar.generate("expression"));
    }

    Ok(())
}

/// Custom validator that ensures proper spacing around operators
#[derive(Debug, Clone)]
struct SpacingValidator;

impl GrammarValidator for SpacingValidator {
    fn validate(&self, text: &str) -> String {
        // Replace any sequence of spaces with a single space
        let text = text.replace("  ", " ");

        // Ensure proper spacing around operators
        let text = text
            .replace(" +", " + ")
            .replace("+ ", " + ")
            .replace(" *", " * ")
            .replace("* ", " * ");

        // Remove any leading or trailing spaces
        text.trim().to_string()
    }

    fn name(&self) -> &str {
        "spacing"
    }

    fn clone_box(&self) -> Box<dyn GrammarValidator> {
        Box::new(self.clone())
    }
}

/// Custom validator that adds spaces around parentheses
#[derive(Debug, Clone)]
struct ParenthesesValidator;

impl GrammarValidator for ParenthesesValidator {
    fn validate(&self, text: &str) -> String {
        // Add spaces before and after parentheses for readability
        let text = text.replace("(", " ( ").replace(")", " ) ");

        text
    }

    fn name(&self) -> &str {
        "parentheses"
    }

    fn clone_box(&self) -> Box<dyn GrammarValidator> {
        Box::new(self.clone())
    }
}

/// Custom validator that converts expressions to LaTeX format
#[derive(Debug, Clone)]
struct LatexValidator;

impl GrammarValidator for LatexValidator {
    fn validate(&self, text: &str) -> String {
        // Convert basic arithmetic expressions to LaTeX format
        let mut latex = text.to_string();

        // Convert multiplication from * to \times
        latex = latex.replace("*", " \\times ");

        // Properly format parentheses
        latex = latex.replace("(", "\\left(").replace(")", "\\right)");

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
