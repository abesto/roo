import pexpect

from .conftest import Connect, Login


def test_login(connect: Connect):
    client = connect()
    client.expect_exact("Ohai! Type 'connect <username>' to get started")
    client.send("connect testuser")
    client.expect("Welcome, testuser")

    client.assert_lua_equals("player.owner", "player")


def test_lua_integer_literal(login: Login) -> None:
    client = login()
    client.sendline(";42")
    client.expect_exact("Integer(42)")


def test_lua_error(login: Login) -> None:
    client = login()
    client.send(";foobar()")
    client.expect("variable 'foobar' is not declared")
    client.send(";Err(1):unwrap()")
    client.expect_exact(":unwrap() called on an Err: 1")


def test_notify(login: Login) -> None:
    client = login(interleave_server_logs=True)

    client.send(";_server_notify(player.uuid, 'test-1')")
    client.expect_exact("test-1")

    client.send(";player:notify('test-2'):unwrap()")
    client.expect_exact("test-2")


def test_gc_works(login: Login) -> None:
    client = login(interleave_server_logs=True)
    client.send(
        ";create(S.Root)",
        ";mt = {}",
        ";function mt.__gc() print('whey'); player:notify('test: done gc') end",
        ";x = {}",
        ";setmetatable(x, mt)",
        ";x = nil",
    )
    client.expect_exact("test: done gc")


def test_unchecked_result(login: Login, server: pexpect.spawn) -> None:
    client = login(interleave_server_logs=True)
    client.send(
        ";print('here')",
        ";create(S.Root)",
        ";mt = {}",
        ";function mt.__gc() print('gc!') end",
        ";x = {}",
        ";setmetatable(x, mt)",
        ";x = nil",
    )
    client.expect("Value of Result was never checked")
