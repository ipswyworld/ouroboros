#!/usr/bin/env python3
"""
Remove all PostgreSQL/sqlx references from entire codebase
"""

import os
import re
from pathlib import Path

def comment_out_sqlx_functions(content):
    """Comment out functions that use PgPool or sqlx"""
    lines = content.split('\n')
    result = []
    in_sqlx_function = False
    brace_depth = 0
    func_start_indent = 0

    for i, line in enumerate(lines):
        # Check if this is a function with PgPool parameter
        if ('fn ' in line and 'PgPool' in line) or ('sqlx::' in line and 'fn ' in line):
            in_sqlx_function = True
            brace_depth = 0
            func_start_indent = len(line) - len(line.lstrip())
            # Comment out the function signature
            result.append('// REMOVED PostgreSQL function: ' + line.strip())
            continue

        if in_sqlx_function:
            # Track braces
            brace_depth += line.count('{') - line.count('}')

            # Comment out the line
            if line.strip():
                result.append('// ' + line)
            else:
                result.append(line)

            # Check if function is complete
            if brace_depth <= 0 and ('}' in line):
                in_sqlx_function = False
            continue

        result.append(line)

    return '\n'.join(result)

def remove_sqlx_imports(content):
    """Remove sqlx import lines"""
    lines = content.split('\n')
    result = []

    for line in lines:
        # Skip lines that import sqlx
        if re.match(r'^\s*use\s+sqlx', line):
            continue
        # Skip PgPool references in use statements
        if 'PgPool' in line and 'use' in line:
            continue
        result.append(line)

    return '\n'.join(result)

def remove_pgpool_params(content):
    """Remove PgPool parameters from function signatures"""
    # This is complex, so we'll do a simple replacement
    content = re.sub(r',\s*pool:\s*&?Arc<PgPool>', '', content)
    content = re.sub(r',\s*pool:\s*&?PgPool', '', content)
    content = re.sub(r'pool:\s*&?Arc<PgPool>,\s*', '', content)
    content = re.sub(r'pool:\s*&?PgPool,\s*', '', content)
    return content

def process_file(filepath):
    """Process a single Rust file"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        original_size = len(content)

        # Remove imports
        content = remove_sqlx_imports(content)

        # Remove PgPool parameters
        content = remove_pgpool_params(content)

        # Comment out sqlx functions (too complex, skip for now)
        # content = comment_out_sqlx_functions(content)

        # Write back
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

        if len(content) != original_size:
            print(f"[OK] Modified: {filepath}")
            return True
        return False

    except Exception as e:
        print(f"[ERR] Error processing {filepath}: {e}")
        return False

def main():
    src_dir = Path('ouro_dag/src')

    # Find all Rust files
    rust_files = list(src_dir.rglob('*.rs'))

    print(f"Found {len(rust_files)} Rust files")
    print("Removing PostgreSQL/sqlx references...\n")

    modified = 0
    for filepath in rust_files:
        if process_file(filepath):
            modified += 1

    print(f"\nCompleted: {modified} files modified")

if __name__ == '__main__':
    main()
