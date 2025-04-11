use grammar_gen::utils::{GrammarValidator, ValidatorExt};
use grammar_gen::Grammar;
use std::error::Error;

/// Example of creating and using custom validators
fn main() -> Result<(), Box<dyn Error>> {
    // Create a simple grammar
    let mut grammar = Grammar::new();
    grammar.add_rule("expression", vec!["<term>", "+", "<expression>"])?;
    grammar.add_rule("expression", vec!["<term>"])?;
    grammar.add_rule("term", vec!["<factor>", "*", "<term>"])?;
    grammar.add_rule("term", vec!["<factor>"])?;
    grammar.add_rule("factor", vec!["(", "<expression>", ")"])?;
    grammar.add_rule("factor", vec!["<number>"])?;
    grammar.add_rule("number", vec!["1"])?;
    grammar.add_rule("number", vec!["2"])?;
    grammar.add_rule("number", vec!["3"])?;
    grammar.add_rule("expression", vec!["<number>"])?; // Base case to terminate recursion

    println!("=== Basic Grammar Output ===");
    for i in 1..=5 {
        let result = grammar.generate("expression");
        println!("{}. {}", i, result.text);
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
        let result = validated_grammar.generate("expression");
        println!("{}. {}", i, result.text);
    }

    // Create and use a LatexValidator
    let latex_validator = LatexValidator;
    let latex_grammar = grammar.clone().with_validator(Box::new(latex_validator));

    println!("\n=== LaTeX Formula Output ===");
    for i in 1..=5 {
        let result = latex_grammar.generate("expression");
        println!("{}. {}", i, result.text);
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
