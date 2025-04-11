use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// Integration with the grammar system
use crate::grammar::Grammar;
use crate::utils::{GrammarError, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
            SqlType::Float => format!("{:.2}", rng.gen_range(0.0..100.0)),
            SqlType::Varchar(size) => {
                let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
                let length = rng.gen_range(1..*size.min(&20));
                let value = (0..length)
                    .map(|_| chars[rng.gen_range(0..chars.len())])
                    .collect::<String>();
                format!("'{}'", value)
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
                let value = (0..word_count)
                    .map(|_| words[rng.gen_range(0..words.len())])
                    .collect::<Vec<&str>>()
                    .join(" ");
                format!("'{}'", value)
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
                    "'{} {:02}:{:02}:{:02}'",
                    date.trim_matches('\''),
                    hour,
                    minute,
                    second
                )
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub sql_type: SqlType,
    #[serde(default)]
    pub primary_key: bool,
    #[serde(default = "default_true")]
    pub nullable: bool,
    #[serde(default)]
    pub unique: bool,
    #[serde(default)]
    pub auto_increment: bool,
    pub foreign_key: Option<(String, String)>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub on_update: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub tables: Vec<TableDefinition>,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub sql_type: SqlType,
    pub primary_key: bool,
    pub nullable: bool,
    pub unique: bool,
    pub auto_increment: bool,
    pub foreign_key: Option<(String, String)>, // (table, column)
    pub default: Option<String>,
    pub on_update: Option<String>,
}

