#!/usr/bin/env python

import signal
import subprocess
import functools
import sys
import os
import telnetlib
import contextlib
from typing import Any, IO, Optional, Set
from unittest.case import TestCase
from functools import wraps

import pexpect
import pexpect.fdpexpect


class Prefixed:
    def __init__(self, prefix: str, f: Optional[IO[Any]] = None) -> None:
        self.f = f or sys.stdout
        self.prefix = prefix

    def write(self, msg: str) -> None:
        self.f.write(
            os.linesep.join(f"{self.prefix}{line}" for line in msg.splitlines())
            + os.linesep
        )

    def flush(self) -> None:
        self.f.flush()


@contextlib.contextmanager
def _server():
    server = pexpect.spawn(
        "./target/debug/roo testing", logfile=Prefixed("server] "), encoding="utf-8"
    )
    try:
        server.expect_exact("Server started")
        print("pexpect] Server started")
        yield server
    finally:
        server.kill(signal.SIGTERM)
        print("pexpect] Server stopped")


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


class Client(pexpect.fdpexpect.fdspawn):
    heartbeat_sequence: int = 0
    name: Optional[str] = None

    def heartbeat(self) -> None:
        self.sendline(f";{self.heartbeat_sequence}")
        self.expect_exact(f"Integer({self.heartbeat_sequence})")
        self.heartbeat_sequence += 1

    def read_uuid(self) -> str:
        self.expect(r"[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}")
        return self.match[0]

    def expect_lines_exact(self, *lines: str) -> None:
        for line in lines:
            self.expect_exact(line)


def new_client(login: Optional[str] = "__unset__"):
    def outer(f):
        if login == "__unset__":
            username = f.__name__
        else:
            username = login

        @wraps(f)
        def inner(self, *args, **kwargs):
            with telnetlib.Telnet("localhost", 8888) as t:
                client = Client(
                    t,
                    encoding="utf-8",
                    timeout=3,
                )
                client.logfile_send = Prefixed(f"{username or ''} >> ")
                client.logfile_read = Prefixed(f"{username or ''} << ")
                if username:
                    client.expect_exact(
                        "Ohai! Type 'connect <username>' to get started"
                    )
                    client.sendline(f"connect {username}")
                    client.expect(f"Welcome, {username}")
                    client.name = username
                f(self, client, *args, **kwargs)

        return inner

    return outer


def compose2(f, g):
    return lambda *a, **kw: f(g(*a, **kw))


def compose(*fs):
    return functools.reduce(compose2, fs)


def new_clients(count: int, prefix: Optional[str] = None):
    def outer(f):
        if prefix:
            real_prefix = prefix
        else:
            real_prefix = f.__name__ + "-"

        usernames = [f"{real_prefix}{n}" for n in range(count)]
        return compose(*(new_client(username) for username in usernames))(f)

    return outer
