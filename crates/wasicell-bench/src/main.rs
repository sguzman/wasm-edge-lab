use std::time::Instant;
use std::process::Command;
use std::fs;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    println!("# wasicell Benchmark Report\n");

    let native_path = "target/debug/file-worker";
    let wasm_path = "target/wasm32-wasmer-wasi/debug/file-worker.wasm";

    // 1. Package Size
    let native_size = fs::metadata(native_path)?.len();
    let wasm_size = fs::metadata(wasm_path)?.len();
    
    // Get docker image size (approximate)
    let docker_output = Command::new("docker")
        .args(["images", "file-worker-bench", "--format", "{{.Size}}"])
        .output()?;
    let docker_size_str = String::from_utf8_lossy(&docker_output.stdout).trim().to_string();

    println!("## 1. Package Size");
    println!("- Native Binary: {:.2} MB", native_size as f64 / 1_048_576.0);
    println!("- Docker Image:  {}", docker_size_str);
    println!("- Wasm Module:   {:.2} MB", wasm_size as f64 / 1_048_576.0);
    println!("- Wasm vs Native: {:.1}x smaller", native_size as f64 / wasm_size as f64);
    println!();

    // 2. Cold Start (Time to exit for file-worker)
    println!("## 2. Cold Start Execution (file-worker)");
    
    // Create dummy input
    let data_dir = Path::new("examples/data");
    fs::create_dir_all(data_dir)?;
    fs::write(data_dir.join("input.txt"), "Hello Wasmer edge container lab!")?;

    // Native run
    let native_abs = fs::canonicalize(native_path)?;
    let start = Instant::now();
    let _ = Command::new(native_abs)
        .current_dir("examples") // Run from examples so it finds ./data
        .env("RUST_LOG", "error")
        .output()?;
    let native_duration = start.elapsed();
    println!("- Native execution:  {:?}", native_duration);

    // Docker run
    let start = Instant::now();
    let _ = Command::new("docker")
        .arg("run")
        .arg("--rm")
        .arg("-v")
        .arg(format!("{}:/data", fs::canonicalize("examples/data")?.display()))
        .arg("file-worker-bench")
        .output()?;
    let docker_duration = start.elapsed();
    println!("- Docker execution:  {:?}", docker_duration);

    // Wasicell run
    let start = Instant::now();
    let _ = Command::new("target/debug/wasicell-host")
        .args(["run", "examples/app.toml", "file_worker"])
        .output()?;
    let wasm_duration = start.elapsed();
    println!("- Wasicell (Wasm):   {:?}", wasm_duration);
    println!();

    // 3. Resource Isolation (Qualitative)
    println!("## 3. Resource Isolation (Qualitative)");
    println!("- Native: No sandbox, full access to host env/FS.");
    println!("- Docker: Kernel-level isolation (cgroups/namespaces).");
    println!("- Wasicell: Wasm-level isolation (capability-based), WASIX sandboxed.");

    Ok(())
}
