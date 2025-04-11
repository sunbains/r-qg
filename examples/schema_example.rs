use grammar_gen::schema::{Column, Schema, SqlGenerator, SqlType, Table};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Create a simple blog schema with users and posts
    let users_table = Table::new("users")
        .add_column(Column::new("id", SqlType::Integer).primary_key())
        .add_column(
            Column::new("username", SqlType::Varchar(50))
                .unique()
                .not_null(),
        )
        .add_column(
            Column::new("email", SqlType::Varchar(100))
                .unique()
                .not_null(),
        )
        .add_column(Column::new("created_at", SqlType::Timestamp).not_null());

    let posts_table = Table::new("posts")
        .add_column(Column::new("id", SqlType::Integer).primary_key())
        .add_column(
            Column::new("user_id", SqlType::Integer)
                .not_null()
                .foreign_key("users", "id"),
        )
        .add_column(Column::new("title", SqlType::Varchar(200)).not_null())
        .add_column(Column::new("content", SqlType::Text).not_null())
        .add_column(Column::new("published", SqlType::Boolean).not_null())
        .add_column(Column::new("created_at", SqlType::Timestamp).not_null());

    let schema = Schema::new().add_table(users_table).add_table(posts_table);

    // Create SQL generator
    let generator = SqlGenerator::new(schema);

    // Generate schema and sample data
    println!("=== Generated Schema and Data ===");
    println!("{}", generator.generate_schema_and_data(5)); // Generate 5 rows per table

    Ok(())
}
