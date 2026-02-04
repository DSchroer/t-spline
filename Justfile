# Justfile to manage project tasks

ci: build build-no-std check-licenses
    cargo test --features fixed --locked
    cargo clippy
    cargo fmt --check

example NAME:
    cargo run --example {{NAME}} > {{NAME}}.obj

examples: (example "fixed") (example "t_junction") (example "simple")

build:
    cargo build --locked

build-no-std:
    (cd t_spline && cargo build --target x86_64-unknown-none --locked)



# Validate that all source files contain the correct license header
check-licenses:
    #!/usr/bin/env python3
    import os
    import sys

    header_file = "license_header.txt"
    try:
        with open(header_file, "r") as f:
            expected_header = f.read()
    except FileNotFoundError:
        print(f"Error: {header_file} not found.")
        sys.exit(1)

    failed = False
    # Walk the directory tree
    for root, dirs, files in os.walk("."):
        # Skip hidden directories and build artifacts
        dirs[:] = [d for d in dirs if not d.startswith('.') and d != 'target']
        
        for file in files:
            if file.endswith(".rs"):
                filepath = os.path.join(root, file)
                try:
                    with open(filepath, "r") as f:
                        # Read enough characters to match the header length
                        content = f.read(len(expected_header))
                        if content != expected_header:
                            print(f"Header mismatch: {filepath}")
                            failed = True
                except Exception as e:
                    print(f"Could not read {filepath}: {e}")
                    failed = True
    
    if failed:
        sys.exit(1)
    else:
        print("All files have valid license headers.")
