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
    
    use std::sync::Arc;
    use wasmer_wasix::virtual_fs::host_fs::FileSystem as HostFs;
    use wasmer_wasix::runtime::task_manager::tokio::TokioTaskManager;
    use wasmer_wasix::runtime::PluggableRuntime;
    use virtual_net::host::LocalNetworking;

    let rt = Arc::new(TokioTaskManager::new(tokio::runtime::Handle::current()));
    let mut runtime = PluggableRuntime::new(rt);
    runtime.set_networking_implementation(LocalNetworking::default());
    runtime.set_engine(store.engine().clone());
    
    let fs = Arc::new(HostFs::new(tokio::runtime::Handle::current(), "/").unwrap());
    
    use wasmer_wasix::capabilities::{Capabilities, CapabilityThreadingV1};
    
    let capabilities = Capabilities {
        insecure_allow_all: true,
        threading: CapabilityThreadingV1 {
            max_threads: Some(10),
            enable_asynchronous_threading: true,
            ..Default::default()
        },
        ..Capabilities::default()
    };

    let mut wasi_env_builder = WasiEnv::builder(name)
        .runtime(Arc::new(runtime))
        .capabilities(capabilities)
        .fs(fs as Arc<dyn wasmer_wasix::virtual_fs::FileSystem + Send + Sync>);

    // Inject env vars
    for (k, v) in &config.env {
        wasi_env_builder.add_env(k, v);
    }

    // Configure networking
    if let Some(net_config) = &config.network {
        tracing::info!("Configuring network: listen on {}", net_config.listen);
        wasi_env_builder.add_env("LISTEN_ADDR", &net_config.listen);
    }

    // Mount directories
    for (host_dir, guest_dir) in &config.mounts {
        let host_path = base_dir.join(host_dir);
        std::fs::create_dir_all(&host_path)?;
        let host_path = host_path.canonicalize()?;
        tracing::info!("Mounting {} to {}", host_path.display(), guest_dir);
        wasi_env_builder = wasi_env_builder.map_dir(guest_dir, &host_path)?;
    }

    tracing::info!("Instantiating module for {}", name);
    let (instance, _wasi_env) = wasi_env_builder.instantiate(module, &mut store)?;

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
                    other => {
                        tracing::error!("Service {} returned a WASI error: {:?}", name, other);
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
