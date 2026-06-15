import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("analyze-gnu-results.py")
SPEC = importlib.util.spec_from_file_location("analyze_gnu_results", SCRIPT_PATH)
analyze_gnu_results = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(analyze_gnu_results)


class AnalyzeGnuResultsTests(unittest.TestCase):
    def test_single_input_can_be_written_as_aggregated_output(self):
        data = {
            "test 1": {"tar version": "FAIL"},
            "test 4": {"interspersed options": "PASS"},
            "test 11": {"--pax-option compatibility": "SKIP"},
        }

        with tempfile.TemporaryDirectory() as temp:
            temp_dir = Path(temp)
            input_file = temp_dir / "gnu-full-result.json"
            output_file = temp_dir / "aggregated-result.json"
            input_file.write_text(json.dumps(data))

            results = analyze_gnu_results.load_or_aggregate_results([input_file])
            analyze_gnu_results.write_results(output_file, results)

            self.assertEqual(json.loads(output_file.read_text()), data)

    def test_load_or_aggregate_requires_input_files(self):
        with self.assertRaisesRegex(ValueError, "no JSON input files provided"):
            analyze_gnu_results.load_or_aggregate_results([])


if __name__ == "__main__":
    unittest.main()
