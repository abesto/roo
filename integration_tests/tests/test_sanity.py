#!/usr/bin/env python
from lib import new_client, IntegrationTest


class SanityTest(IntegrationTest):
    @new_client(login=None)
    def test_login(self, client) -> None:
        client.expect_exact("Ohai! Type 'connect <username>' to get started")
        client.sendline("connect testuser")
        client.expect("Welcome, testuser")

    @new_client()
    def test_verb_code_by_index(self, client) -> None:
        client.sendline(";o = create(S.Root):unwrap()")
        client.sendline(';o:add_verb({S.uuid, "r", {"testverb"}}, {"any"}):unwrap()')
        client.sendline(';o:set_verb_code("testverb", "print(99)"):unwrap()')
        client.sendline(";print(o:verb_code(0):unrap())")
        client.expect_exact("print 99")
