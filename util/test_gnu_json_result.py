import importlib.util
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("gnu-json-result.py")
SPEC = importlib.util.spec_from_file_location("gnu_json_result", SCRIPT_PATH)
gnu_json_result = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(gnu_json_result)


class GnuJsonResultTests(unittest.TestCase):
    def test_parse_gnu_tar_autotest_results(self):
        with tempfile.TemporaryDirectory() as temp:
            test_dir = Path(temp)
            (test_dir / "testsuite.log").write_text(
                "\n".join(
                    [
                        "4. interspersed options (options02.at:26): ok     (0m0.000s 0m0.004s)",
                        "11. --pax-option compatibility (opcomp06.at:21): skipped (opcomp06.at:24)",
                        "1. version.at:19: 1. tar version (version.at:19): FAILED (version.at:21)",
                    ]
                )
            )

            self.assertEqual(
                gnu_json_result.extract_results(test_dir),
                {
                    "test 1": {"tar version": "FAIL"},
                    "test 4": {"interspersed options": "PASS"},
                    "test 11": {"--pax-option compatibility": "SKIP"},
                },
            )

    def test_parse_legacy_autotest_result(self):
        self.assertEqual(
            gnu_json_result.parse_autotest_line(
                "  1: basic functionality                ok"
            ),
            ("test 1", "basic functionality", "PASS"),
        )

    def test_ignores_nested_testsuite_log_without_result(self):
        with tempfile.TemporaryDirectory() as temp:
            test_dir = Path(temp)
            (test_dir / "testsuite.log").write_text(
                "4. interspersed options (options02.at:26): ok     (0m0.000s 0m0.004s)"
            )
            nested = test_dir / "testsuite.dir" / "004"
            nested.mkdir(parents=True)
            (nested / "testsuite.log").write_text("test detail without trailer\n")

            self.assertEqual(
                gnu_json_result.extract_results(test_dir),
                {"test 4": {"interspersed options": "PASS"}},
            )


if __name__ == "__main__":
    unittest.main()
