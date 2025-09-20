use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "langgraph-rs")]
#[command(about = "Convert LangGraph Python workflows to Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a LangGraph workflow to Rust
    Convert {
        /// Python file containing the LangGraph workflow
        #[arg(value_name = "PYTHON_FILE")]
        input: PathBuf,

        /// Output directory for generated Rust code
        #[arg(short, long, default_value = "./generated")]
        output: PathBuf,

        /// Generate tests alongside the code
        #[arg(long)]
        with_tests: bool,
    },

    /// Inspect a LangGraph workflow and output metadata
    Inspect {
        /// Python file containing the LangGraph workflow
        #[arg(value_name = "PYTHON_FILE")]
        graph: PathBuf,

        /// Output format (json, yaml, toml)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Validate that Rust code matches Python behavior
    Validate {
        /// Original Python file
        #[arg(long)]
        python: PathBuf,

        /// Generated Rust code
        #[arg(long)]
        rust: PathBuf,

        /// Test data file
        #[arg(long)]
        test_data: Option<PathBuf>,
    },

    /// Generate a visualization of the graph
    Visualize {
        /// Python file containing the LangGraph workflow
        #[arg(value_name = "PYTHON_FILE")]
        graph: PathBuf,

        /// Output format (mermaid, dot, svg)
        #[arg(short, long, default_value = "mermaid")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let filter = match cli.verbose {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()))
        .init();

    match cli.command {
        Commands::Convert { input, output, with_tests } => {
            convert_workflow(input, output, with_tests).await?;
        }
        Commands::Inspect { graph, format } => {
            inspect_graph(graph, format).await?;
        }
        Commands::Validate { python, rust, test_data } => {
            validate_conversion(python, rust, test_data).await?;
        }
        Commands::Visualize { graph, format, output } => {
            visualize_graph(graph, format, output).await?;
        }
    }

    Ok(())
}

async fn convert_workflow(input: PathBuf, output: PathBuf, with_tests: bool) -> Result<()> {
    info!("Converting LangGraph workflow: {:?}", input);

    // TODO: Implement Python execution to extract graph info
    // For now, we'll use a placeholder

    // Step 1: Extract graph info from Python
    info!("Extracting graph metadata...");
    let graph_json = extract_graph_info(&input)?;

    // Step 2: Generate Rust code
    info!("Generating Rust code...");
    let rust_code = langgraph_generator::generate_from_json(&graph_json)?;

    // Step 3: Create output directory
    fs::create_dir_all(&output)?;

    // Step 4: Write generated code
    let main_file = output.join("lib.rs");
    fs::write(&main_file, rust_code)?;
    info!("Generated Rust code written to: {:?}", main_file);

    // Step 5: Generate Cargo.toml
    let cargo_toml = generate_cargo_toml(&input);
    fs::write(output.join("Cargo.toml"), cargo_toml)?;

    // Step 6: Generate tests if requested
    if with_tests {
        let test_code = generate_test_code(&graph_json)?;
        fs::write(output.join("tests.rs"), test_code)?;
        info!("Generated tests written to: {:?}", output.join("tests.rs"));
    }

    info!("Conversion complete! Output written to: {:?}", output);
    Ok(())
}

async fn inspect_graph(graph: PathBuf, format: String) -> Result<()> {
    info!("Inspecting graph: {:?}", graph);

    let graph_json = extract_graph_info(&graph)?;

    match format.as_str() {
        "json" => {
            println!("{}", graph_json);
        }
        "yaml" => {
            // TODO: Convert to YAML
            error!("YAML format not yet implemented");
        }
        "toml" => {
            // TODO: Convert to TOML
            error!("TOML format not yet implemented");
        }
        _ => {
            error!("Unknown format: {}", format);
        }
    }

    Ok(())
}

async fn validate_conversion(python: PathBuf, rust: PathBuf, test_data: Option<PathBuf>) -> Result<()> {
    info!("Validating conversion...");
    info!("Python: {:?}", python);
    info!("Rust: {:?}", rust);

    if let Some(data) = test_data {
        info!("Using test data: {:?}", data);
    }

    // TODO: Implement validation logic
    // 1. Run Python version with test data
    // 2. Run Rust version with same data
    // 3. Compare outputs

    error!("Validation not yet implemented");
    Ok(())
}

async fn visualize_graph(graph: PathBuf, format: String, output: Option<PathBuf>) -> Result<()> {
    info!("Visualizing graph: {:?}", graph);

    let graph_json = extract_graph_info(&graph)?;
    let graph_info: langgraph_inspector::GraphInfo = serde_json::from_str(&graph_json)?;

    let visualization = match format.as_str() {
        "mermaid" => generate_mermaid(&graph_info),
        "dot" => generate_dot(&graph_info),
        _ => {
            error!("Unknown format: {}", format);
            return Ok(());
        }
    };

    if let Some(output_file) = output {
        fs::write(output_file, visualization)?;
        info!("Visualization written to file");
    } else {
        println!("{}", visualization);
    }

    Ok(())
}

fn extract_graph_info(input: &PathBuf) -> Result<String> {
    // Check if Python file exists
    if !input.exists() {
        return Err(anyhow::anyhow!("Python file not found: {:?}", input));
    }

    // NOTE: Full PyO3 integration requires Python runtime setup
    // This would typically involve:
    // 1. Initialize Python interpreter
    // 2. Load the langgraph module
    // 3. Execute the Python file
    // 4. Call langgraph_inspector::extract_graph_info

    // For now, return example data with a warning
    eprintln!("WARNING: Python introspection not yet fully implemented");
    eprintln!("Using example graph metadata for demonstration purposes");

    // Return example that matches simple_workflow.py structure
    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "nodes": [
            {
                "name": "start",
                "func_name": "start_node",
                "signature": "(state: AgentState) -> AgentState",
                "docstring": "Initial processing node",
                "source_hint": null
            },
            {
                "name": "process",
                "func_name": "process_node",
                "signature": "(state: AgentState) -> AgentState",
                "docstring": "Main processing logic",
                "source_hint": null
            },
            {
                "name": "end",
                "func_name": "end_node",
                "signature": "(state: AgentState) -> AgentState",
                "docstring": "Cleanup and finalization",
                "source_hint": null
            }
        ],
        "edges": [
            {"from": "start", "to": "process", "condition": null},
            {"from": "process", "to": "process", "condition": "counter < 3"},
            {"from": "process", "to": "end", "condition": "counter >= 3"},
            {"from": "end", "to": "__end__", "condition": null}
        ],
        "state_schema": {
            "fields": [
                {
                    "name": "messages",
                    "type_name": "list",
                    "is_optional": false,
                    "default_value": []
                },
                {
                    "name": "context",
                    "type_name": "dict",
                    "is_optional": false,
                    "default_value": {}
                },
                {
                    "name": "next_action",
                    "type_name": "str",
                    "is_optional": false,
                    "default_value": ""
                },
                {
                    "name": "counter",
                    "type_name": "int",
                    "is_optional": false,
                    "default_value": 0
                }
            ]
        },
        "entry_point": "start",
        "conditional_edges": {
            "process": {
                "condition_func": "route_next",
                "branches": {
                    "process": "process",
                    "end": "end"
                }
            }
        }
    }))?)
}

