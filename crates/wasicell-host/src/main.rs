use clap::{Parser, Subcommand};
use wasicell_common::AppManifest;
use std::path::PathBuf;

mod runtime;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a wasicell app from a manifest
    Run {
        /// Path to the app.toml manifest
        manifest: PathBuf,
        /// Optional service name to run (if omitted, runs all)
        service: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { manifest, service } => {
            let app_manifest = AppManifest::load_from_file(&manifest)?;
            if let Some(svc) = service {
                if let Some(config) = app_manifest.service.get(&svc) {
                    tracing::info!("Running single service: {}", svc);
                    runtime::run_service(&svc, config, &manifest.parent().unwrap_or(std::path::Path::new(".")))?;
                } else {
                    anyhow::bail!("Service '{}' not found in manifest", svc);
                }
            } else {
                tracing::info!("Running all services from manifest");
                for (name, config) in &app_manifest.service {
                    tracing::info!("Starting service: {}", name);
                    let base_dir = manifest.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
                    if let Err(e) = runtime::run_service(&name, config, &base_dir) {
                        tracing::error!("Service {} failed: {:?}", name, e);
                    }
                }
            }
        }
    }

    Ok(())
}
