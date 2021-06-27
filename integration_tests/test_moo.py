"""Test correct implementation of moo server functions"""

from .conftest import Login


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

    # Happy: Move by object reference
    client.send(f";o:move(db['{r1}']):unwrap() == nil", ";o.location.uuid")
    client.expect_exact("Boolean(true)")
    assert r1 == client.read_uuid()

    # Happy: Move by uuid
    client.send(f";o:move('{r2}'):unwrap() == nil", ";o.location.uuid")
    client.expect_exact("Boolean(true)")
    assert r2 == client.read_uuid()

    # Sad: Move to something that's not a uuid
    client.send(";o:move('foobar'):unwrap().code")
    client.expect_exact("E_INVARG")

    # Sad: Move to a nonexistent uuid
    p = client.lua_create("S.Root")
    client.send(
        f";recycle('{p}'):unwrap()",
        f";o:move('{p}'):unwrap()",
    )
    client.expect_exact(f"E_PERM ({p} not found)")
