#!/usr/bin/env python3
import unittest
import subprocess
import time
import sys
import os

class TestSafeRun(unittest.TestCase):
    def setUp(self):
        self.script_path = os.path.join(os.path.dirname(__file__), "safe-run.py")

    def test_successful_execution(self):
        """Test that safe-run.py successfully executes a command and captures output."""
        result = subprocess.run(
            [sys.executable, self.script_path, "5", "echo", "hello test"],
            capture_output=True,
            text=True
        )
        self.assertEqual(result.returncode, 0)
        self.assertIn("hello test", result.stdout)

    def test_timeout_execution(self):
        """Test that safe-run.py kills a command if it exceeds the timeout."""
        start = time.time()
        result = subprocess.run(
            [sys.executable, self.script_path, "1", "sleep", "10"],
            capture_output=True,
            text=True
        )
        duration = time.time() - start
        
        # Should exit with timeout code 124
        self.assertEqual(result.returncode, 124)
        # Should finish close to 1 second
        self.assertLess(duration, 3.0)

    def test_daemon_detachment(self):
        """Test that spawned background daemons do not hold the output pipes."""
        # We run a python script that spawns a background sleep daemon and exits.
        # If safe-run does not properly isolate FDs, the subprocess.run will hang waiting for stdout to close.
        # Since safe-run uses start_new_session and close_fds, it should return immediately.
        start = time.time()
        
        # A small script that forks a sleeper and exits
        daemon_script = (
            "import os, time, sys\n"
            "pid = os.fork()\n"
            "if pid == 0:\n"
            "    time.sleep(10)\n"
            "    sys.exit(0)\n"
            "else:\n"
            "    print('Forked daemon')\n"
            "    sys.exit(0)\n"
        )
        
        result = subprocess.run(
            [sys.executable, self.script_path, "5", sys.executable, "-c", daemon_script],
            capture_output=True,
            text=True
        )
        duration = time.time() - start
        
        self.assertEqual(result.returncode, 0)
        self.assertIn("Forked daemon", result.stdout)
        # It must exit immediately, not after 10 seconds
        self.assertLess(duration, 3.0)

if __name__ == '__main__':
    unittest.main()
