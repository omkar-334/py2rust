# Python to Rust Converter

Minimal tool that converts Python repositories to Rust using Google Gemini AI.

## Features

- 🤖 **Gemini-Powered**: Uses Google Gemini 2.5 Pro for intelligent code conversion
- 🦀 **Complete Projects**: Generates compilable Rust code with proper Cargo.toml
- ✅ **Auto Testing**: Compiles, tests, formats, and lints generated code
- 📁 **Simple Ingestion**: Finds all Python files (excludes test/docs folders)
- ⚙️ **GitHub Workflow**: Automated conversion via GitHub Actions

## Quick Start

### Prerequisites

1. **Python 3.8+**
2. **Rust toolchain**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. **Gemini API key**: Get from [Google AI Studio](https://aistudio.google.com/)

### Installation

```bash
git clone <this-repo>
cd py2rust
pip install -r requirements.txt
export GEMINI_API_KEY="your-api-key"
```

### Usage

```bash
# Convert a Python repository
python main.py https://github.com/user/python-repo

# Convert local project
python main.py ./my-python-project

# Organized output (for workflows)
python main.py https://github.com/user/repo --projects-dir ./projects

# Quick test (skip compilation)
python main.py ./python-code --skip-compilation --dry-run
```

### Options

```bash
Options:
  --output-dir DIR        Output directory (default: ./rust_output)
  --projects-dir DIR      Organize in projects structure
  --verbose, -v           Enable verbose logging
  --dry-run               Generate prompt only
  --skip-compilation      Skip compilation step
```

## GitHub Workflow Setup

### 1. Repository Setup

1. Fork this repository
2. Enable GitHub Actions in Settings
3. Create a `projects/` directory for converted projects

### 2. Add API Key

1. Go to **Settings → Secrets and variables → Actions**
2. Add secret: Name `GEMINI_API_KEY`, Value: your Gemini API key

### 3. Convert Repositories

1. Go to **Actions** tab
2. Select **"Convert Python to Rust"** workflow
3. Click **"Run workflow"**
4. Enter Python repository URL (e.g., `https://github.com/user/python-project`)
5. Click **"Run workflow"**

### Output Structure

```
your-repo/
├── py2rust/             # Converter source code
├── projects/            # Converted projects
│   ├── python-repo-1/
│   │   ├── python/             # Original Python source
│   │   ├── rust/               # Converted Rust project
│   │   │   ├── Cargo.toml
│   │   │   ├── src/
│   │   │   └── target/         # Compiled binaries
│   │   └── conversion_metadata.txt
│   └── python-repo-2/
└── .github/workflows/   # GitHub Actions
```

## How It Works

1. **Clones** Python repository (GitHub workflow handles this)
2. **Ingests** Python files from local directory
3. **Generates** comprehensive prompt with all Python code
4. **Converts** using Gemini 2.5 Pro with Rust best practices
5. **Compiles** with `cargo build` and runs tests
6. **Formats** with `cargo fmt` and lints with `cargo clippy`
7. **Organizes** output with both Python source and Rust project

## Local Development

```bash
# Simple conversion (local directory)
python main.py ./python-project

# Workflow-style organization
python main.py ./python-project --projects-dir ./my-projects --verbose

# Fast testing
python main.py ./python-project --skip-compilation --dry-run
```

## Troubleshooting

**"GEMINI_API_KEY not set"** → Set environment variable with your API key

**"Rust toolchain not found"** → Install Rust from https://rustup.rs/

**"Compilation failed"** → Normal for complex projects, review generated code manually

**"No Python files found"** → Check repository has .py files outside test/docs folders

## Why Gemini 2.5 Pro?

- ✅ Latest model with best code generation  
- ✅ Massive context window (1M+ tokens)
- ✅ High output limit (65K tokens)
- ✅ System instructions for better guidance
- ✅ Advanced thinking mode for higher quality
- ✅ Structured outputs and advanced capabilities

## Project Structure

```
py2rust/
├── main.py              # Main conversion workflow
├── repo_ingester.py     # Python file discovery
├── prompt_generator.py  # Gemini prompt creation
├── rust_compiler.py     # Rust compilation & testing
├── requirements.txt     # Dependencies
└── README.md           # This file
```

The tool provides a **clean, automated workflow** for converting Python projects to Rust with minimal setup and maximum automation.
