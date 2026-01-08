#!/usr/bin/env python3
"""
Remove all PostgreSQL/sqlx code from lib.rs
"""

import re

def remove_postgres_from_lib():
    with open('ouro_dag/src/lib.rs', 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.split('\n')

    # Track which lines to keep
    keep_lines = []
    skip_until_line = -1
    in_migration_function = False
    in_sql_split_function = False
    brace_count = 0

    i = 0
    while i < len(lines):
        line = lines[i]

        # Skip sqlx imports
        if line.strip().startswith('use sqlx'):
            i += 1
            continue

        # Detect start of split_sql_statements function
        if 'pub fn split_sql_statements' in line:
            in_sql_split_function = True
            brace_count = 0
            i += 1
            continue

        # Detect start of run_migrations function
        if 'pub async fn run_migrations' in line:
            in_migration_function = True
            brace_count = 0
            # Also skip the doc comment before it
            if keep_lines and keep_lines[-1].strip().startswith('///'):
                # Remove previous doc comment lines
                while keep_lines and keep_lines[-1].strip().startswith('///'):
                    keep_lines.pop()
            i += 1
            continue

        # Track braces in functions we're removing
        if in_sql_split_function or in_migration_function:
            brace_count += line.count('{') - line.count('}')
            i += 1
            if brace_count <= 0 and ('{' in line or '}' in line):
                in_sql_split_function = False
                in_migration_function = False
            continue

        # Keep this line
        keep_lines.append(line)
        i += 1

    # Join and write back
    new_content = '\n'.join(keep_lines)

    with open('ouro_dag/src/lib.rs', 'w', encoding='utf-8') as f:
        f.write(new_content)

    print(f"Removed PostgreSQL code from lib.rs")
    print(f"Original lines: {len(lines)}")
    print(f"New lines: {len(keep_lines)}")
    print(f"Removed: {len(lines) - len(keep_lines)} lines")

if __name__ == '__main__':
    remove_postgres_from_lib()
