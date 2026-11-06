"""CLI validation tests: --help terminates before renderer/window initialization."""
import subprocess
import unittest
from pathlib import Path

BINARY = Path(__file__).resolve().parents[1] / 'target/release/raytracer-window'


class CliValidationTests(unittest.TestCase):
    def run_cli(self, *arguments):
        return subprocess.run([str(BINARY), *arguments], capture_output=True, text=True, timeout=10)

    def test_help_is_headless(self):
        result = self.run_cli('--help')
        self.assertEqual(result.returncode, 0)
        self.assertIn('Usage:', result.stdout)
        self.assertNotIn('Initializing raylib', result.stdout)

    def test_unsigned_seed_boundaries(self):
        for seed in ('0', '42', '18446744073709551615'):
            with self.subTest(seed=seed):
                self.assertEqual(self.run_cli('--seed', seed, '--help').returncode, 0)

    def test_malformed_seeds_are_rejected_before_help_or_window(self):
        for seed in ('', '-1', ' -1', '\t-1', '+1', '1 ', '1x', '18446744073709551616'):
            with self.subTest(seed=seed):
                result = self.run_cli('--seed', seed, '--help')
                self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                self.assertNotIn('Initializing raylib', result.stdout)

    def test_positive_integer_options_reject_malformed_values(self):
        for option in ('--width', '--height', '--samples', '--frames'):
            for value in ('0', '-1', ' 1', '+1', '1x', '2147483648'):
                with self.subTest(option=option, value=value):
                    self.assertEqual(self.run_cli(option, value, '--help').returncode, 2)

    def test_missing_values(self):
        for option in ('--width', '--height', '--samples', '--frames', '--seed', '--output', '--screenshot'):
            with self.subTest(option=option):
                self.assertEqual(self.run_cli(option).returncode, 2)

    def test_unknown_option(self):
        self.assertEqual(self.run_cli('--unknown', 'value', '--help').returncode, 2)

    def test_path_options_do_not_consume_flags_or_empty_values(self):
        for option in ('--output', '--screenshot'):
            for value in ('', '--static', '--help'):
                with self.subTest(option=option, value=value):
                    self.assertEqual(self.run_cli(option, value, '--help').returncode, 2)


if __name__ == '__main__':
    unittest.main()
