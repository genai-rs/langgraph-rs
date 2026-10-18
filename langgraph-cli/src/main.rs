use anyhow::{bail, ensure, Context, Result};
use clap::{Parser, Subcommand};
use langgraph_cli::package_writer::write_generated_package;
use langgraph_cli::python_adapter::{self, ExtractionOptions};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "langgraph-rs")]
#[command(version)]
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

        #[command(flatten)]
        extraction: ExtractionOptions,
    },

    /// Inspect a LangGraph workflow and output metadata
    Inspect {
        /// Python file containing the LangGraph workflow
        #[arg(value_name = "PYTHON_FILE")]
        graph: PathBuf,

        /// Output format (json; yaml and toml are not implemented)
        #[arg(short, long, default_value = "json")]
        format: String,

        #[command(flatten)]
        extraction: ExtractionOptions,
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

        /// Output format (mermaid, dot; svg is not implemented)
        #[arg(short, long, default_value = "mermaid")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        #[command(flatten)]
        extraction: ExtractionOptions,
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
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()))
        .init();

    match cli.command {
        Commands::Convert {
            input,
            output,
            with_tests,
            extraction,
        } => {
            convert_workflow(input, output, with_tests, extraction).await?;
        }
        Commands::Inspect {
            graph,
            format,
            extraction,
        } => {
            inspect_graph(graph, format, extraction).await?;
        }
        Commands::Validate {
            python,
            rust,
            test_data,
        } => {
            validate_conversion(python, rust, test_data).await?;
        }
        Commands::Visualize {
            graph,
            format,
            output,
            extraction,
        } => {
            visualize_graph(graph, format, output, extraction).await?;
        }
    }

    Ok(())
}

async fn convert_workflow(
    input: PathBuf,
    output: PathBuf,
    with_tests: bool,
    extraction: ExtractionOptions,
) -> Result<()> {
    info!("Converting LangGraph workflow: {:?}", input);

    // Step 1: Extract graph info from Python
    info!("Extracting graph metadata...");
    let graph_info = python_adapter::extract_graph_info(&input, &extraction).await?;
    let graph_json = serde_json::to_string_pretty(&graph_info)?;

    // Step 2: Generate Rust code
    info!("Generating Rust code...");
    let rust_code = langgraph_generator::generate_from_json(&graph_json)?;

    // Step 3: Write a complete, discoverable Cargo package.
    let package_name =
        write_generated_package(&output, &input, &rust_code, &graph_json, with_tests)?;
    info!(
        "Generated Cargo package {package_name} written to: {:?}",
        output
    );

    info!("Conversion complete! Output written to: {:?}", output);
    Ok(())
}

async fn inspect_graph(
    graph: PathBuf,
    format: String,
    extraction: ExtractionOptions,
) -> Result<()> {
    info!("Inspecting graph: {:?}", graph);

    ensure!(
        format == "json",
        "Unsupported inspect format: {format}; only json is supported"
    );
    let graph_info = python_adapter::extract_graph_info(&graph, &extraction).await?;
    let graph_json = serde_json::to_string_pretty(&graph_info)?;
    println!("{}", graph_json);

    Ok(())
}

async fn validate_conversion(
    python: PathBuf,
    rust: PathBuf,
    test_data: Option<PathBuf>,
) -> Result<()> {
    info!("Validating conversion...");
    info!("Python: {:?}", python);
    info!("Rust: {:?}", rust);

    validate_input_file(&python)?;
    // Rust input may be a source file or a generated project directory.
    fs::metadata(&rust).with_context(|| format!("Cannot access Rust input: {}", rust.display()))?;
    if let Some(data) = test_data {
        validate_input_file(&data)?;
    }

    bail!("Validation is not implemented")
}

async fn visualize_graph(
    graph: PathBuf,
    format: String,
    output: Option<PathBuf>,
    extraction: ExtractionOptions,
) -> Result<()> {
    info!("Visualizing graph: {:?}", graph);

    ensure!(
        matches!(format.as_str(), "mermaid" | "dot"),
        "Unsupported visualization format: {format}; supported formats are mermaid and dot"
    );
    let graph_info = python_adapter::extract_graph_info(&graph, &extraction).await?;

    let visualization = match format.as_str() {
        "mermaid" => generate_mermaid(&graph_info),
        "dot" => generate_dot(&graph_info),
        _ => bail!("Unsupported visualization format: {format}"),
    };

    if let Some(output_file) = output {
        fs::write(output_file, visualization)?;
        info!("Visualization written to file");
    } else {
        println!("{}", visualization);
    }

    Ok(())
}

fn validate_input_file(input: &Path) -> Result<()> {
    ensure!(
        fs::metadata(input)
            .with_context(|| format!("Cannot inspect input file: {}", input.display()))?
            .is_file(),
        "Input is not a regular file: {}",
        input.display()
    );
    fs::File::open(input)
        .with_context(|| format!("Cannot open input file: {}", input.display()))?;
    Ok(())
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