impl Column {
    pub fn new(name: &str, sql_type: SqlType) -> Self {
        Column {
            name: name.to_string(),
            sql_type,
            primary_key: false,
            nullable: true,
            unique: false,
            auto_increment: false,
            foreign_key: None,
            default: None,
            on_update: None,
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

    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self.primary_key = true;
        self.nullable = false;
        self
    }

    pub fn foreign_key(mut self, table: &str, column: &str) -> Self {
        self.foreign_key = Some((table.to_string(), column.to_string()));
        self
    }

    pub fn to_sql_string(&self) -> String {
        let mut parts = vec![format!("{} {}", self.name, self.sql_type.to_sql_string())];

        if self.auto_increment {
            parts.push("AUTO_INCREMENT".to_string());
        }

        if self.primary_key {
            parts.push("PRIMARY KEY".to_string());
        }

        if !self.nullable {
            parts.push("NOT NULL".to_string());
        }

        if self.unique {
            parts.push("UNIQUE".to_string());
        }

        if let Some(ref default) = self.default {
            parts.push(format!("DEFAULT {}", default));
        }

        if let Some(ref on_update) = self.on_update {
            parts.push(format!("ON UPDATE {}", on_update));
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

        for _ in 0..count {
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

    /// Create a new Schema from a JSON file
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    /// Create a new Schema from a JSON string
    pub fn from_json_str(json: &str) -> Result<Self> {
        let def: SchemaDefinition = serde_json::from_str(json)?;

        let mut schema = Schema::new();
        for table_def in def.tables {
            let mut table = Table::new(&table_def.name);

            for col_def in table_def.columns {
                let mut column = Column::new(&col_def.name, col_def.sql_type);

                if col_def.primary_key {
                    column = column.primary_key();
                }
                if !col_def.nullable {
                    column = column.not_null();
                }
                if col_def.unique {
                    column = column.unique();
                }
                if col_def.auto_increment {
                    column = column.auto_increment();
                }
                if let Some((table, col)) = col_def.foreign_key {
                    column = column.foreign_key(&table, &col);
                }
                if let Some(default) = col_def.default {
                    column.default = Some(default);
                }
                if let Some(on_update) = col_def.on_update {
                    column.on_update = Some(on_update);
                }

                table = table.add_column(column);
            }

            schema = schema.add_table(table);
        }

        Ok(schema)
    }
}

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

/// Load common column types from a JSON file
pub fn load_common_column_types<P: AsRef<Path>>(path: P) -> Result<HashMap<String, Vec<String>>> {
    let content = fs::read_to_string(path).map_err(GrammarError::Io)?;
    let column_types: HashMap<String, Vec<String>> =
        serde_json::from_str(&content).map_err(|e| GrammarError::Json(e))?;
    Ok(column_types)
}

/// Helper function to generate a random schema JSON file with common column types
pub fn generate_random_schema_json_with_types(
    template: &str,
    common_types: Option<&HashMap<String, Vec<String>>>,
) -> Result<String> {
    use rand::seq::SliceRandom;
    use rand::Rng;
    use serde_json::{json, Value};

    let mut rng = rand::thread_rng();
    let template_value: Value = serde_json::from_str(template)?;

    // Extract database name and tables from template
    let database_name = template_value["database"].as_str().unwrap_or("test_db");
    let mut tables_array: Vec<serde_json::Value> = template_value["tables"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    // If no tables specified, generate random table names
    if tables_array.is_empty() {
        let num_tables = rng.gen_range(1..=3);
        for i in 1..=num_tables {
            let table_name = format!("t_{}", i);
            tables_array.push(json!({
                "name": table_name
            }));
        }
    }

    let column_types = if let Some(types) = common_types {
        types.clone()
    } else {
        let mut default_types = HashMap::new();
        let integer_cols = vec![
            "id".to_string(),
            "user_id".to_string(),
            "category_id".to_string(),
            "order_id".to_string(),
        ];
        let varchar_cols = vec![
            "name".to_string(),
            "title".to_string(),
            "description".to_string(),
            "email".to_string(),
            "username".to_string(),
        ];
        let text_cols = vec![
            "content".to_string(),
            "description".to_string(),
            "notes".to_string(),
            "comment".to_string(),
        ];
        let float_cols = vec![
            "price".to_string(),
            "amount".to_string(),
            "total".to_string(),
            "cost".to_string(),
        ];
        let boolean_cols = vec![
            "active".to_string(),
            "enabled".to_string(),
            "published".to_string(),
            "verified".to_string(),
        ];
        let timestamp_cols = vec![
            "created_at".to_string(),
            "updated_at".to_string(),
            "deleted_at".to_string(),
            "last_login".to_string(),
        ];

        default_types.insert("integer".to_string(), integer_cols);
        default_types.insert("varchar".to_string(), varchar_cols);
        default_types.insert("text".to_string(), text_cols);
        default_types.insert("float".to_string(), float_cols);
        default_types.insert("boolean".to_string(), boolean_cols);
        default_types.insert("timestamp".to_string(), timestamp_cols);
        default_types
    };

    let mut tables: Vec<serde_json::Value> = Vec::new();
    let mut foreign_key_refs: Vec<(String, String)> = Vec::new(); // (from_table, to_table)

    for table in tables_array.clone() {
        // Use common column types if provided, otherwise use default types
        let table_name = table["name"].as_str().unwrap_or("unknown");
        let mut columns = Vec::new();

        // Always add an ID column
        columns.push(json!({
            "name": table_name.to_string() + "_id",
            "type": "integer",
            "primary_key": true,
            "nullable": false,
            "auto_increment": true
        }));

        // Add random number of additional columns (2-5)
        let num_columns = rng.gen_range(2..=5);
        for _ in 0..num_columns {
            let type_names: Vec<_> = column_types.keys().collect();
            let type_name = type_names.choose(&mut rng).unwrap();
            let possible_names = column_types.get(*type_name).unwrap();
            let column_name = possible_names.choose(&mut rng).unwrap();

            let mut column = json!({
                "name": table_name.to_string() + "_" + column_name + "_" + rng.gen_range(0..1000).to_string().as_str(),
                "type": if *type_name == "varchar" {
                    json!({"varchar": rng.gen_range(50..=255)})
                } else {
                    json!(type_name)
                },
                "nullable": rng.gen_bool(0.7),
                "unique": rng.gen_bool(0.2)
            });

            // Add timestamp defaults for created_at/updated_at
            if *column_name == "created_at" || *column_name == "updated_at" {
                column["default"] = json!("CURRENT_TIMESTAMP");
                if *column_name == "updated_at" {
                    column["on_update"] = json!("CURRENT_TIMESTAMP");
                }
            }

            columns.push(column);
        }

        // Add foreign key with 30% probability
        if rng.gen_bool(0.3) && tables_array.len() > 1 {
            let other_tables = tables_array
                .iter()
                .filter_map(|t| t["name"].as_str())
                .filter(|&t| t != table_name)
                .collect::<Vec<_>>();

            if let Some(ref_table) = other_tables.choose(&mut rng) {
                // Check if adding this foreign key would create a cycle
                let mut temp_refs = foreign_key_refs.clone();
                temp_refs.push((table_name.to_string(), ref_table.to_string()));

                if !has_cycle(&temp_refs) {
                    foreign_key_refs.push((table_name.to_string(), ref_table.to_string()));
                    columns.push(json!({
                        "name": format!("{}_id", ref_table),
                        "type": "integer",
                        "nullable": false,
                        "foreign_key": [ref_table, format!("{}_id", ref_table)]
                    }));
                }
            }
        }

        tables.push(json!({
            "name": table_name,
            "columns": columns
        }));
    }

    // Create the final JSON schema
    let schema = json!({
        "database": database_name,
        "tables": tables
    });

    Ok(serde_json::to_string_pretty(&schema)?)
}

/// Helper function to detect cycles in foreign key references
fn has_cycle(refs: &[(String, String)]) -> bool {
    use std::collections::{HashMap, HashSet};

    // Build adjacency list
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for (from, to) in refs {
        graph.entry(from.clone()).or_default().push(to.clone());
    }

    // DFS to detect cycles
    let mut visited = HashSet::new();
    let mut recursion_stack = HashSet::new();

    for node in graph.keys() {
        if !visited.contains(node) {
            if is_cyclic_util(node, &graph, &mut visited, &mut recursion_stack) {
                return true;
            }
        }
    }

    false
}

fn is_cyclic_util(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    recursion_stack: &mut HashSet<String>,
) -> bool {
    if recursion_stack.contains(node) {
        return true;
    }

    if visited.contains(node) {
        return false;
    }

    visited.insert(node.to_string());
    recursion_stack.insert(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if is_cyclic_util(neighbor, graph, visited, recursion_stack) {
                return true;
            }
        }
    }

    recursion_stack.remove(node);
    false
}

/// Load schema data from a JSON file, with optional fallback to random generation
pub fn load_schema_from_file<P: AsRef<Path>>(path: P, fallback_to_random: bool) -> Result<String> {
    use std::fs;

    // Try to read the file
    match fs::read_to_string(&path) {
        Ok(content) => {
            // Try to parse the JSON
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => Ok(content), // Valid JSON, return it
                Err(_) if fallback_to_random => {
                    // Invalid JSON but fallback allowed, generate random schema
                    generate_random_schema_json_with_types("{}", None)
                }
                Err(e) => Err(GrammarError::Json(e)), // Invalid JSON and no fallback
            }
        }
        Err(_e) if fallback_to_random => {
            // File not found but fallback allowed, generate random schema
            generate_random_schema_json_with_types("{}", None)
        }
        Err(e) => Err(GrammarError::Io(e)), // File not found and no fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_schema() -> Schema {
        let users = Table::new("users")
            .add_column(Column::new("id", SqlType::Integer).primary_key())
            .add_column(Column::new("name", SqlType::Varchar(50)).not_null())
            .add_column(Column::new("email", SqlType::Varchar(100)).unique());

        let posts = Table::new("posts")
            .add_column(Column::new("id", SqlType::Integer).primary_key())
            .add_column(Column::new("user_id", SqlType::Integer).foreign_key("users", "id"))
            .add_column(Column::new("title", SqlType::Varchar(200)))
            .add_column(Column::new("content", SqlType::Text))
            .add_column(Column::new("published", SqlType::Boolean))
            .add_column(Column::new("created_at", SqlType::Timestamp));

        Schema::new().add_table(users).add_table(posts)
    }

    #[test]
    fn test_sql_type_string() {
        assert_eq!(SqlType::Integer.to_sql_string(), "INTEGER");
        assert_eq!(SqlType::Float.to_sql_string(), "FLOAT");
        assert_eq!(SqlType::Varchar(50).to_sql_string(), "VARCHAR(50)");
        assert_eq!(SqlType::Text.to_sql_string(), "TEXT");
        assert_eq!(SqlType::Boolean.to_sql_string(), "BOOLEAN");
        assert_eq!(SqlType::Date.to_sql_string(), "DATE");
        assert_eq!(SqlType::Timestamp.to_sql_string(), "TIMESTAMP");
    }

    #[test]
    fn test_random_value_generation() {
        let mut seen_values = std::collections::HashSet::new();
        for _ in 0..10 {
            // Test integer generation
            let int_val = SqlType::Integer.generate_random_value();
            assert!(int_val.parse::<i32>().is_ok());

            // Test float generation
            let float_val = SqlType::Float.generate_random_value();
            assert!(float_val.parse::<f64>().is_ok());

            // Test varchar generation
            let varchar_val = SqlType::Varchar(10).generate_random_value();
            assert!(varchar_val.len() <= 10);

            // Test uniqueness of random values
            seen_values.insert(varchar_val);
        }
        // Ensure we got some unique values
        assert!(seen_values.len() > 1);
    }

    #[test]
    fn test_column_constraints() {
        let col = Column::new("test", SqlType::Integer)
            .primary_key()
            .unique()
            .not_null();

        let sql = col.to_sql_string();
        assert!(sql.contains("PRIMARY KEY"));
        assert!(sql.contains("UNIQUE"));
        assert!(sql.contains("NOT NULL"));
    }

    #[test]
    fn test_foreign_key() {
        let col = Column::new("user_id", SqlType::Integer).foreign_key("users", "id");

        assert_eq!(
            col.foreign_key,
            Some(("users".to_string(), "id".to_string()))
        );
    }

    #[test]
    fn test_table_creation() {
        let schema = create_test_schema();
        let sql = schema.create_schema_sql();

        // Check for table creation
        assert!(sql.contains("CREATE TABLE users"));
        assert!(sql.contains("CREATE TABLE posts"));

        // Check for constraints
        assert!(sql.contains("PRIMARY KEY"));
        assert!(sql.contains("FOREIGN KEY"));
        assert!(sql.contains("NOT NULL"));
        assert!(sql.contains("UNIQUE"));
    }

    #[test]
    fn test_data_generation() {
        let schema = create_test_schema();
        let sql = schema.generate_data_sql(5);

        // Check for insert statements
        assert!(sql.contains("INSERT INTO users"));
        assert!(sql.contains("INSERT INTO posts"));

        // Count the number of inserts
        let user_inserts = sql.matches("INSERT INTO users").count();
        let post_inserts = sql.matches("INSERT INTO posts").count();

        assert_eq!(user_inserts, 5);
        assert_eq!(post_inserts, 5);
    }

    #[test]
    fn test_select_query_generation() {
        let users = Table::new("users")
            .add_column(Column::new("id", SqlType::Integer).primary_key())
            .add_column(Column::new("name", SqlType::Varchar(50)));

        let query = users.generate_select_query(1);

        assert!(query.starts_with("SELECT"));
        assert!(query.contains("FROM users"));
        assert!(query.contains("WHERE"));
    }

    #[test]
    fn test_sql_generator() {
        let schema = create_test_schema();
        let generator = SqlGenerator::new(schema);

        let output = generator.generate_schema_and_data(3);
        assert!(output.contains("-- Schema Definition"));
        assert!(output.contains("-- Data Population"));

        // Verify schema and data are included
        assert!(output.contains("CREATE TABLE"));
        assert!(output.contains("INSERT INTO"));
    }

    #[test]
    fn test_schema_from_json() {
        let json = r#"{
            "tables": [
                {
                    "name": "users",
                    "columns": [
                        {
                            "name": "id",
                            "type": "integer",
                            "primary_key": true,
                            "nullable": false
                        },
                        {
                            "name": "email",
                            "type": {"varchar": 100},
                            "unique": true,
                            "nullable": false
                        }
                    ]
                },
                {
                    "name": "posts",
                    "columns": [
                        {
                            "name": "id",
                            "type": "integer",
                            "primary_key": true
                        },
                        {
                            "name": "user_id",
                            "type": "integer",
                            "foreign_key": ["users", "id"]
                        },
                        {
                            "name": "title",
                            "type": {"varchar": 200}
                        },
                        {
                            "name": "content",
                            "type": "text"
                        }
                    ]
                }
            ]
        }"#;

        let schema = Schema::from_json_str(json).unwrap();

        println!("{}", schema.create_schema_sql());

        // Verify the schema was created correctly
        assert_eq!(schema.tables.len(), 2);

        // Check users table
        let users = &schema.tables[0];
        assert_eq!(users.name, "users");
        assert_eq!(users.columns.len(), 2);

        // Check posts table
        let posts = &schema.tables[1];
        assert_eq!(posts.name, "posts");
        assert_eq!(posts.columns.len(), 4);

        // Verify foreign key
        let user_id_col = &posts.columns[1];
        assert_eq!(
            user_id_col.foreign_key,
            Some(("users".to_string(), "id".to_string()))
        );
    }

    #[test]
    fn test_schema_from_json_file() -> std::result::Result<(), Box<dyn Error>> {
        // Create a temporary JSON file
        let mut temp_file = NamedTempFile::new()?;
        let json_schema = r#"{
            "database": "test_db",
            "tables": [
                {
                    "name": "categories",
                    "columns": [
                        {
                            "name": "id",
                            "type": "integer",
                            "primary_key": true,
                            "nullable": false,
                            "auto_increment": true
                        },
                        {
                            "name": "name",
                            "type": {"varchar": 50},
                            "nullable": false,
                            "unique": true
                        },
                        {
                            "name": "created_at",
                            "type": "timestamp",
                            "nullable": false,
                            "default": "CURRENT_TIMESTAMP"
                        },
                        {
                            "name": "updated_at",
                            "type": "timestamp",
                            "nullable": false,
                            "default": "CURRENT_TIMESTAMP",
                            "on_update": "CURRENT_TIMESTAMP"
                        }
                    ]
                },
                {
                    "name": "products",
                    "columns": [
                        {
                            "name": "id",
                            "type": "integer",
                            "primary_key": true,
                            "nullable": false,
                            "auto_increment": true
                        },
                        {
                            "name": "name",
                            "type": {"varchar": 100},
                            "nullable": false
                        },
                        {
                            "name": "category_id",
                            "type": "integer",
                            "nullable": false,
                            "foreign_key": ["categories", "id"]
                        },
                        {
                            "name": "price",
                            "type": "float",
                            "nullable": false
                        },
                        {
                            "name": "description",
                            "type": "text"
                        },
                        {
                            "name": "created_at",
                            "type": "timestamp",
                            "nullable": false,
                            "default": "CURRENT_TIMESTAMP"
                        },
                        {
                            "name": "updated_at",
                            "type": "timestamp",
                            "nullable": false,
                            "default": "CURRENT_TIMESTAMP",
                            "on_update": "CURRENT_TIMESTAMP"
                        }
                    ]
                }
            ]
        }"#;

        temp_file.write_all(json_schema.as_bytes())?;

        // Import schema from the temporary file
        let schema = Schema::from_json_file(temp_file.path())?;

        // Verify schema structure
        assert_eq!(schema.tables.len(), 2);

        // Check categories table
        let categories = &schema.tables[0];
        assert_eq!(categories.name, "categories");
        assert_eq!(categories.columns.len(), 4);
        assert!(categories.columns[0].primary_key);
        assert!(categories.columns[1].unique);

        // Check products table
        let products = &schema.tables[1];
        assert_eq!(products.name, "products");
        assert_eq!(products.columns.len(), 7);
        assert_eq!(
            products.columns[2].foreign_key,
            Some(("categories".to_string(), "id".to_string()))
        );

        // Generate complete SQL statements
        let mut sql = String::new();

        // Add database creation
        sql.push_str("-- Create database\n");
        sql.push_str("CREATE DATABASE IF NOT EXISTS test_db;\n");
        sql.push_str("USE test_db;\n\n");

        // Add schema creation
        sql.push_str("-- Create tables\n");
        sql.push_str(&schema.create_schema_sql());
        sql.push_str("\n\n");

        // Add cleanup
        sql.push_str("-- Cleanup\n");
        sql.push_str("DROP DATABASE IF EXISTS test_db;\n");

        // Verify SQL contains all expected statements
        assert!(sql.contains("CREATE DATABASE"));
        assert!(sql.contains("USE test_db"));
        assert!(sql.contains("CREATE TABLE categories"));
        assert!(sql.contains("CREATE TABLE products"));
        assert!(sql.contains("FOREIGN KEY (category_id) REFERENCES categories(id)"));
        assert!(sql.contains("DROP DATABASE"));
        assert!(sql.contains("AUTO_INCREMENT"));
        assert!(sql.contains("DEFAULT CURRENT_TIMESTAMP"));
        assert!(sql.contains("ON UPDATE CURRENT_TIMESTAMP"));

        Ok(())
    }

    #[test]
    fn test_generate_random_schema() -> Result<()> {
        let template = r#"{
            "database": "test_db",
            "tables": [
                { "name": "users" },
                { "name": "posts" },
                { "name": "categories" }
            ]
        }"#;

        let schema_json = generate_random_schema_json_with_types(template, None)?;
        println!("Generated schema:\n{}", schema_json);

        // Verify the schema can be parsed
        let schema = Schema::from_json_str(&schema_json)?;
        assert_eq!(schema.tables.len(), 3);

        // Verify each table has at least an ID column
        for table in &schema.tables {
            let table_name = &table.name.clone();
            assert!(table
                .columns
                .iter()
                .any(|col| col.name == table_name.to_string() + "_id" && col.primary_key));
        }

        Ok(())
    }

