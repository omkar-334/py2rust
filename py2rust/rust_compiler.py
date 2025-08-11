"""
Rust Compiler and Project Manager
=================================

Handles saving generated Rust code and compiling it into working projects.
"""

import os
import re
import subprocess
import shutil
from pathlib import Path
from typing import Dict, List, Any
import logging


class RustCompiler:
    """Handles Rust project creation, compilation, and validation."""
    
    def __init__(self):
        self.logger = logging.getLogger(__name__)
        
        # Check Rust installation
        try:
            result = subprocess.run(['cargo', '--version'], capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                self.logger.info(f"Rust toolchain: {result.stdout.strip()}")
        except (subprocess.TimeoutExpired, FileNotFoundError):
            self.logger.warning("Rust toolchain not found. Install from https://rustup.rs/")
    
    def save_rust_project(self, llm_response: str, output_dir: Path) -> Path:
        """Parse LLM response and save as a Rust project."""
        self.logger.info(f"Saving Rust project to {output_dir}")
        
        files = self._parse_llm_response(llm_response)
        if not files:
            raise ValueError("No Rust files found in LLM response")
        
        # Ensure Cargo.toml exists
        if 'Cargo.toml' not in files:
            self.logger.warning("No Cargo.toml found, creating default")
            files['Cargo.toml'] = self._create_default_cargo_toml()
        
        # Create project
        project_name = self._extract_project_name(files.get('Cargo.toml', ''))
        project_dir = output_dir / project_name
        
        if project_dir.exists():
            shutil.rmtree(project_dir)
        project_dir.mkdir(parents=True)
        
        # Save all files
        for file_path, content in files.items():
            full_path = project_dir / file_path
            full_path.parent.mkdir(parents=True, exist_ok=True)
            
            with open(full_path, 'w', encoding='utf-8') as f:
                f.write(content)
            self.logger.debug(f"Saved: {file_path}")
        
        # Ensure src directory with main.rs if no lib.rs
        src_dir = project_dir / 'src'
        src_dir.mkdir(exist_ok=True)
        
        if not (src_dir / 'lib.rs').exists() and not (src_dir / 'main.rs').exists():
            self.logger.info("Creating default main.rs")
            with open(src_dir / 'main.rs', 'w') as f:
                f.write('fn main() {\n    println!("Hello, world!");\n}\n')
        
        self.logger.info(f"Rust project saved: {project_dir}")
        return project_dir
    
    def _parse_llm_response(self, response: str) -> Dict[str, str]:
        """Parse LLM response to extract file contents."""
        files = {}
        
        # Match code blocks with file paths
        pattern = r'```(?:toml|rust|cargo)\s*\n(?:\[([^\]]+)\]\s*\n)?(.*?)```'
        matches = re.findall(pattern, response, re.DOTALL)
        
        for file_path, content in matches:
            if file_path:
                file_path = file_path.strip()
                content = content.strip()
                
                if file_path.lower() == 'cargo.toml':
                    files['Cargo.toml'] = content
                elif file_path.startswith('src/'):
                    files[file_path] = content
                elif '/' not in file_path and file_path.endswith('.rs'):
                    files[f'src/{file_path}'] = content
                else:
                    files[file_path] = content
        
        # Extract Cargo.toml without code blocks if not found
        if 'Cargo.toml' not in files:
            cargo_match = re.search(r'\[package\].*?(?=\n\n|\n```|\Z)', response, re.DOTALL)
            if cargo_match:
                files['Cargo.toml'] = cargo_match.group(0).strip()
        
        # Extract standalone Rust code if no file paths
        if not files:
            rust_blocks = re.findall(r'```rust\s*\n(.*?)```', response, re.DOTALL)
            for i, content in enumerate(rust_blocks):
                filename = 'main.rs' if i == 0 else f'module_{i}.rs'
                files[f'src/{filename}'] = content.strip()
        
        self.logger.info(f"Extracted {len(files)} files from LLM response")
        return files
    
    def _extract_project_name(self, cargo_toml: str) -> str:
        """Extract project name from Cargo.toml."""
        match = re.search(r'name\s*=\s*["\']([^"\']+)["\']', cargo_toml)
        return match.group(1) if match else 'converted_project'
    
    def _create_default_cargo_toml(self) -> str:
        """Create default Cargo.toml."""
        return '''[package]
name = "converted_project"
version = "0.1.0"
edition = "2021"

[dependencies]
'''
    
    def compile_and_test(self, project_dir: Path) -> bool:
        """Compile and test the Rust project."""
        self.logger.info(f"Compiling and testing: {project_dir}")
        
        if not (project_dir / 'Cargo.toml').exists():
            self.logger.error("Cargo.toml not found")
            return False
        
        try:
            # Check syntax
            self.logger.info("Running cargo check...")
            check_result = subprocess.run(
                ['cargo', 'check'], cwd=project_dir, capture_output=True, text=True, timeout=300
            )
            
            if check_result.returncode != 0:
                self.logger.error("Cargo check failed:")
                self.logger.error(check_result.stderr)
                self._analyze_errors(check_result.stderr)
                return False
            
            # Build
            self.logger.info("Building...")
            build_result = subprocess.run(
                ['cargo', 'build'], cwd=project_dir, capture_output=True, text=True, timeout=600
            )
            
            if build_result.returncode != 0:
                self.logger.error("Build failed:")
                self.logger.error(build_result.stderr)
                self._analyze_errors(build_result.stderr)
                return False
            
            self.logger.info("✅ Build successful!")
            
            # Run tests
            test_success = self._run_tests(project_dir)
            
            # Format and lint
            self._format_code(project_dir)
            self._lint_code(project_dir)
            
            return test_success
                
        except subprocess.TimeoutExpired:
            self.logger.error("Operation timed out")
            return False
        except Exception as e:
            self.logger.error(f"Error: {e}")
            return False
    
    def _run_tests(self, project_dir: Path) -> bool:
        """Run tests."""
        try:
            self.logger.info("Running tests...")
            test_result = subprocess.run(
                ['cargo', 'test'], cwd=project_dir, capture_output=True, text=True, timeout=300
            )
            
            if test_result.returncode == 0:
                self.logger.info("✅ All tests passed!")
                return True
            else:
                self.logger.warning("⚠️ Some tests failed")
                return False
                
        except Exception as e:
            self.logger.warning(f"Test execution error: {e}")
            return False
    
    def _format_code(self, project_dir: Path) -> None:
        """Format code with rustfmt."""
        try:
            subprocess.run(['cargo', 'fmt'], cwd=project_dir, capture_output=True, timeout=60)
            self.logger.info("Code formatted")
        except Exception:
            pass
    
    def _lint_code(self, project_dir: Path) -> None:
        """Lint with clippy."""
        try:
            clippy_result = subprocess.run(
                ['cargo', 'clippy', '--', '-D', 'warnings'],
                cwd=project_dir, capture_output=True, text=True, timeout=300
            )
            if clippy_result.returncode != 0:
                suggestions = [line for line in clippy_result.stderr.split('\n') 
                             if 'warning:' in line or 'error:' in line]
                if suggestions:
                    self.logger.info(f"Clippy suggestions: {len(suggestions)}")
        except Exception:
            pass
    
    def _analyze_errors(self, stderr: str) -> None:
        """Analyze compilation errors."""
        common_errors = {
            'cannot find crate': "Missing dependency in Cargo.toml",
            'cannot find function': "Function not defined or imported",
            'cannot find type': "Type not defined or imported",
            'mismatched types': "Type mismatch - check variable types",
            'borrow checker': "Ownership/borrowing issue - review lifetimes",
            'cannot move out': "Ownership violation - consider cloning or borrowing"
        }
        
        self.logger.info("Error analysis:")
        for error_pattern, suggestion in common_errors.items():
            if error_pattern in stderr.lower():
                self.logger.info(f"  - {error_pattern}: {suggestion}")
    
    def get_project_info(self, project_dir: Path) -> Dict[str, Any]:
        """Get information about the compiled Rust project."""
        info = {
            'name': project_dir.name,
            'path': str(project_dir),
            'cargo_toml_exists': (project_dir / 'Cargo.toml').exists(),
            'src_exists': (project_dir / 'src').exists(),
            'files': [],
            'dependencies': []
        }
        
        # List all Rust files
        if project_dir.exists():
            for rust_file in project_dir.rglob("*.rs"):
                rel_path = rust_file.relative_to(project_dir)
                info['files'].append(str(rel_path))
        
        # Parse dependencies from Cargo.toml
        cargo_toml = project_dir / 'Cargo.toml'
        if cargo_toml.exists():
            try:
                with open(cargo_toml, 'r') as f:
                    content = f.read()
                
                # Simple regex to extract dependencies
                dep_pattern = r'\[dependencies\]\s*(.*?)(?=\[|\Z)'
                match = re.search(dep_pattern, content, re.DOTALL)
                if match:
                    deps_section = match.group(1)
                    for line in deps_section.strip().split('\n'):
                        if '=' in line and not line.strip().startswith('#'):
                            dep_name = line.split('=')[0].strip()
                            if dep_name:
                                info['dependencies'].append(dep_name)
                                
            except Exception as e:
                self.logger.warning(f"Failed to parse Cargo.toml: {e}")
        
        return info
