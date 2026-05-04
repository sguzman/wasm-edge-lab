use wasmer_wasix::WasiEnv;
use wasmer_wasix::virtual_fs::FileSystem;
use wasmer_wasix::virtual_fs::host_fs::FileSystem as HostFs;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let mut store = wasmer::Store::default();
    let module = wasmer::Module::new(&store, "(module)").unwrap();

    println!("Testing map_dir with explicit root HostFs...");
    let fs = Arc::new(HostFs::new(tokio::runtime::Handle::current(), "/").unwrap());
    
    let mut builder = WasiEnv::builder("test").fs(fs as Arc<dyn FileSystem + Send + Sync>);
    builder = builder.map_dir("/data", "/win/linux/Code/rust/wasm-edge-lab/examples/data").unwrap();
    match builder.instantiate(module.clone(), &mut store) {
        Ok(_) => println!("map_dir absolute Instantiated successfully!"),
        Err(e) => println!("map_dir absolute Error: {:?}", e),
    }
}
