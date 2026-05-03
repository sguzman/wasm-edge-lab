use std::fs;
use std::env;
use std::io::Write;

fn main() {
    println!("Starting file-worker...");

    // Check environment variables
    if let Ok(log_level) = env::var("RUST_LOG") {
        println!("RUST_LOG is set to: {}", log_level);
    }

    // Try reading the file from the mounted directory
    let input_path = "/data/input.txt";
    let output_path = "/data/output.txt";

    match fs::read_to_string(input_path) {
        Ok(content) => {
            println!("Successfully read from {}: {}", input_path, content);
            
            // Process the content (reverse it for example)
            let processed: String = content.chars().rev().collect();
            
            match fs::File::create(output_path) {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "Processed output: {}", processed) {
                        eprintln!("Failed to write to {}: {}", output_path, e);
                    } else {
                        println!("Successfully wrote to {}", output_path);
                    }
                }
                Err(e) => eprintln!("Failed to create {}: {}", output_path, e),
            }
        }
        Err(e) => {
            eprintln!("Failed to read from {}: {}", input_path, e);
        }
    }

    println!("file-worker completed.");
}
