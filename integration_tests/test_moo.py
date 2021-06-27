"""Test correct implementation of moo server functions"""

from .conftest import Login, Client


def test_verb_code_by_index(login: Login) -> None:
    client = login()
    client.send(
        ";o = create(S.Root):unwrap()",
        ';o:add_verb({S.uuid, "r", {"testverb"}}, {"any"}):unwrap()',
        ';o:set_verb_code("testverb", "print(99)"):unwrap()',
        ";pl.tablex.deepcompare(o:verb_code(1):unwrap(), {'print(99)'})",
    )
    client.expect_exact("Boolean(true)")


def test_move(login: Login) -> None:
    client = login()
    r1 = client.lua_create("S.Room")
    r2 = client.lua_create("S.Room")

    # Assert the new object doesn't initially have a location
    client.send(";o = create(S.Root):unwrap()", ";o.location == nil")
    client.expect_exact("Boolean(true)")

    # Move by object reference
    client.send(f";o:move(db['{r1}'])", ";o.location.uuid")
    assert r1 == client.read_uuid()

    # Move by uuid
    client.send(f";o:move('{r2}')", ";o.location.uuid")
    assert r2 == client.read_uuid()
