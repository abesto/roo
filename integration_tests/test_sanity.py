from .conftest import Connect, Login


def test_login(connect: Connect):
    client = connect()
    client.expect_exact("Ohai! Type 'connect <username>' to get started")
    client.send("connect testuser")
    client.expect("Welcome, testuser")


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
