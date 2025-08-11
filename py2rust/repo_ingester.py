"""
Python Repository Ingester
==========================

Simple ingestion of Python files from local directories.
"""

from pathlib import Path
from typing import List, NamedTuple
import logging


class PythonFile(NamedTuple):
    """A Python file with its content."""
    path: str
    content: str


def ingest_python_repo(repo_path: str) -> List[PythonFile]:
    """
    Ingest Python files from local directory, excluding test/ and docs/ folders.
    
    Args:
        repo_path: Local directory path
        
    Returns:
        List of PythonFile objects
    """
    logger = logging.getLogger(__name__)
    local_path = Path(repo_path).resolve()
    
    if not local_path.exists():
        raise FileNotFoundError(f"Path does not exist: {local_path}")
    
    python_files = []
    
    # Find all .py files, excluding test and docs folders
    for py_file in local_path.rglob("*.py"):
        # Skip test and docs folders
        if any(part.lower() in ('test', 'tests', 'doc', 'docs') for part in py_file.parts):
            continue
        
        try:
            with open(py_file, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
            
            relative_path = str(py_file.relative_to(local_path))
            python_files.append(PythonFile(relative_path, content))
            logger.debug(f"Ingested: {relative_path}")
            
        except Exception as e:
            logger.warning(f"Failed to read {py_file}: {e}")
    
    logger.info(f"Found {len(python_files)} Python files")
    return python_files
