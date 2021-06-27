#!/usr/bin/env python
from integration_tests.lib import new_client, IntegrationTest, Client


class MooTest(IntegrationTest):
    """Test correct implementation of moo server functions"""

    @new_client()
    def test_verb_code_by_index(self, client: Client) -> None:
        client.sendline(";o = create(S.Root):unwrap()")
        client.sendline(';o:add_verb({S.uuid, "r", {"testverb"}}, {"any"}):unwrap()')
        client.sendline(';o:set_verb_code("testverb", "print(99)"):unwrap()')
        client.sendline(
            ";pl.tablex.deepcompare(o:verb_code(1):unwrap(), {'print(99)'})"
        )
        client.expect_exact("Boolean(true)")
