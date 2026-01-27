// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;

fn main() {
    // Skip code generation on docs.rs - use pre-generated files
    // docs.rs has a read-only filesystem
    if std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let out_dir = PathBuf::from("src/api/generated");
    // Ensure directory exists
    std::fs::create_dir_all(&out_dir).unwrap();

    tonic_build::configure()
        .out_dir(&out_dir)
        .build_server(true)
        .compile_protos(
            &[
                "proto/common/version.proto",
                "proto/common/common.proto",
                "proto/machine/machine.proto",
            ],
            &["proto"],
        )
        .unwrap();

    // Add SPDX header to generated files
    for file_name in &["version.rs", "common.rs", "machine.rs"] {
        let generated_file = out_dir.join(file_name);
        if generated_file.exists() {
            let content = std::fs::read_to_string(&generated_file).unwrap();
            if !content.starts_with("// SPDX-License-Identifier") {
                let new_content = format!(
                    "// SPDX-License-Identifier: MIT OR Apache-2.0\n// DO NOT EDIT\n{}",
                    content
                );
                std::fs::write(&generated_file, new_content).unwrap();
            }

            // Format the generated file
            let _ = std::process::Command::new("rustfmt")
                .arg("--edition")
                .arg("2021")
                .arg(&generated_file)
                .status();
        }
    }

    // Rerun if proto changes
    println!("cargo:rerun-if-changed=proto/common/version.proto");
    println!("cargo:rerun-if-changed=proto/common/common.proto");
    println!("cargo:rerun-if-changed=proto/machine/machine.proto");
}
