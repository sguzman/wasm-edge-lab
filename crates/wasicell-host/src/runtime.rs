use std::path::Path;
use wasmer::{Module, Store};
use wasmer_wasix::{WasiEnv, WasiError};
use wasicell_common::ServiceConfig;
use anyhow::Context;

pub fn run_service(name: &str, config: &ServiceConfig, base_dir: &Path) -> anyhow::Result<()> {
    tracing::info!("Initializing Wasmer store for {}", name);
    let mut store = Store::default();

    // Resolve module path relative to manifest
    let module_path = base_dir.join(&config.module);
    tracing::info!("Loading module from: {}", module_path.display());

    let wasm_bytes = std::fs::read(&module_path)
        .with_context(|| format!("Failed to read module at {}", module_path.display()))?;

    let module = Module::new(&store, wasm_bytes)
        .context("Failed to compile WASM module")?;

    tracing::info!("Configuring WASIX environment for {}", name);
    
    let mut wasi_env_builder = WasiEnv::builder(name);

    // Inject env vars
    for (k, v) in &config.env {
        wasi_env_builder.add_env(k, v);
    }

    // Mount directories
    for (host_dir, guest_dir) in &config.mounts {
        let host_path = base_dir.join(host_dir);
        // Ensure host path exists
        std::fs::create_dir_all(&host_path)?;
        tracing::info!("Mounting {} to {}", host_path.display(), guest_dir);
        wasi_env_builder.map_dir(guest_dir, host_path)?;
    }

    let mut wasi_env = wasi_env_builder.build()?;
    
    let import_object = wasi_env.import_object(&mut store, &module)?;

    let instance = wasmer::Instance::new(&mut store, &module, &import_object)?;

    // Initialize WASI environment for the instance
    wasi_env.initialize(&mut store, instance.clone())?;

    // Find the entrypoint. By default, it's `_start`.
    let start_func = instance.exports.get_function("_start")?;

    tracing::info!("Running {}", name);
    match start_func.call(&mut store, &[]) {
        Ok(_) => {
            tracing::info!("Service {} exited successfully.", name);
        }
        Err(e) => {
            if let Some(wasi_err) = e.downcast_ref::<WasiError>() {
                match wasi_err {
                    WasiError::Exit(code) => {
                        tracing::info!("Service {} exited with code: {}", name, code);
                    }
                    WasiError::UnknownWasiVersion => {
                        tracing::error!("Unknown WASI version for service {}", name);
                    }
                }
            } else {
                tracing::error!("Service {} crashed: {:?}", name, e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}
