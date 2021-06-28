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


def test_create(login: Login) -> None:
    client = login()

    # owner and parent set correctly when both are specified
    client.send(";o = create(S.Root, player):unwrap()")
    client.assert_lua_equals("o.owner", "player")
    client.assert_lua_equals("o.parent", "S.Root")

    # owner defaults to player
    client.send(";o = create(S.Root):unwrap()")
    client.assert_lua_equals("o.owner", "player")

    # owner is the new object itself if set to S.nothing
    client.send(";o = create(S.Root, S.nothing):unwrap()")
    client.assert_lua_equals("o.owner", "o")


def test_move(login: Login) -> None:
    client = login()
    r1 = client.lua_create("S.Room")
    r2 = client.lua_create("S.Room")

    # Assert the new object doesn't initially have a location
    client.send(";o = create(S.Root):unwrap()")
    client.assert_lua_nil("o.location")

    # Happy: Move by object reference
    client.assert_lua_nil(f"o:move(db['{r1}']):unwrap()")
    client.send(";o.location.uuid")
    assert r1 == client.read_uuid()

    # Happy: Move by uuid
    client.assert_lua_nil(f"o:move('{r2}'):unwrap()")
    client.send(";o.location.uuid")
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


def test_chparent(login: Login) -> None:
    client = login()

    client.send(
        ";o1 = create(S.Root):unwrap()",
        ";o2 = create(S.Root):unwrap()",
    )

    # Happy path
    client.assert_lua_nil("o1:chparent(o2):unwrap()")
    client.assert_lua_equals("o1.parent", "o2")

    # Can be reparented to nothing
    client.assert_lua_nil("o2:chparent(S.nothing):unwrap()")
    client.assert_lua_equals("o2.parent", "S.nothing")

    # TODO test errors


def test_get_property(login: Login) -> None:
    client = login()

    # Happy
    client.assert_lua_equals("player.name", "'test_get_property'")

    # Sad: setup
    uuid = client.lua_create("S.Root")
    client.send(
        f";o = db['{uuid}']",
        ";o.name = 'testobj'",
    )
    client.assert_lua_equals("o.name", "'testobj'")

    # Sad: assert
    client.send(";o:recycle():unwrap()", ";o.name")
    client.expect_exact(f"{uuid} not found")


def test_set_property(login: Login) -> None:
    client = login()

    # Integer
    client.send(";player.x = 33")
    client.assert_lua_equals("player.x", "33")

    # String
    client.send(";player.x = 'foo'")
    client.assert_lua_equals("player.x", "'foo'")

    # UUID
    client.send(";player.x = S.nothing.uuid")
    client.assert_lua_equals("player.x", "S.nothing")

    # Object reference
    client.send(";player.x = S.nothing")
    client.assert_lua_equals("player.x", "S.nothing")

    # Try to set parent, fail
    client.send(";player.parent = 23")
    client.expect_exact(".parent cannot be set directly")

    # Try to set name to wrong type
    client.send(";player.name = 3")
    client.expect_exact("Tried to assign value of wrong type")


def test_add_verb(login: Login) -> None:
    client = login()

    # Simple case
