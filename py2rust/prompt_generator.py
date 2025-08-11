"""
Prompt Generator for Python to Rust Conversion
==============================================

Generates conversion prompts for Gemini API.
"""

from typing import List
from repo_ingester import PythonFile


SYSTEM_PROMPT = """You are an expert Rust programmer converting Python code to idiomatic Rust.

CONVERSION GUIDELINES:
======================

1. RUST BEST PRACTICES:
   - Use Rust's ownership system correctly
   - Prefer &str over String when possible
   - Use Result<T, E> for error handling with ? operator
   - Use Option<T> for nullable values
   - Follow snake_case naming conventions

2. MEMORY MANAGEMENT:
   - Replace Python's garbage collection with ownership
   - Use references (&) when borrowing is sufficient
   - Use Box<T> for heap allocation when needed

3. DATA STRUCTURES:
   - Convert Python lists to Vec<T>
   - Convert Python dicts to HashMap<K, V>
   - Convert Python sets to HashSet<T>
   - Convert Python tuples to Rust tuples or structs

4. ERROR HANDLING:
   - Replace Python exceptions with Result<T, E>
   - Use anyhow crate for error handling convenience
   - Propagate errors with ? operator

5. DEPENDENCIES:
   - Use only well-tested, stable Rust crates with CORRECT features:
     * serde = { version = "1.0", features = ["derive"] }
     * serde_json = "1.0" (separate crate for JSON)
     * reqwest = { version = "0.11", features = ["json"] }
     * clap = { version = "4.0", features = ["derive"] }
     * tokio = { version = "1.0", features = ["full"] }
     * anyhow = "1.0"
     * rand = "0.8" (NOT "random")
   - For GUI: Use simple libraries like eframe/egui = "0.20"
   - NEVER use non-existent features like "fltk-theme", "macros", "gzip"
   - ALWAYS verify features exist before using them

6. PROJECT STRUCTURE:
   - Create proper Cargo.toml with dependencies
   - Organize code into modules
   - Include documentation comments (///)

OUTPUT FORMAT:
==============
Provide complete Rust project:

```toml
[Cargo.toml]
[package]
name = "converted_project"
version = "0.1.0"
edition = "2021"

[dependencies]
# List dependencies with versions
```

```rust
[src/main.rs]
// Main Rust code
```

```rust
[src/lib.rs]
// Library code if needed
```

Include ALL necessary files for a complete, compilable Rust project."""


def generate_conversion_prompt(python_files: List[PythonFile]) -> str:
    """Generate conversion prompt for Gemini (system prompt is handled separately)."""
    if not python_files:
        raise ValueError("No Python files provided")
    
    prompt_parts = [
        "=" * 80,
        "PYTHON PROJECT TO CONVERT:",
        "=" * 80,
        "",
        f"Total files: {len(python_files)}",
        "",
    ]
    
    # Add each Python file
    for py_file in python_files:
        prompt_parts.extend([
            f"FILE: {py_file.path}",
            "-" * 50,
            "```python",
            py_file.content,
            "```",
            ""
        ])
    
    prompt_parts.extend([
        "=" * 80,
        "CONVERSION REQUEST:",
        "=" * 80,
        "",
        "Convert this Python project to a complete Rust project.",
        "Ensure all functionality is preserved with idiomatic Rust code.",
        "Include proper Cargo.toml with necessary dependencies.",
        "Make the code compile and run successfully."
    ])
    
    return "\n".join(prompt_parts)
