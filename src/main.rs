use clap::{Parser, Subcommand};
use grammar_gen::Grammar;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

/// Grammar-based text generator
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the grammar file
    #[arg(help = "Path to the grammar file")]
    grammar_file: Option<PathBuf>,

    /// The starting non-terminal symbol
    #[arg(help = "Starting non-terminal symbol")]
    start_symbol: Option<String>,

    /// Number of texts to generate
    #[arg(help = "Number of texts to generate", default_value = "1")]
    count: Option<usize>,

    /// Subcommands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate example grammar files
    Example {
        /// Type of grammar to generate
        #[arg(help = "Type of grammar (sql, json, etc.)", default_value = "sql")]
        grammar_type: String,

        /// Output file path
        #[arg(help = "Output file path")]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        match command {
            Commands::Example {
                grammar_type,
                output,
            } => {
                let output_path = output.unwrap_or_else(|| {
                    let filename = format!("example_{}_grammar.txt", grammar_type);
                    PathBuf::from(&filename)
                });

                match grammar_type.as_str() {
                    "sql" => read_sql_grammar(&output_path)?,
                    _ => {
                        return Err(format!("Unknown grammar type: {}", grammar_type).into());
                    }
                }

                println!(
                    "Created example {} grammar at: {}",
                    grammar_type,
                    output_path.display()
                );
                return Ok(());
            }
        }
    }

    // Process normal grammar generation
    let grammar_file = cli.grammar_file.ok_or("Grammar file path required")?;
    let start_symbol = cli.start_symbol.ok_or("Start symbol required")?;
    let count = cli.count.unwrap_or(1);

    println!("Loading grammar from {}...", grammar_file.display());
    let grammar = Grammar::from_file(&grammar_file, &start_symbol)?;

    println!("Loaded {} rules.", grammar.rules().len());
    println!("Generating {} random samples:\n", count);

    for i in 0..count {
        let generated = grammar.generate();
        println!("{}. {}", i + 1, generated);
    }

    Ok(())
}

fn read_sql_grammar(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut grammar = String::new();
    reader.read_to_string(&mut grammar)?;

    Ok(())
}
