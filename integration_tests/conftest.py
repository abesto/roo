#!/usr/bin/env python

import signal
import sys
import os
import telnetlib
from typing import Any, IO, Optional, Protocol, Generator
import contextlib

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


@pytest.fixture
def server(build_server) -> pexpect.spawn:
    server = pexpect.spawn(
        "./target/debug/roo testing",
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

    def read_uuid(self) -> str:
        self.expect(r"[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}")
        return self.match[0]

    def expect_lines_exact(self, *lines: str) -> None:
        for line in lines:
            self.expect_exact(line)

    def send(self, *lines: str) -> None:
        for line in lines:
            self._log(line, "send")
            os.write(self.child_fd, f"{line}{self.linesep}".encode("utf-8"))
            if self.interleave_server_logs:
                # Wait for the server to finish executing the command.
                # We know it's our command, because we're not concurrent.
                self.server.expect(r"eval-start: (\S*)")
                chunk_name = self.server.match[1]
                self.server.expect_exact(f"eval-done: {chunk_name}")

    def lua_create(self, parent: str) -> str:
        self.send(f";create({parent}):unwrap().uuid")
        return self.read_uuid()

    def expect_lua_boolean(self, expected: bool, msg: str = ""):
        idx = self.expect_exact(["Boolean(true)", "Boolean(false)"])
        if expected:
            assert idx == 0, f"Expected true, found false: {msg}"
        else:
            assert idx == 1, f"Expected false, found true: {msg}"

    def assert_lua_equals(self, left: str, right: str):
        code = f"{left} == {right}"
        self.send(f";{code}")
        self.expect_lua_boolean(True, code)

    def assert_lua_nil(self, code: str):
        self.assert_lua_equals(code, "nil")


@pytest.fixture()
def connect(server):
    exitstack = contextlib.ExitStack()

    def _connect():
        t = telnetlib.Telnet("localhost", 8888)
        exitstack.enter_context(t)
        client = Client(
            t,
            encoding="utf-8",
            timeout=3,
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
