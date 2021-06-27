#!/usr/bin/env python
from integration_tests.lib import new_clients, IntegrationTest, Client


class CoreTest(IntegrationTest):
    """Tests for the minimal core included with the server"""

    @new_clients(2)
    def test_say(self, c1: Client, c2: Client) -> None:
        c1.sendline("say hello hi")
        c1.expect_exact('You say, "hello hi"')
        c2.expect_exact(f'{c1.name} says, "hello hi"')

    @new_clients(2)
    def test_say_shortcut(self, c1: Client, c2: Client) -> None:
        c1.sendline('"yes awesome')
        c1.expect_exact('You say, "yes awesome"')
        c2.expect_exact(f'{c1.name} says, "yes awesome"')

    @new_clients(2)
    def test_emote(self, c1: Client, c2: Client) -> None:
        c1.sendline("emote waves")
        c2.expect_exact(f"{c1.name} waves")

    @new_clients(2)
    def test_emote_shortcut(self, c1: Client, c2: Client) -> None:
        c2.sendline(":waves")
        c1.expect_exact(f"{c2.name} waves")

    @new_clients(2)
    def test_look(self, c1: Client, c2: Client) -> None:
        c1.sendline("look")
        c2.sendline("look")
        c1.expect_lines_exact(
            "The Void",
            "There is nothing, and you are in it.",
            f"You see here: {c2.name}",
        )
        c2.expect_lines_exact(
            "The Void",
            "There is nothing, and you are in it.",
            f"You see here: {c1.name}",
        )
