use std::process::Command;

fn main() {
    // Set build date
    let build_date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
    
    // Set Rust version
    let rustc_version = Command::new("rustc")
        .arg("--version")
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=RUSTC_VERSION={}", rustc_version);
    
    // Rerun if Cargo.toml changes
    println!("cargo:rerun-if-changed=Cargo.toml");
}