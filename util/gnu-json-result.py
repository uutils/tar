"""
Extract the GNU logs into a JSON file.
"""

import json
import re
import sys
from pathlib import Path

out = {}

if len(sys.argv) != 2:
    print("Usage: python gnu-json-result.py <gnu_test_directory>")
    sys.exit(1)

test_dir = Path(sys.argv[1])
if not test_dir.is_dir():
    print(f"Directory {test_dir} does not exist.")
    sys.exit(1)

# Test all the logs from the test execution
for filepath in test_dir.glob("**/*.log"):
    path = Path(filepath)
    if path.name == "testsuite.log":
        # Handle Autotest testsuite.log
        try:
            with open(path, "r", errors="ignore") as f:
                for line in f:
                    # Look for lines like: "  1: basic functionality                ok"
                    # or " 10: ...                                     FAILED (basic.at:123)"
                    match = re.match(r"^\s*(\d+):\s+(.*?)\s+(ok|FAILED|skipped)(?:\s+\(.*\))?$", line)
                    if match:
                        num, name, status = match.groups()
                        # Map Autotest status to Automake-style status
                        status_map = {"ok": "PASS", "FAILED": "FAIL", "skipped": "SKIP"}
                        out[f"test {num}"] = {name.strip(): status_map.get(status, status)}
        except Exception as e:
            print(f"Error processing testsuite.log {path}: {e}", file=sys.stderr)
        continue

    # Handle individual Automake-style .log files
    current = out
    for key in path.parent.relative_to(test_dir).parts:
        if key not in current:
            current[key] = {}
        current = current[key]
    try:
        with open(path, "r", errors="ignore") as f:
            # Only read the end of the file where the result is usually located
            f.seek(0, 2)
            size = f.tell()
            f.seek(max(0, size - 1000), 0)
            content = f.read()
            result = re.search(
                r"(PASS|FAIL|SKIP|ERROR) [^ ]+ \(exit status: \d+\)$", content
            )
            if result:
                current[path.name] = result.group(1)
    except Exception as e:
        print(f"Error processing file {path}: {e}", file=sys.stderr)

print(json.dumps(out, indent=2, sort_keys=True))