    #[test]
    fn test_load_schema_from_file() -> Result<()> {
        // Test with valid JSON file
        let mut valid_file = NamedTempFile::new()?;
        let valid_json = r#"{
            "database": "test_db",
            "tables": [
                {
                    "name": "users",
                    "columns": [
                        {
                            "name": "id",
                            "type": "integer",
                            "primary_key": true
                        }
                    ]
                }
            ]
        }"#;
        valid_file.write_all(valid_json.as_bytes())?;

        let result = load_schema_from_file(valid_file.path(), false)?;
        assert!(result.contains("test_db"));
        assert!(result.contains("users"));

        // Test with invalid JSON file and fallback
        let mut invalid_file = NamedTempFile::new()?;
        invalid_file.write_all(b"invalid json")?;

        let result = load_schema_from_file(invalid_file.path(), true)?;
        assert!(result.contains("database"));
        assert!(result.contains("tables"));

        // Test with non-existent file and fallback
        let result = load_schema_from_file("nonexistent.json", true)?;
        assert!(result.contains("database"));
        assert!(result.contains("tables"));

        // Test with non-existent file and no fallback
        let result = load_schema_from_file("nonexistent.json", false);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_cycle_detection() -> Result<()> {
        // Test case with no cycles
        let no_cycle_refs = vec![
            ("users".to_string(), "posts".to_string()),
            ("posts".to_string(), "comments".to_string()),
        ];
        assert!(!has_cycle(&no_cycle_refs));

        // Test case with a cycle
        let cycle_refs = vec![
            ("users".to_string(), "posts".to_string()),
            ("posts".to_string(), "comments".to_string()),
            ("comments".to_string(), "users".to_string()),
        ];
        assert!(has_cycle(&cycle_refs));

        // Test case with multiple cycles
        let multiple_cycles = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
            ("c".to_string(), "a".to_string()),
            ("d".to_string(), "e".to_string()),
            ("e".to_string(), "d".to_string()),
        ];
        assert!(has_cycle(&multiple_cycles));

