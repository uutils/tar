"""
Extract the GNU logs into a JSON file.
"""

import json
import re
import sys
from pathlib import Path

STATUS_MAP = {"ok": "PASS", "FAILED": "FAIL", "skipped": "SKIP"}

# GNU tar's Autotest log emits successful/skipped tests as:
#   4. interspersed options (options02.at:26): ok     (0m0.000s 0m0.004s)
# and failed tests in the detailed section as:
#   1. version.at:19: 1. tar version (version.at:19): FAILED (version.at:21)
AUTOTEST_RESULT_RE = re.compile(
    r"^\s*(\d+)\.\s+"
    r"(?:(?:\S+\.at:\d+):\s+)?"
    r"(?:\d+\.\s+)?"
    r"(.+?)\s+\(\S+\.at:\d+\):\s+"
    r"(ok|FAILED|skipped)\b"
)

# Older Autotest output uses a colon after the test number.
AUTOTEST_COLON_RESULT_RE = re.compile(
    r"^\s*(\d+):\s+(.*?)\s+(ok|FAILED|skipped)(?:\s+\(.*\))?$"
)

AUTOMAKE_RESULT_RE = re.compile(
    r"(PASS|FAIL|SKIP|ERROR) [^ ]+ \(exit status: \d+\)$"
)


def parse_autotest_line(line):
    match = AUTOTEST_RESULT_RE.match(line)
    if not match:
        match = AUTOTEST_COLON_RESULT_RE.match(line)
    if not match:
        return None

    num, name, status = match.groups()
    return f"test {num}", name.strip(), STATUS_MAP.get(status, status)


def parse_autotest_log(path):
    results = {}
    with open(path, "r", errors="ignore") as f:
        for line in f:
            parsed = parse_autotest_line(line)
            if parsed:
                test_group, name, status = parsed
                results[test_group] = {name: status}
    return results


def parse_automake_log(path):
    with open(path, "r", errors="ignore") as f:
        # Only read the end of the file where the result is usually located.
        f.seek(0, 2)
        size = f.tell()
        f.seek(max(0, size - 1000), 0)
        content = f.read()
        result = AUTOMAKE_RESULT_RE.search(content)
        if result:
            return result.group(1)
    return None


def extract_results(test_dir):
    out = {}

    # Test all the logs from the test execution.
    for filepath in test_dir.glob("**/*.log"):
        path = Path(filepath)
        if path == test_dir / "testsuite.log":
            try:
                out.update(parse_autotest_log(path))
            except Exception as e:
                print(f"Error processing testsuite.log {path}: {e}", file=sys.stderr)
            continue

        try:
            result = parse_automake_log(path)
        except Exception as e:
            print(f"Error processing file {path}: {e}", file=sys.stderr)
            continue
        if not result:
            continue

        # Handle individual Automake-style .log files.
        current = out
        for key in path.parent.relative_to(test_dir).parts:
            if key not in current:
                current[key] = {}
            current = current[key]
        current[path.name] = result

    return out


def main():
    if len(sys.argv) != 2:
        print("Usage: python gnu-json-result.py <gnu_test_directory>")
        return 1

    test_dir = Path(sys.argv[1])
    if not test_dir.is_dir():
        print(f"Directory {test_dir} does not exist.", file=sys.stderr)
        return 1

    out = extract_results(test_dir)
    if not out:
        print(f"No GNU test results found in {test_dir}", file=sys.stderr)
        return 1

    print(json.dumps(out, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
