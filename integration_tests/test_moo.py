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
