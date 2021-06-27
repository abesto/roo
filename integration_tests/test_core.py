"""Tests for the minimal core included with the server"""

from .conftest import Logins


def test_say(logins: Logins) -> None:
    c1, c2 = logins(2)
    c1.send("say hello hi")
    c1.expect_exact('You say, "hello hi"')
    c2.expect_exact(f'{c1.name} says, "hello hi"')


def test_say_shortcut(logins: Logins) -> None:
    c1, c2 = logins(2)
    c1.send('"yes awesome')
    c1.expect_exact('You say, "yes awesome"')
    c2.expect_exact(f'{c1.name} says, "yes awesome"')


def test_emote(logins: Logins) -> None:
    c1, c2 = logins(2)
    c1.send("emote waves")
    c2.expect_exact(f"{c1.name} waves")


def test_emote_shortcut(logins: Logins) -> None:
    c1, c2 = logins(2)
    c2.send(":jumps")
    c1.expect_exact(f"{c2.name} jumps")


def test_look(logins: Logins) -> None:
    c1, c2 = logins(2)
    c1.send("look")
    c2.send("look")
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
