#!/usr/bin/env python

import signal
import sys
import os
import telnetlib
from typing import Any, IO, Optional, Protocol, Generator
import contextlib
import textwrap
import re

import pexpect
import pexpect.fdpexpect
from _pytest.fixtures import SubRequest
import pytest


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


@pytest.fixture(scope="session")
def build_server() -> None:
    output, status = pexpect.run(
        "bash -lc 'exec cargo build --color=never'",
        timeout=300,
        encoding="utf-8",
        withexitstatus=True,
    )
    if status != 0:
        print(output)
    assert status == 0
    print("Server built")


@pytest.fixture
def server(build_server) -> pexpect.spawn:
    server = pexpect.spawn(
        # "./target/debug/roo testing",
        "./target/debug/roo",
        encoding="utf-8",
    )
    server.logfile_read = Prefixed("server] ")
    server.expect_exact("Server started")
    print("pexpect] Server started")
    yield server
    server.kill(signal.SIGTERM)
    print("pexpect] Server stopped")


class Client(pexpect.fdpexpect.fdspawn):
    server: pexpect.spawn
    heartbeat_sequence: int = 0
    name: Optional[str] = None
    interleave_server_logs: bool = False

    def heartbeat(self) -> None:
        self.sendline(f";{self.heartbeat_sequence}")
        self.expect_exact(f"Integer({self.heartbeat_sequence})")
        self.heartbeat_sequence += 1

    def expect_lines_exact(self, *lines: str) -> None:
        #print(f"pexpect] Expect: {lines}")
        for line in lines:
            self.expect(re.compile(f"^{re.escape(line)}\r\n"))

    def send(self, *lines: str) -> None:
        for line in lines:
            self._log(line, "send")
            os.write(self.child_fd, f"{line}{self.linesep}".encode("utf-8"))
            if self.interleave_server_logs:
                # TODO: Wait for the server to finish executing the command.
                pass
                # self.server.expect(r"eval-start: (\S*)")
                # chunk_name = self.server.match[1]
                # self.server.expect_exact(f"eval-done: {chunk_name}")

    def cram(self, spec: str) -> None:
        lines = textwrap.dedent(spec).splitlines()
        while lines and not lines[0]:
            lines.pop(0)

        expect_lines = []
        for line in lines:
            if line.startswith("$"):
                self.expect_lines_exact(*expect_lines)
                expect_lines = []
                self.send(line.lstrip("$ "))
            else:
                expect_lines.append(line)
        self.expect_lines_exact(*expect_lines)


@pytest.fixture()
def connect(server):
    exitstack = contextlib.ExitStack()

    def _connect():
        t = telnetlib.Telnet("localhost", 8888)
        exitstack.enter_context(t)
        client = Client(
            t,
            encoding="utf-8",
            timeout=1,
        )
        client.logfile_send = Prefixed(">> ")
        client.logfile_read = Prefixed("<< ")
        client.server = server
        return client

    yield _connect

    exitstack.close()


class Connect(Protocol):
    def __call__(self) -> Client:
        ...


@pytest.fixture()
def login(request: SubRequest, connect: Connect):
    def _login(username: Optional[str] = None, interleave_server_logs: bool = False):
        if username is None:
            username = request.function.__name__
        client = connect()
        client.interleave_server_logs = interleave_server_logs
        client.logfile_send.prefix = f"{username} >> "
        client.logfile_read.prefix = f"{username} << "
        client.name = username
        client.send(f"connect {username}")
        client.expect_exact(f"Welcome, {username}")
        return client

    return _login


class Login(Protocol):
    def __call__(
        self, username: Optional[str] = None, interleave_server_logs: bool = False
    ) -> Client:
        ...


@pytest.fixture()
def logins(request: SubRequest, login: Login):
    def _logins(
        count: int, prefix: Optional[str] = None, interleave_server_logs: bool = False
    ):
        if prefix is None:
            prefix = request.function.__name__ + "-"
        return [login(f"{prefix}{n}", interleave_server_logs) for n in range(count)]

    return _logins


class Logins(Protocol):
    def __call__(
        self,
        count: int,
        prefix: Optional[str] = None,
        interleave_server_logs: bool = False,
    ) -> Generator[Client, None, None]:
        ...
