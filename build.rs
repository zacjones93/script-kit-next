// Build script for script-kit-gpui
// This ensures the binary is rebuilt when the SDK changes and
// copies the SDK to ~/.kit/lib/ for immediate use during development

use std::fs;
use std::path::PathBuf;

fn main() {
    // Ensure rebuild when SDK changes
    println!("cargo:rerun-if-changed=scripts/kit-sdk.ts");
    
    // Copy SDK to ~/.kit/lib/ during build for dev workflow
    // This ensures hot-reload picks up SDK changes immediately
    if let Some(home) = dirs::home_dir() {
        let kit_lib = home.join(".kit/lib");
        let sdk_dest = kit_lib.join("kit-sdk.ts");
        let sdk_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/kit-sdk.ts");
        
        // Create directory if needed
        if !kit_lib.exists() {
            if let Err(e) = fs::create_dir_all(&kit_lib) {
                println!("cargo:warning=Failed to create ~/.kit/lib/: {}", e);
                return;
            }
        }
        
        // Copy SDK
        match fs::copy(&sdk_src, &sdk_dest) {
            Ok(bytes) => {
                println!("cargo:warning=Copied SDK to {} ({} bytes)", sdk_dest.display(), bytes);
            }
            Err(e) => {
                println!("cargo:warning=Failed to copy SDK: {}", e);
            }
        }
    }
}
