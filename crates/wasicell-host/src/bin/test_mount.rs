use wasmer_wasix::WasiEnv;

#[tokio::main]
async fn main() {
    let mut builder = WasiEnv::builder("test");
    let host_path = std::path::PathBuf::from("/tmp/tmp_test");
    println!("Host path: {}", host_path.display());
    builder.add_preopen_dir(host_path).unwrap();
    
    let mut store = wasmer::Store::default();
    let module = wasmer::Module::new(&store, "(module)").unwrap();
    match builder.instantiate(module, &mut store) {
        Ok(_) => println!("Instantiate success!"),
        Err(e) => println!("Instantiate error: {:?}", e),
    }
}
