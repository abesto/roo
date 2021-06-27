#!/usr/bin/env python

import signal
import subprocess
import sys
import telnetlib
import contextlib
from typing import Optional
from unittest.case import TestCase
from functools import wraps

import pexpect
import pexpect.fdpexpect


@contextlib.contextmanager
def _server():
    server = pexpect.spawn(
        "./target/debug/roo testing", logfile=sys.stdout, encoding="utf-8"
    )
    try:
        server.expect_exact("Server started")
        print("pexpect: Server started")
        yield server
    finally:
        server.kill(signal.SIGTERM)
        print("pexpect: Server stopped")


class IntegrationTest(TestCase):
    def setUp(self) -> None:
        super().setUp()
        self.exitstack = contextlib.ExitStack()
        self.server = self.exitstack.enter_context(_server())

    def tearDown(self) -> None:
        self.exitstack.close()
        super().tearDown()

    @classmethod
    def setUpClass(cls) -> None:
        p = subprocess.run(
            ["bash", "-lc", "cargo build"],
            stderr=subprocess.PIPE,
            stdout=subprocess.PIPE,
        )
        print(p.stderr)
        print(p.stdout)
        p.check_returncode()


def new_client(login: Optional[str] = "testuser"):
    def outer(f):
        @wraps(f)
        def inner(self, *args, **kwargs):
            t = telnetlib.Telnet("localhost", 8888)
            try:
                client = pexpect.fdpexpect.fdspawn(
                    t,
                    logfile=sys.stdout,
                    encoding="utf-8",
                    timeout=3,
                )
                if login:
                    client.expect_exact(
                        "Ohai! Type 'connect <username>' to get started"
                    )
                    client.sendline(f"connect {login}")
                    client.expect(f"Welcome, {login}")
                f(self, client, *args, **kwargs)
            finally:
                t.close()

        return inner

    return outer
