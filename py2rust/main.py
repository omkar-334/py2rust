#!/usr/bin/env python3
"""
Python to Rust Code Converter
=============================

Minimal tool that converts Python repositories to Rust using Gemini.

Usage: python main.py <repo_url_or_path> [--projects-dir DIR] [--verbose]
"""

import os
import sys
import argparse
import logging
import shutil
from pathlib import Path

from google import genai
from google.genai import types

from repo_ingester import ingest_python_repo
from prompt_generator import generate_conversion_prompt, SYSTEM_PROMPT
from rust_compiler import RustCompiler


def setup_logging(verbose: bool = False) -> None:
    """Configure logging for the application."""
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        handlers=[
            logging.StreamHandler(sys.stdout),
            logging.FileHandler('conversion.log')
        ]
    )


def convert_with_gemini(prompt: str) -> str:
    """Convert Python code to Rust using Gemini."""
    api_key = os.getenv('GEMINI_API_KEY')
    if not api_key:
        raise ValueError("GEMINI_API_KEY environment variable not set")
    
    # Initialize the new Gemini client
    client = genai.Client(api_key=api_key)
    
    # Configure generation parameters with system instructions
    config = types.GenerateContentConfig(
        system_instruction=SYSTEM_PROMPT,
        temperature=0.1,
        top_p=0.95,
        max_output_tokens=65536,
        thinking_config=types.ThinkingConfig(thinking_budget=0)  # Disable thinking for faster response
    )
    
    response = client.models.generate_content(
        model="gemini-2.5-pro",
        contents=prompt,
        config=config
    )
    
    if not response.text:
        raise RuntimeError("Empty response from Gemini API")
    
    return response.text


def extract_repo_name(repo_path: str) -> str:
    """Extract repository name from local path."""
    return Path(repo_path).name


def organize_project(rust_project_path: Path, target_dir: Path, repo_name: str, python_dir: Path = None) -> Path:
    """Organize converted project into structured directory."""
    organized_path = target_dir / repo_name
    organized_path.mkdir(parents=True, exist_ok=True)
    
    # Copy Python source
    if python_dir and python_dir.exists():
        if (organized_path / "python").exists():
            shutil.rmtree(organized_path / "python")
        shutil.copytree(python_dir, organized_path / "python")
    
    # Copy Rust project
    if rust_project_path.exists():
        if (organized_path / "rust").exists():
            shutil.rmtree(organized_path / "rust")
        shutil.copytree(rust_project_path, organized_path / "rust")
    
    # Create metadata file
    metadata_file = organized_path / "conversion_metadata.txt"
    with open(metadata_file, 'w') as f:
        f.write(f"Original Repository: {repo_name}\n")
        f.write(f"Conversion Date: {Path().stat().st_mtime}\n")
        f.write(f"Python Source: ./python/\n")
        f.write(f"Rust Project: ./rust/\n")
        f.write(f"Build Status: {'✅ Success' if (rust_project_path / 'target').exists() else '❌ Failed'}\n")
    
    return organized_path


def main():
    """Main workflow."""
    parser = argparse.ArgumentParser(description="Convert Python repositories to Rust using Gemini")
    parser.add_argument("repo_path", help="Python repository local path")
    parser.add_argument("--output-dir", default="./rust_output", help="Output directory (default: ./rust_output)")
    parser.add_argument("--projects-dir", help="Organize output in projects structure")
    parser.add_argument("--verbose", "-v", action="store_true", help="Enable verbose logging")
    parser.add_argument("--dry-run", action="store_true", help="Generate prompt only, don't call Gemini")
    parser.add_argument("--skip-compilation", action="store_true", help="Skip Rust compilation step")
    parser.add_argument("--keep-temp", action="store_true", help="Keep temporary files after conversion")
    
    args = parser.parse_args()
    setup_logging(args.verbose)
    logger = logging.getLogger(__name__)
    
    try:
        repo_name = extract_repo_name(args.repo_path)
        use_projects_structure = args.projects_dir is not None
        output_path = Path("./temp_conversion") if use_projects_structure else Path(args.output_dir)
        
        logger.info(f"Converting {args.repo_path} -> {repo_name}")
        
        # Ingest Python files
        python_files = ingest_python_repo(args.repo_path)
        logger.info(f"Found {len(python_files)} Python files")
        
        if not python_files:
            raise ValueError("No Python files found")
        
        # Generate prompt
        prompt = generate_conversion_prompt(python_files)
        
        if args.dry_run:
            print("=== GENERATED PROMPT ===")
            print(prompt)
            return
        
        # Convert with Gemini
        logger.info("Converting to Rust...")
        rust_code = convert_with_gemini(prompt)
        
        # Save and compile
        output_path.mkdir(parents=True, exist_ok=True)
        rust_compiler = RustCompiler()
        rust_project_path = rust_compiler.save_rust_project(rust_code, output_path)
        
        compilation_success = True
        if not args.skip_compilation:
            logger.info("Compiling and testing...")
            compilation_success = rust_compiler.compile_and_test(rust_project_path)
        
        # Organize if needed
        if use_projects_structure:
            python_source_dir = Path(args.repo_path)
            final_location = organize_project(rust_project_path, Path(args.projects_dir), repo_name, python_source_dir)
            # Cleanup temporary files
            if not args.keep_temp:
                shutil.rmtree(output_path, ignore_errors=True)
            final_location = final_location / "rust"
        else:
            final_location = rust_project_path
        
        logger.info(f"{'✅ Success' if compilation_success else '⚠️ Issues'}: {final_location}")
        sys.exit(0 if compilation_success else 1)
        
    except Exception as e:
        logger.error(f"Failed: {e}")
        if args.verbose:
            logger.exception("Full traceback:")
        sys.exit(1)


if __name__ == "__main__":
    main()
