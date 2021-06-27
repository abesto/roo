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


@pytest.fixture
def server() -> pexpect.spawn:
    output, status = pexpect.run(
        "bash -lc 'exec cargo build --color=never'",
        timeout=300,
        logfile=sys.stdout,
        encoding="utf-8",
        withexitstatus=True,
    )
    assert status == 0
    server = pexpect.spawn(
        "./target/debug/roo testing", logfile=Prefixed("server] "), encoding="utf-8"
    )
    server.expect_exact("Server started")
    print("pexpect] Server started")
    yield server
    server.kill(signal.SIGTERM)
    print("pexpect] Server stopped")


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

    def send(self, *lines: str) -> None:
        for line in lines:
            self._log(line, "send")
            os.write(self.child_fd, f"{line}{self.linesep}".encode("utf-8"))

    def lua_create(self, parent: str) -> str:
        self.send(f";create({parent}):unwrap().uuid")
        return self.read_uuid()


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
        return client

    yield _connect

    exitstack.close()


class Connect(Protocol):
    def __call__(self) -> Client:
        ...


@pytest.fixture()
def login(request: SubRequest, connect: Connect):
    def _login(username: Optional[str] = None):
        if username is None:
            username = request.function.__name__
        client = connect()
        client.logfile_send.prefix = f"{username} >> "
        client.logfile_read.prefix = f"{username} << "
        client.name = username
        client.send(f"connect {username}")
        client.expect_exact(f"Welcome, {username}")
        return client

    return _login


class Login(Protocol):
    def __call__(self, username: Optional[str] = None) -> Client:
        ...


@pytest.fixture()
def logins(request: SubRequest, login: Login):
    def _logins(count: int, prefix: Optional[str] = None):
        if prefix is None:
            prefix = request.function.__name__ + "-"
        return [login(f"{prefix}{n}") for n in range(count)]

    return _logins


class Logins(Protocol):
    def __call__(
        self, count: int, prefix: Optional[str] = None
    ) -> Generator[Client, None, None]:
        ...
