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


def test_gc(login: Login) -> None:
    client = login()
    client.send(
        ";mt = {}",
        ";function mt.__gc() print('whey'); player:notify('test: done gc') end",
        ";x = {}",
        ";setmetatable(x, mt)",
        ";x = nil",
    )
    client.expect_exact("test: done gc")


def test_pl_class_gc(login: Login) -> None:
    client = login(interleave_server_logs=True)

    client.send(
        ";pl.class.Test()",
        ";function Test:__gc() player:notify('pl gc done') end",
        ";x = Test()",
        ";x = nil",
    )
    client.expect_exact("pl gc done")

    client.send(";pl.class.Sub(Test)", ";y = Sub()", ";y = nil")
    client.expect_exact("pl gc done")


def test_unchecked_result(login: Login) -> None:
    client = login(interleave_server_logs=True)
    client.send(";x = create(S.Root)", ";x = nil")
    client.expect_exact(
        "Value of Result was never checked. Created at: stack traceback:"
    )
    client.expect_exact('[string ";-command"]:1: in main chunk')
