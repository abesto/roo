"""Test correct implementation of moo server functions"""

from .conftest import Login


def test_verb_code_by_index(login: Login) -> None:
    client = login()
    client.send(
        ";o = create(S.Root):unwrap()",
        ';o:add_verb({S.uuid, "r", {"testverb"}}, {"any"}):unwrap()',
        ';o:set_verb_code("testverb", "print(99)"):unwrap()',
    )
    client.assert_lua_deepequal("o:verb_code(1):unwrap()", "{'print(99)'}")

    # TODO error cases


def test_set_verb_code(login: Login) -> None:
    client = login(interleave_server_logs=True)
    o = client.lua_create("S.Root", "o")

    # Happy path by name
    client.send(
        ';o:add_verb({S.uuid, "r", {"testverb"}}, {"any"}):unwrap()',
        ';o:set_verb_code("testverb", "print(99)"):unwrap()',
    )
    client.assert_lua_deepequal("o:verb_code(1):unwrap()", "{'print(99)'}")

    # Happy path by index
    client.send(
        ';o:set_verb_code(1, "print(42)"):unwrap()',
    )
    client.assert_lua_deepequal("o:verb_code(1):unwrap()", "{'print(42)'}")

    # Invalid object
    p = client.lua_create("S.Root", "p")
    client.send(";p:recycle():unwrap()", ";p:set_verb_code('testverb', {}):unwrap()")
    client.expect_exact(f"E_INVARG ({p} not found)")

    # Invalid code
    client.send(
        ';o:set_verb_code(1, "this is not valid lua code at all"):unwrap()',
    )
    client.expect_exact("E_INVARG (syntax error")

    # Verb by name doesn't exist
    client.send(
        ';o:set_verb_code("whee", "print(9000)"):unwrap()',
    )
    client.expect_exact(f"E_VERBNF ({o} has no verb with name whee)")

    # Verb by index doesn't exist
    client.send(
        ';o:set_verb_code(30, "print(9000)"):unwrap()',
    )
    client.expect_exact(f"E_VERBNF ({o} has no verb with index 30)")


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
    client = login(interleave_server_logs=True)
    client.lua_create("S.Root", "o")
    client.lua_create("S.Room", "r1")
    client.lua_create("S.Room", "r2")

    # Assert the new object doesn't initially have a location
    client.assert_lua_nil("o.location")

    # Happy: Move by object reference
    client.assert_lua_nil("o:move(r1):unwrap()")
    client.assert_lua_equals("o.location", "r1")

    # Happy: Move by uuid
    client.assert_lua_nil("o:move(r2.uuid):unwrap()")
    client.assert_lua_equals("o.location", "r2")

    # Sad: Move to something that's not a uuid
    client.send(";o:move('foobar'):unwrap().code")
    client.expect_exact("E_INVARG")

    # Sad: Move to a nonexistent uuid
    p = client.lua_create("S.Root", "p")
    client.send(
        ";uuid = p.uuid",
        ";p:recycle():unwrap()",
        ";o:move(uuid):unwrap()",
    )
    client.expect_exact(f"E_PERM ({p} not found)")

    # Sad: Move to object reference to nonexistent object
    p = client.lua_create("S.Root", "p")
    client.send(
        ";p:recycle():unwrap()",
        ";o:move(p):unwrap()",
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
    uuid = client.lua_create("S.Root", "o")
    client.send(";o.name = 'testobj'")
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


def test_set_into_list(login: Login) -> None:
    client = login()
    o = client.lua_create("S.Root", "o")

    # Create a new list with some default elements
    client.send(";o.l = {1, 'foo'}")
    client.assert_lua_deepequal("o.l._inner", "{1, 'foo'}")

    # Overwrite an existing element with nil
    client.send(";o.l[1] = nil")
    client.assert_lua_deepequal("o.l._inner", "{nil, 'foo'}")

    # UUID gets expanded into object
    client.send(";table.insert(o.l, o)")
    client.assert_lua_equals("o.l[3]", "o")

    # Append to the end
    client.send(";table.insert(o.l, 1212)")
    client.assert_lua_deepequal("o.l._inner", "{nil, 'foo', '%s', 1212}" % (o,))

    # Test removing items from the list
    client.send(";o.l = {1, 2, 3, 4, 5}", ";table.remove(o.l)")
    client.assert_lua_deepequal("o.l._inner", "{1, 2, 3, 4}")
    client.send(";table.remove(o.l, 2)")
    client.assert_lua_deepequal("o.l._inner", "{1, 3, 4}")

    # Out of bounds, nested for good measure
    client.send(";o.l = {1, 2, {3, 4}}", ";o.l[3][4] = 'foo'")
    client.expect_exact(f"E_RANGE (#{o}.l[3] == 2 (index out of bounds: 4))")

    # TODO missing property
    # TODO not a list
    # TODO index list with string


def test_add_verb(login: Login) -> None:
    client = login(interleave_server_logs=True)

    # Simple case
    client.lua_create("S.Root", "o")
    client.assert_lua_nil("o:add_verb({player, 'rx', {'testverb'}}, {'any'}):unwrap()")
    # TODO check verbinfo, verbargs once those functions are implemented

    # Recreate verb
    client.send(";o:add_verb({player, 'rx', {'testverb'}}, {'any'}):unwrap()")
    client.expect_exact("already contains verb testverb")

    # Non-existent owner
    missing = client.lua_create("S.Root", "missing")
    client.send(
        ";missing:recycle():unwrap()",
        ";o:add_verb({missing, 'rx', {'testverb2'}}, {'any'}):unwrap()",
    )
    client.expect_exact(f"E_PERM ({missing} not found)")

    # Totally invalid verb-info
    client.send(";o:add_verb('trololo', {}):unwrap()")
    client.expect_exact("argument 2 expected a 'table', got a 'string'")

    # Mildly invalid verb-info
    client.send(";o:add_verb({missing}, {}):unwrap()")
    client.expect_exact("verb-info table must have exactly three elements")


def test_valid(login: Login) -> None:
    client = login()

    # Happy
    client.assert_lua_equals("player:valid()", "0")
    client.assert_lua_equals("valid(player)", "0")
    client.assert_lua_equals("valid(player.uuid)", "0")

    # Sad
    client.send(
        ";o = create(S.Root):unwrap()", ";uuid = o.uuid", ";o:recycle():unwrap()"
    )
    client.assert_lua_equals("o:valid()", "1")
    client.assert_lua_equals("valid(o)", "1")
    client.assert_lua_equals("valid(uuid)", "1")

    # Wrong type
    client.send(";valid(23)")
    client.expect_exact("E_TYPE")

    # Invalid UUID
    client.send(";valid('totally-not-a-valid-uuid')")
    client.expect_exact("E_INVARG")


def test_verbs(login: Login) -> None:
    client = login()

    # Setup
    client.send(";o = create(S.Root):unwrap()")
    client.assert_lua_nil("o:add_verb({player, 'rx', {'testverb1'}}, {'any'}):unwrap()")
    client.assert_lua_nil("o:add_verb({player, 'rx', {'testverb2'}}, {'any'}):unwrap()")

    # Happy
    client.assert_lua_deepequal("o:verbs():unwrap()", "{'testverb1', 'testverb2'}")
    client.assert_lua_deepequal("verbs(o):unwrap()", "{'testverb1', 'testverb2'}")
    client.assert_lua_deepequal("verbs(o.uuid):unwrap()", "{'testverb1', 'testverb2'}")

    # Recycled object
    client.send(";o:recycle():unwrap()")
    client.assert_lua_equals("o:verbs():err().code", "'E_INVARG'")

    # Not an object
    client.assert_lua_equals("verbs('foobar'):err().code", "'E_INVARG'")

    # Wrong type
    client.assert_lua_equals("verbs(88):err().code", "'E_TYPE'")