        Ok(())
    }

    #[test]
    fn test_schema_generation_with_foreign_keys() -> Result<()> {
        let template = r#"{
            "database": "test_db",
            "tables": [
                { "name": "users" },
                { "name": "posts" },
                { "name": "comments" }
            ]
        }"#;

        let schema_json = generate_random_schema_json_with_types(template, None)?;
        let schema = Schema::from_json_str(&schema_json)?;

        // Verify the schema structure
        assert_eq!(schema.tables.len(), 3);

        // Verify each table has an ID column
        for table in &schema.tables {
            let table_name = &table.name;
            assert!(table
                .columns
                .iter()
                .any(|col| col.name == format!("{}_id", table_name) && col.primary_key));
        }

        // Verify foreign key relationships form a DAG
        let mut foreign_key_refs = Vec::new();
        for table in &schema.tables {
            for column in &table.columns {
                if let Some((ref_table, _)) = &column.foreign_key {
                    foreign_key_refs.push((table.name.clone(), ref_table.clone()));
                }
            }
        }
        assert!(!has_cycle(&foreign_key_refs));

        Ok(())
    }

    #[test]
    fn test_schema_validation() -> Result<()> {
        let template = r#"{
            "database": "test_db",
            "tables": [
                { "name": "users" },
                { "name": "posts" }
            ]
        }"#;

        let schema_json = generate_random_schema_json_with_types(template, None)?;
        let schema = Schema::from_json_str(&schema_json)?;

        // Verify database name
        assert!(schema_json.contains("test_db"));

        // Verify each table has required columns
        for table in &schema.tables {
            // Check for ID column
            assert!(table
                .columns
                .iter()
                .any(|col| col.name.ends_with("_id") && col.primary_key));

            // Check for at least one non-ID column
            assert!(table.columns.len() > 1);

            // Verify foreign key constraints
            for column in &table.columns {
                if let Some((ref_table, ref_column)) = &column.foreign_key {
                    // Verify referenced table exists
                    assert!(schema.tables.iter().any(|t| t.name == *ref_table));
                    // Verify referenced column exists in the referenced table
                    let ref_table = schema.tables.iter().find(|t| t.name == *ref_table).unwrap();
                    assert!(ref_table.columns.iter().any(|col| col.name == *ref_column));
                }
            }

            // Verify column types are valid
            for column in &table.columns {
                match &column.sql_type {
                    SqlType::Varchar(size) => assert!(*size > 0 && *size <= 255),
                    _ => (), // Other types don't have size constraints
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_schema_generation_with_custom_types() -> Result<()> {
        let mut custom_types = HashMap::new();
        custom_types.insert(
            "integer".to_string(),
            vec![
                "id".to_string(),
                "user_id".to_string(),
                "post_id".to_string(),
            ],
        );
        custom_types.insert(
            "varchar".to_string(),
            vec![
                "name".to_string(),
                "title".to_string(),
                "content".to_string(),
            ],
        );

        let template = r#"{
            "database": "test_db",
            "tables": [
                { "name": "users" },
                { "name": "posts" }
            ]
        }"#;

        let schema_json = generate_random_schema_json_with_types(template, Some(&custom_types))?;
        let schema = Schema::from_json_str(&schema_json)?;

        // Verify custom type usage
        for table in &schema.tables {
            for column in &table.columns {
                if column.name.contains("id") {
                    assert!(matches!(column.sql_type, SqlType::Integer));
                } else if column.name.contains("name")
                    || column.name.contains("title")
                    || column.name.contains("content")
                {
                    assert!(matches!(column.sql_type, SqlType::Varchar(_)));
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_schema_generation_edge_cases() -> Result<()> {
        // Test with empty template
        let empty_template = "{}";
        let schema_json = generate_random_schema_json_with_types(empty_template, None)?;
        let schema = Schema::from_json_str(&schema_json)?;
        assert!(!schema.tables.is_empty());

        // Test with single table
        let single_table_template = r#"{
            "database": "test_db",
            "tables": [
                { "name": "users" }
            ]
        }"#;
        let schema_json = generate_random_schema_json_with_types(single_table_template, None)?;
        let schema = Schema::from_json_str(&schema_json)?;
        assert_eq!(schema.tables.len(), 1);

        // Test with many tables
        let many_tables_template = r#"{
            "database": "test_db",
            "tables": [
                { "name": "t1" }, { "name": "t2" }, { "name": "t3" },
                { "name": "t4" }, { "name": "t5" }, { "name": "t6" }
            ]
        }"#;
        let schema_json = generate_random_schema_json_with_types(many_tables_template, None)?;
        let schema = Schema::from_json_str(&schema_json)?;
        println!("Schema:\n{}", schema_json);
        assert_eq!(schema.tables.len(), 6);

        Ok(())
    }
}
