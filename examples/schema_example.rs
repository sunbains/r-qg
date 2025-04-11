use grammar_gen::schema::{
    Column, MySqlDialect, Schema, SqlGenerator, SqlGrammarExtension, SqlType, Table,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let dialect = MySqlDialect;
    // 1. Basic Schema Creation
    println!("=== Basic Schema Creation ===");
    let mut schema = Schema::new();

    // Create a users table
    let users = Table::new("users")
        .add_column(
            Column::new("id", SqlType::Integer)
                .primary_key()
                .auto_increment(),
        )
        .add_column(
            Column::new("username", SqlType::Varchar(50))
                .not_null()
                .unique(),
        )
        .add_column(
            Column::new("email", SqlType::Varchar(100))
                .not_null()
                .unique(),
        )
        .add_column(Column::new("created_at", SqlType::Timestamp).not_null())
        .add_column(Column::new("is_active", SqlType::Boolean).not_null());

    // Create a posts table with foreign key
    let posts = Table::new("posts")
        .add_column(
            Column::new("id", SqlType::Integer)
                .primary_key()
                .auto_increment(),
        )
        .add_column(Column::new("user_id", SqlType::Integer).not_null())
        .add_column(Column::new("title", SqlType::Varchar(200)).not_null())
        .add_column(Column::new("content", SqlType::Text))
        .add_column(Column::new("created_at", SqlType::Timestamp).not_null())
        .add_column(Column::new("updated_at", SqlType::Timestamp))
        .add_column(Column::new("status", SqlType::Varchar(20)).not_null())
        .add_column(Column::new("views", SqlType::Integer).not_null())
        .add_column(Column::new("likes", SqlType::Integer).not_null());

    // Add foreign key constraint
    let posts = posts.add_column(
        Column::new("user_id", SqlType::Integer)
            .not_null()
            .foreign_key("users", "id"),
    );

    // Add tables to schema
    schema = schema.add_table(users).add_table(posts);

    // 2. Generate DDL
    println!("\n=== Generated DDL ===");
    let ddl = schema.create_schema_sql(&dialect);
    println!("{}", ddl);

    // 3. Generate Sample Data
    println!("\n=== Generated Sample Data ===");
    let sample_data = schema.generate_data_sql(3, &dialect);
    println!("{}", sample_data);

    // 4. Generate Select Queries
    println!("\n=== Generated Select Queries ===");
    let select_queries = schema.tables[1].generate_select_query(2);
    println!("{}", select_queries);

    // 5. Using SqlGenerator
    println!("\n=== Using SqlGenerator ===");
    let generator = SqlGenerator::new(schema.clone(), dialect.clone());
    let complete_sql = generator.generate_schema_and_data(2);
    println!("{}", complete_sql);

    // 6. Using SqlGrammarExtension
    println!("\n=== Using SqlGrammarExtension ===");
    let extension = SqlGrammarExtension::new(schema.clone());
    let ddl = extension.generate_ddl();
    println!("DDL:\n{}", ddl);

    let dml = extension.generate_dml(2);
    println!("\nDML:\n{}", dml);

    let queries = extension.generate_queries(3);
    println!("\nQueries:");
    for (i, query) in queries.iter().enumerate() {
        println!("{}. {}", i + 1, query);
    }

    // 7. Load Schema from JSON
    println!("\n=== Loading Schema from JSON ===");
    let json_schema = r#"
    {
        "tables": [
            {
                "name": "products",
                "columns": [
                    {
                        "name": "id",
                        "type": "integer",
                        "primary_key": true,
                        "auto_increment": true
                    },
                    {
                        "name": "name",
                        "type": {
                            "varchar": 100
                        },
                        "nullable": false
                    },
                    {
                        "name": "price",
                        "type": "float",
                        "nullable": false
                    }
                ]
            }
        ]
    }
    "#;

    let schema_from_json = Schema::from_json_str(json_schema)?;
    println!("Schema loaded from JSON:");
    println!("{}", schema_from_json.create_schema_sql(&dialect));

    // 8. Test Different SQL Types
    println!("\n=== Testing Different SQL Types ===");
    let mut test_table = Table::new("test_types");
    test_table = test_table
        .add_column(Column::new("int_col", SqlType::Integer))
        .add_column(Column::new("float_col", SqlType::Float))
        .add_column(Column::new("varchar_col", SqlType::Varchar(50)))
        .add_column(Column::new("text_col", SqlType::Text))
        .add_column(Column::new("bool_col", SqlType::Boolean))
        .add_column(Column::new("date_col", SqlType::Date))
        .add_column(Column::new("timestamp_col", SqlType::Timestamp));

    let test_schema = Schema::new().add_table(test_table);
    println!("{}", test_schema.create_schema_sql(&dialect));

    // 9. Generate Insert Statements
    println!("\n=== Generated Insert Statements ===");
    let insert_statements = test_schema.tables[0].generate_insert_statements(2, &dialect);
    for (i, stmt) in insert_statements.iter().enumerate() {
        println!("{}. {}", i + 1, stmt);
    }

    Ok(())
}
