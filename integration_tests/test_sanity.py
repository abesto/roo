from .conftest import Connect
import pytest


def test_simple_rhai(connect: Connect) -> None:
    c = connect()
    c.cram(
        """
    $ ;40 + 2
    42
"""
    )


def test_rhai_variables(connect: Connect) -> None:
    c = connect()
    c.cram(
        """
    $ ;let x = 20
    $ ;x
    20
    """
    )


def test_rhai_variable_isolation(connect: Connect) -> None:
    c1 = connect()
    c2 = connect()

    c1.send(";x = 'foo'")
    c2.cram(
        """
    $ ;x
    Variable not found: x (line 1, position 1)
    """
    )


def test_rhai_echo(connect: Connect) -> None:
    connect().cram(
        """
    $ ;echo("foo bar")
    foo bar
    """
    )


def test_highest_object_number(connect: Connect) -> None:
    connect().cram(
        """
        $ ;create()
        #1
        $ ;create()
        #2
        $ ;get_highest_object_number()
        2
        """
    )

    connect().cram(
        """
        $ ;get_highest_object_number()
        2
        """
    )


def test_object_literal(connect: Connect) -> None:
    connect().cram(
        """
        $ ;O(24)
        #24
        """
    )


def test_valid(connect: Connect) -> None:
    connect().cram(
        """
        $ ;valid(O(9999))
        false
        $ ;valid(create())
        true
        """
    )


def test_get_name(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create()
        $ ;o.name

        $ ;o.foobar
        E_PROPNF
        $ ;O(9999).name
        E_INVIND
        """
    )


def test_set_name(connect: Connect) -> None:
    c1 = connect()

    c1.cram(
        """
        $ ;let o = create()
        $ ;o.name = "first"
        $ ;o.name
        first
        """
    )

    c1.send(";o")
    id = c1.readline().lstrip("#").rstrip()
    connect().cram(
        f"""
    $ ;O({id})
    #{id}
    $ ;O({id}).name
    first
    $ ;let o = O({id})
    $ ;o.name = "second"
    $ ;O({id}).name
    second
    """
    )

    connect().cram(
        f"""
    $ ;O({id}).name
    second
    """
    )


def test_add_property_check_value(connect: Connect) -> None:
    connect().cram(
        """
    $ ;let o1 = create()
    $ ;let o2 = create()
    $ ;add_property(o2, "testprop", "testval", [o1, "rw"])
    $ ;o2.testprop
    testval
    """
    )


def test_add_property_object_wrong_type(connect: Connect) -> None:
    connect().cram(
        """
        $ ;add_property("this-is-not-an-obj", "testprop", "testval", [create(), ""])
        Function not found: add_property (&str | ImmutableString | String, &str | ImmutableString | String, &str | ImmutableString | String, array) (line 1, position 1)
        """
    )


def test_add_property_invalid_object(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let owner = create()
        $ ;let o = O(get_highest_object_number() + 1)
        $ ;add_property(o, "testprop", "testval", [owner, ""])
        E_INVARG
        """
    )


def test_add_property_invalid_owner(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create()
        $ ;let owner = O(get_highest_object_number() + 1)
        $ ;add_property(o, "testprop", "testval", [owner, ""])
        E_INVARG
        """
    )


def test_add_property_already_exists(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create()
        $ ;let owner1 = create()
        $ ;let owner2 = create()
        $ ;add_property(o, "testprop", "testval1", [owner1, "rw"])
        $ ;add_property(o, "testprop", "testval2", [owner2, "wc"])
        E_INVARG
        $ ;o.testprop
        testval1
        """
    )
    # TODO verify correct owner, perms


@pytest.mark.xfail(reason="chparent / parent hierarchies not yet implemented")
def test_add_property_already_exists_on_grandparent(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create()
        $ ;let p = create()
        $ ;let gp = create()
        $ ;chparent(o, p)
        $ ;chparent(p, gp)
        $ ;let owner1 = create()
        $ ;let owner2 = create()
        $ ;add_property(gp, "gprop", "val1", [owner1, "rw"])
        $ ;add_property(o, "gprop", "val2", [owner2, "wc"])
        E_INVARG
        $ ;o.gprop
        val
        """
    )
    # TODO verify correct owner, perms


def test_add_property_invalid_perms(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let owner = create()
        $ ;let o = create()
        $ ;add_property(o, "testprop", "testval", [owner, "qxa"])
        E_INVARG
        """
    )

def test_add_property_check_property_info(connect: Connect) -> None:
    connect().cram(
        """
    $ ;let o1 = create()
    $ ;let o2 = create()
    $ ;add_property(o2, "testprop", "testval", [o1, "rw"])
    $ ;property_info(o2, "testprop")
    """
    )
