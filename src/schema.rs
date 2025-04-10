// src/sql/schema.rs
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SqlType {
    Integer,
    Float,
    Varchar(usize),
    Text,
    Boolean,
    Date,
    Timestamp,
    // Add other SQL types as needed
}

impl SqlType {
    pub fn to_sql_string(&self) -> String {
        match self {
            SqlType::Integer => "INTEGER".to_string(),
            SqlType::Float => "FLOAT".to_string(),
            SqlType::Varchar(size) => format!("VARCHAR({})", size),
            SqlType::Text => "TEXT".to_string(),
            SqlType::Boolean => "BOOLEAN".to_string(),
            SqlType::Date => "DATE".to_string(),
            SqlType::Timestamp => "TIMESTAMP".to_string(),
        }
    }

    pub fn generate_random_value(&self) -> String {
        let mut rng = rand::thread_rng();
        match self {
            SqlType::Integer => format!("{}", rng.gen_range(1..1000)),
            SqlType::Float => format!("{:.2}", rng.gen::<f64>() * 100.0),
            SqlType::Varchar(size) => {
                let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
                let length = rng.gen_range(1..*size.min(&20));
                (0..length)
                    .map(|_| chars[rng.gen_range(0..chars.len())])
                    .collect::<String>()
            }
            SqlType::Text => {
                let words = [
                    "lorem",
                    "ipsum",
                    "dolor",
                    "sit",
                    "amet",
                    "consectetur",
                    "adipiscing",
                ];
                let word_count = rng.gen_range(3..10);
                (0..word_count)
                    .map(|_| words[rng.gen_range(0..words.len())])
                    .collect::<Vec<&str>>()
                    .join(" ")
            }
            SqlType::Boolean => if rng.gen_bool(0.5) { "TRUE" } else { "FALSE" }.to_string(),
            SqlType::Date => {
                let year = rng.gen_range(2000..2023);
                let month = rng.gen_range(1..13);
                let day = rng.gen_range(1..29); // Simplifying to avoid month length issues
                format!("'{}-{:02}-{:02}'", year, month, day)
            }
            SqlType::Timestamp => {
                let date = SqlType::Date.generate_random_value();
                let hour = rng.gen_range(0..24);
                let minute = rng.gen_range(0..60);
                let second = rng.gen_range(0..60);
                format!(
                    "{} {:02}:{:02}:{:02}",
                    date.trim_matches('\''),
                    hour,
                    minute,
                    second
                )
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub sql_type: SqlType,
    pub primary_key: bool,
    pub nullable: bool,
    pub unique: bool,
    pub foreign_key: Option<(String, String)>, // (table, column)
}

impl Column {
    pub fn new(name: &str, sql_type: SqlType) -> Self {
        Column {
            name: name.to_string(),
            sql_type,
            primary_key: false,
            nullable: true,
            unique: false,
            foreign_key: None,
        }
    }

    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false;
        self
    }

    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn foreign_key(mut self, table: &str, column: &str) -> Self {
        self.foreign_key = Some((table.to_string(), column.to_string()));
        self
    }

    pub fn to_sql_string(&self) -> String {
        let mut parts = vec![format!("{} {}", self.name, self.sql_type.to_sql_string())];

        if self.primary_key {
            parts.push("PRIMARY KEY".to_string());
        }

        if !self.nullable {
            parts.push("NOT NULL".to_string());
        }

        if self.unique {
            parts.push("UNIQUE".to_string());
        }

        parts.join(" ")
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

impl Table {
    pub fn new(name: &str) -> Self {
        Table {
            name: name.to_string(),
            columns: Vec::new(),
        }
    }

    pub fn add_column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn create_table_sql(&self) -> String {
        let columns_sql: Vec<String> = self.columns.iter().map(|col| col.to_sql_string()).collect();

        let foreign_keys: Vec<String> = self
            .columns
            .iter()
            .filter_map(|col| {
                col.foreign_key.as_ref().map(|(table, column)| {
                    format!(
                        "FOREIGN KEY ({}) REFERENCES {}({})",
                        col.name, table, column
                    )
                })
            })
            .collect();

        let mut all_constraints = columns_sql.clone();
        all_constraints.extend(foreign_keys);

        format!(
            "CREATE TABLE {} (\n  {}\n);",
            self.name,
            all_constraints.join(",\n  ")
        )
    }

    pub fn generate_insert_statements(&self, count: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let mut statements = Vec::with_capacity(count);

        // Track foreign key values to ensure referential integrity
        let mut foreign_values: HashMap<String, Vec<String>> = HashMap::new();

        for i in 0..count {
            let mut column_names = Vec::new();
            let mut values = Vec::new();

            for column in &self.columns {
                column_names.push(column.name.clone());

                // Check if it's a foreign key that needs to reference existing values
                if let Some((table, col)) = &column.foreign_key {
                    let key = format!("{}.{}", table, col);
                    if let Some(existing_values) = foreign_values.get(&key) {
                        if !existing_values.is_empty() {
                            let idx = rng.gen_range(0..existing_values.len());
                            values.push(existing_values[idx].clone());
                            continue;
                        }
                    }
                }

                // Generate value based on the column type
                let value = column.sql_type.generate_random_value();

                // If this is a primary key or unique column, store it for potential foreign keys
                if column.primary_key || column.unique {
                    let key = format!("{}.{}", self.name, column.name);
                    foreign_values
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(value.clone());
                }

                values.push(value);
            }

            let statement = format!(
                "INSERT INTO {} ({}) VALUES ({});",
                self.name,
                column_names.join(", "),
                values.join(", ")
            );

            statements.push(statement);
        }

        statements
    }

    // Generate a SELECT statement with WHERE clauses
    pub fn generate_select_query(&self, where_clauses: usize) -> String {
        let mut rng = rand::thread_rng();
        let columns = if rng.gen_bool(0.7) {
            "*".to_string()
        } else {
            let num_cols = rng.gen_range(1..=self.columns.len());
            let selected_cols: Vec<String> = self
                .columns
                .iter()
                .map(|col| col.name.clone())
                .collect::<Vec<_>>()
                .choose_multiple(&mut rng, num_cols)
                .cloned()
                .collect();
            selected_cols.join(", ")
        };

        let mut query = format!("SELECT {} FROM {}", columns, self.name);

        if where_clauses > 0 {
            let mut clauses = Vec::new();
            let operators = ["=", ">", "<", ">=", "<=", "<>", "LIKE"];

            for _ in 0..where_clauses {
                let col_idx = rng.gen_range(0..self.columns.len());
                let column = &self.columns[col_idx];

                let op_idx = rng.gen_range(0..operators.len());
                let operator = operators[op_idx];

                let value = if operator == "LIKE" {
                    match column.sql_type {
                        SqlType::Varchar(_) | SqlType::Text => {
                            format!(
                                "'%{}%'",
                                column.sql_type.generate_random_value().trim_matches('\'')
                            )
                        }
                        _ => column.sql_type.generate_random_value(),
                    }
                } else {
                    column.sql_type.generate_random_value()
                };

                clauses.push(format!("{} {} {}", column.name, operator, value));
            }

            query = format!("{} WHERE {}", query, clauses.join(" AND "));
        }

        query + ";"
    }
}

#[derive(Debug, Clone)]
pub struct Schema {
    pub tables: Vec<Table>,
}

impl Schema {
    pub fn new() -> Self {
        Schema { tables: Vec::new() }
    }

    pub fn add_table(mut self, table: Table) -> Self {
        self.tables.push(table);
        self
    }

    pub fn create_schema_sql(&self) -> String {
        self.tables
            .iter()
            .map(|table| table.create_table_sql())
            .collect::<Vec<String>>()
            .join("\n\n")
    }

    pub fn generate_data_sql(&self, rows_per_table: usize) -> String {
        let mut result = String::new();

        // Create tables in proper order (respecting foreign key constraints)
        let mut created_tables = Vec::new();
        let mut tables_to_create: Vec<usize> = (0..self.tables.len()).collect();

        while !tables_to_create.is_empty() {
            let mut progress = false;

            tables_to_create.retain(|&idx| {
                let table = &self.tables[idx];

                // Check if this table can be created now (all foreign tables exist)
                let can_create = table.columns.iter().all(|col| {
                    if let Some((ref fk_table, _)) = col.foreign_key {
                        created_tables.contains(fk_table)
                    } else {
                        true
                    }
                });

                if can_create {
                    created_tables.push(table.name.clone());

                    // Add table creation SQL
                    result.push_str(&table.create_table_sql());
                    result.push_str("\n\n");

                    // Add INSERT statements
                    let inserts = table.generate_insert_statements(rows_per_table);
                    for insert in inserts {
                        result.push_str(&insert);
                        result.push('\n');
                    }
                    result.push('\n');

                    progress = true;
                    false // Remove from tables_to_create
                } else {
                    true // Keep in tables_to_create
                }
            });

            // If we made no progress but still have tables to create, there's a circular dependency
            if !progress && !tables_to_create.is_empty() {
                panic!("Circular foreign key dependencies detected in schema");
            }
        }

        result
    }
}

// Integration with the grammar system
use crate::grammar::{Element, Grammar, GrammarConfig, Production};
use crate::utils::{GrammarError, GrammarValidator, Result};

pub struct SqlGrammarExtension {
    schema: Schema,
}

impl SqlGrammarExtension {
    pub fn new(schema: Schema) -> Self {
        SqlGrammarExtension { schema }
    }

    pub fn extend_grammar(&self, grammar: &mut Grammar) -> Result<()> {
        // Add rules for table names
        let table_names: Vec<&str> = self.schema.tables.iter().map(|t| t.name.as_str()).collect();

        if !table_names.is_empty() {
            grammar.add_rule("table_name", table_names.clone())?;
        }

        // Add rules for column references
        for table in &self.schema.tables {
            let table_columns: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();

            if !table_columns.is_empty() {
                grammar.add_rule(&format!("{}_column", table.name), table_columns)?;
            }

            // Add rules for qualified column references (table.column)
            let mut qualified_columns = Vec::new();
            for column in &table.columns {
                qualified_columns.push(format!("{}.{}", table.name, column.name));
            }

            if !qualified_columns.is_empty() {
                let qualified_strs: Vec<&str> =
                    qualified_columns.iter().map(|s| s.as_str()).collect();

                grammar.add_rule("qualified_column", qualified_strs)?;
            }
        }

        // Add rule for CREATE TABLE statements
        grammar.add_rule("create_table", vec!["<create_table_stmt>"])?;

        // Add rule for SQL queries
        grammar.add_rule(
            "sql_query",
            vec![
                "<select_stmt>",
                "<insert_stmt>",
                "<update_stmt>",
                "<delete_stmt>",
            ],
        )?;

        Ok(())
    }

    // Generate actual SQL based on the schema
    pub fn generate_ddl(&self) -> String {
        self.schema.create_schema_sql()
    }

    pub fn generate_dml(&self, rows_per_table: usize) -> String {
        let mut result = String::new();

        // Generate INSERT statements
        for table in &self.schema.tables {
            let inserts = table.generate_insert_statements(rows_per_table);
            for insert in inserts {
                result.push_str(&insert);
                result.push('\n');
            }
            result.push('\n');
        }

        result
    }

    pub fn generate_queries(&self, count: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let mut queries = Vec::with_capacity(count);

        for _ in 0..count {
            if self.schema.tables.is_empty() {
                continue;
            }

            let table_idx = rng.gen_range(0..self.schema.tables.len());
            let table = &self.schema.tables[table_idx];

            let where_clauses = rng.gen_range(0..=3);
            let query = table.generate_select_query(where_clauses);
            queries.push(query);
        }

        queries
    }
}

// SQL Generator that can be used in place of the regular text generator
pub struct SqlGenerator {
    extension: SqlGrammarExtension,
}

impl SqlGenerator {
    pub fn new(schema: Schema) -> Self {
        SqlGenerator {
            extension: SqlGrammarExtension::new(schema),
        }
    }

    pub fn generate_schema_and_data(&self, rows_per_table: usize) -> String {
        let ddl = self.extension.generate_ddl();
        let dml = self.extension.generate_dml(rows_per_table);

        format!(
            "-- Schema Definition\n{}\n\n-- Data Population\n{}",
            ddl, dml
        )
    }
}
