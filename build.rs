use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from("src/api/generated");
    // Ensure directory exists
    std::fs::create_dir_all(&out_dir).unwrap();

    tonic_build::configure()
        .out_dir(&out_dir)
        .compile_protos(&["proto/common/version.proto"], &["proto"])
        .unwrap();

    // Add SPDX header to generated files
    let generated_file = out_dir.join("version.rs");
    if generated_file.exists() {
        let content = std::fs::read_to_string(&generated_file).unwrap();
        let new_content = format!(
            "// SPDX-License-Identifier: MIT OR Apache-2.0\n// DO NOT EDIT\n{}",
            content
        );
        std::fs::write(generated_file, new_content).unwrap();
    }

    // Rerun if proto changes
    println!("cargo:rerun-if-changed=proto/common/version.proto");
}
