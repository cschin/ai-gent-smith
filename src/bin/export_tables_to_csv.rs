use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use csv::Writer;
use serde_json::Value;
use sqlx::postgres::PgPool;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the SQL schema file
    #[arg(short, long, value_name = "FILE")]
    schema: PathBuf,

    /// Directory to output CSV files
    #[arg(short, long, value_name = "DIR")]
    output: PathBuf,

    /// Database URL
    #[arg(short, long, env("DATABASE_URL"))]
    database_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let pool = PgPool::connect(&cli.database_url)
        .await
        .context("Failed to connect to the database")?;

    let table_names = extract_table_names(&cli.schema)
        .context("Failed to extract table names from schema file")?;

    for table_name in table_names {
        export_table_to_csv(&pool, &table_name, &cli.output)
            .await
            .context(format!("Failed to export table {} to CSV", table_name))?;
    }

    Ok(())
}

fn extract_table_names(schema_path: &PathBuf) -> Result<Vec<String>> {
    let mut file = File::open(schema_path).context("Failed to open schema file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .context("Failed to read schema file")?;

    let table_names: Vec<String> = contents
        .lines()
        .filter(|line| line.trim().to_lowercase().starts_with("create table"))
        .map(|line| {
            line.split_whitespace()
                .nth(2)
                .unwrap_or("")
                .trim_matches(|c| c == '(' || c == ';' || c == '"')
                .to_string()
        })
        .collect();

    Ok(table_names)
}

async fn export_table_to_csv(pool: &PgPool, table_name: &str, output_dir: &Path) -> Result<()> {
    let query = format!("SELECT * FROM {}", table_name);
    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .context(format!("Failed to fetch data from table {}", table_name))?;

    let file_path = output_dir.join(format!("{}.csv", table_name));
    let mut wtr = Writer::from_path(&file_path).context(format!(
        "Failed to create CSV file for table {}",
        table_name
    ))?;

    if let Some(first_row) = rows.first() {
        let headers: Vec<String> = first_row
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect();
        wtr.write_record(&headers)
            .context("Failed to write headers to CSV")?;
    }

    for (row_index, row) in rows.iter().enumerate() {
        let values: Result<Vec<String>> = row
            .columns()
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let value = row.try_get_raw(i).unwrap();
                if value.is_null() {
                    return Ok("NULL".to_string());
                }
                match col.type_info().name() {
                    "INT4" => row
                        .try_get::<i32, _>(i)
                        .map(|v| v.to_string())
                        .map_err(|e| {
                            anyhow!(
                                "Failed to get INT4 value for column {} in row {}: {:?}",
                                col.name(),
                                row_index,
                                e
                            )
                        }),
                    "INT8" => row
                        .try_get::<i64, _>(i)
                        .map(|v| v.to_string())
                        .map_err(|e| {
                            anyhow!(
                                "Failed to get INT8 value for column {} in row {}: {:?}",
                                col.name(),
                                row_index,
                                e
                            )
                        }),
                    "FLOAT4" => row
                        .try_get::<f32, _>(i)
                        .map(|v| v.to_string())
                        .map_err(|e| {
                            anyhow!(
                                "Failed to get FLOAT4 value for column {} in row {}: {:?}",
                                col.name(),
                                row_index,
                                e
                            )
                        }),
                    "FLOAT8" => row
                        .try_get::<f64, _>(i)
                        .map(|v| v.to_string())
                        .map_err(|e| {
                            anyhow!(
                                "Failed to get FLOAT8 value for column {} in row {}: {:?}",
                                col.name(),
                                row_index,
                                e
                            )
                        }),
                    "BOOL" => row
                        .try_get::<bool, _>(i)
                        .map(|v| v.to_string())
                        .map_err(|e| {
                            anyhow!(
                                "Failed to get BOOL value for column {} in row {}: {:?}",
                                col.name(),
                                row_index,
                                e
                            )
                        }),
                    "TIMESTAMPTZ" => row
                        .try_get::<DateTime<Utc>, _>(i)
                        .map(|v| v.to_rfc3339())
                        .map_err(|e| {
                            anyhow!(
                                "Failed to get TIMESTAMPTZ value for column {} in row {}: {:?}",
                                col.name(),
                                row_index,
                                e
                            )
                        }),
                    "JSONB" => row
                        .try_get::<Value, _>(i)
                        .map(|v| v.to_string())
                        .map_err(|e| {
                            anyhow!(
                                "Failed to get JSONB value for column {} in row {}: {:?}",
                                col.name(),
                                row_index,
                                e
                            )
                        }),
                    _ => row.try_get::<String, _>(i).map_err(|e| {
                        anyhow!(
                            "Failed to get String value for column {} in row {}: {:?}",
                            col.name(),
                            row_index,
                            e
                        )
                    }),
                }
            })
            .collect();

        let values = values.context(format!(
            "Failed to process row {} in table {}",
            row_index, table_name
        ))?;
        wtr.write_record(&values).context(format!(
            "Failed to write row {} to CSV for table {}",
            row_index, table_name
        ))?;
    }

    wtr.flush().context("Failed to flush CSV writer")?;
    println!("Exported table {} to CSV", table_name);

    Ok(())
}
