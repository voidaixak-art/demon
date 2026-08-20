use axiom_engine::cli::run_cli;
use axiom_engine::engine::AxiomEngine;
use axiom_engine::reasoner::Reasoner;
use axiom_engine::server::run_server;
use axiom_engine::storage::StorageManager;
use axiom_engine::vector_engine::VectorEngine;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "axiom_engine")]
#[command(about = "Axiom Engine 2.0: Developmental Neuro-Symbolic Cognitive Mind", long_about = None)]
struct CliArgs {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, default_value = "axiom_brain.db")]
    db: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Axiom Core async daemon server and REST API
    Daemon {
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
    /// Launch interactive cognitive REPL
    Cli,
    /// Run an offline dream / synaptic mutation cycle
    Dream {
        #[arg(short, long, default_value_t = 100)]
        trials: usize,
        #[arg(short, long, default_value_t = 0.50)]
        threshold: f64,
    },
    /// Deduce and mathematically prove the causal connection between two concepts
    Deduce {
        #[arg(short, long)]
        start: String,
        #[arg(short, long)]
        goal: String,
    },
    /// Export the entire brain graph to static JSON files for the 3D Dashboard
    ExportGraph,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    let storage = StorageManager::new(&args.db)?;
    let engine = AxiomEngine::new(storage)?;

    match args.command.unwrap_or(Commands::Daemon { port: 8080 }) {
        Commands::Daemon { port } => {
            run_server(engine, port).await?;
        }
        Commands::Cli => {
            run_cli(engine);
        }
        Commands::Dream { trials, threshold } => {
            println!("🌙 Running one-off Dream Mutation Cycle...");
            let report = VectorEngine::run_dream_mutation(&engine, trials, threshold)?;
            println!("Trials: {}, New Synapses: {}", report.total_trials, report.successful_synapses.len());
            for s in report.successful_synapses {
                println!("  [{}] ⚡ [{}] (score: {:.3})", s.source_concept, s.target_concept, s.similarity);
            }
        }
        Commands::Deduce { start, goal } => {
            let proof = Reasoner::prove_deduction(&engine, &start, &goal);
            println!("{}", proof.explanation);
        }
        Commands::ExportGraph => {
            println!("🚀 Exporting entire Brain Graph for 3D Dashboard...");
            let graph = engine.graph.read().unwrap();
            
            std::fs::create_dir_all("dashboard").unwrap();
            
            let nodes_file = std::fs::File::create("dashboard/nodes.json").unwrap();
            serde_json::to_writer_pretty(nodes_file, &*graph).unwrap();
            
            let mut all_links = Vec::new();
            for node in graph.values() {
                all_links.extend(node.links.clone());
            }
            
            let links_file = std::fs::File::create("dashboard/links.json").unwrap();
            serde_json::to_writer_pretty(links_file, &all_links).unwrap();
            
            println!("✅ Successfully exported {} nodes and {} synapses!", graph.len(), all_links.len());
        }
    }

    Ok(())
}
