#!/usr/bin/env python
from integration_tests.lib import new_client, IntegrationTest, Client


class SanityTest(IntegrationTest):
    @new_client(login=None)
    def test_login(self, client) -> None:
        client.expect_exact("Ohai! Type 'connect <username>' to get started")
        client.sendline("connect testuser")
        client.expect("Welcome, testuser")

    @new_client()
    def test_lua_integer_literal(self, client: Client) -> None:
        client.sendline(";42")
        client.expect_exact("Integer(42)")

    @new_client()
    def test_lua_error(self, client: Client) -> None:
        client.sendline(";foobar()")
        client.expect("variable 'foobar' is not declared")
        client.sendline(";Err(1):unwrap()")
        client.expect_exact(":unwrap() called on an Err: 1")