fn generate_cargo_toml(input: &PathBuf) -> String {
    let name = input.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("generated_graph");

    // Use path dependency to langgraph-runtime in the workspace
    // This assumes the generated project is within or near the workspace
    format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.40", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = "1.0"
async-trait = "0.1"
# Use workspace path dependency until crate is published
langgraph-runtime = {{ path = "../langgraph-runtime" }}
"#, name)
}

fn generate_test_code(_graph_json: &str) -> Result<String> {
    Ok(r#"#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graph_execution() {
        let initial_state = GraphState::new();
        let result = run_graph(initial_state).await.unwrap();
        // TODO: Add assertions
    }
}
"#.to_string())
}

fn generate_mermaid(graph_info: &langgraph_inspector::GraphInfo) -> String {
    let mut mermaid = String::from("graph TD\n");

    for node in &graph_info.nodes {
        mermaid.push_str(&format!("    {}[{}]\n", node.name, node.name));
    }

    for edge in &graph_info.edges {
        mermaid.push_str(&format!("    {} --> {}\n", edge.from, edge.to));
    }

    mermaid
}

fn generate_dot(graph_info: &langgraph_inspector::GraphInfo) -> String {
    let mut dot = String::from("digraph G {\n");

    for node in &graph_info.nodes {
        dot.push_str(&format!("    \"{}\";\n", node.name));
    }

    for edge in &graph_info.edges {
        dot.push_str(&format!("    \"{}\" -> \"{}\";\n", edge.from, edge.to));
    }

    dot.push_str("}\n");
    dot
}