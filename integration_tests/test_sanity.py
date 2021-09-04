from .conftest import Connect
import pytest


def test_simple_rhai(connect: Connect) -> None:
    c = connect()
    c.cram(
        """
    $ ;40 + 2
    => 42
    """
    )


def test_rhai_variables(connect: Connect) -> None:
    c = connect()
    c.cram(
        """
    $ ;let x = 20
    $ ;x
    => 20
    """
    )


def test_object_N_notation(connect: Connect) -> None:
    connect().cram(
        """
        $ ;N42.to_string()
        => N42
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
    => foo bar
    """
    )


def test_highest_object_number(connect: Connect) -> None:
    connect().cram(
        """
        $ ;create(N0, N0).to_string()
        => N1
        $ ;create(N0, N0).to_string()
        => N2
        $ ;get_highest_object_number()
        => 2
        """
    )

    connect().cram(
        """
        $ ;get_highest_object_number()
        => 2
        """
    )


def test_object_O_notation(connect: Connect) -> None:
    connect().cram(
        """
        $ ;O(24).to_string()
        => N24
        """
    )


def test_valid(connect: Connect) -> None:
    connect().cram(
        """
        $ ;valid(O(-1))
        => false
        $ ;valid(create(N0, N0))
        => true
        """
    )


def test_get_name(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create(N0, N0)
        $ ;o.name
        => 
        $ ;o.foobar
        => E_PROPNF
        $ ;O(9999).name
        => E_INVIND
        """
    )


def test_set_name(connect: Connect) -> None:
    c1 = connect()

    c1.cram(
        """
        $ ;let o = create(N0, N0)
        $ ;o.name = "first"
        $ ;o.name
        => first
        """
    )

    c1.send(";o.to_string()")
    id = c1.readline().lstrip("=> N").rstrip()
    connect().cram(
        f"""
    $ ;O({id}).to_string()
    => N{id}
    $ ;N{id}.name
    => first
    $ ;N{id}.name = "second"
    $ ;N{id}.name
    => second
    """
    )

    connect().cram(
        f"""
    $ ;O({id}).name
    => second
    """
    )


def test_add_property_check_value(connect: Connect) -> None:
    connect().cram(
        """
    $ ;let o1 = create(N0, N0)
    $ ;let o2 = create(N0, N0)
    $ ;add_property(o2, "testprop", "testval", [o1, "rw"])
    $ ;o2.testprop
    => testval
    """
    )


def test_add_property_object_wrong_type(connect: Connect) -> None:
    connect().cram(
        """
        $ ;add_property("this-is-not-an-obj", "testprop", "testval", [N0, ""])
        Function not found: add_property (&str | ImmutableString | String, &str | ImmutableString | String, &str | ImmutableString | String, array) (line 1, position 1)
        """
    )


def test_add_property_invalid_object(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let owner = create(N0, N0)
        $ ;let o = O(get_highest_object_number() + 1)
        $ ;add_property(o, "testprop", "testval", [owner, ""])
        => E_INVARG
        """
    )


def test_add_property_invalid_owner(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create(N0, N0)
        $ ;let owner = O(get_highest_object_number() + 1)
        $ ;add_property(o, "testprop", "testval", [owner, ""])
        => E_INVARG
        """
    )


def test_add_property_already_exists(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create(N0, N0)
        $ ;let owner1 = create(N0, N0)
        $ ;let owner2 = create(N0, N0)
        $ ;add_property(o, "testprop", "testval1", [owner1, "rw"])
        $ ;add_property(o, "testprop", "testval2", [owner2, "wc"])
        => E_INVARG
        $ ;o.testprop
        => testval1
        """
    )
    # TODO verify correct owner, perms


@pytest.mark.xfail(reason="chparent / parent hierarchies not yet implemented")
def test_add_property_already_exists_on_grandparent(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create(N0, N0)
        $ ;let p = create(o, N0)
        $ ;let gp = create(p, N0)
        $ ;let owner1 = create()
        $ ;let owner2 = create()
        $ ;add_property(gp, "gprop", "val1", [owner1, "rw"])
        $ ;add_property(o, "gprop", "val2", [owner2, "wc"])
        => E_INVARG
        $ ;o.gprop
        => val
        """
    )
    # TODO verify correct owner, perms


def test_add_property_invalid_perms(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let owner = create(N0, N0)
        $ ;let o = create(N0, N0)
        $ ;add_property(o, "testprop", "testval", [owner, "qxa"])
        => E_INVARG
        """
    )


def test_add_property_check_property_info(connect: Connect) -> None:
    c = connect()
    c.send(";create(N0, N0).to_string()")
    owner = c.readline().lstrip("=> N").rstrip()

    connect().cram(
        f"""
    $ ;let o = create(N0, N0)
    $ ;add_property(o, "testprop", "testval", [N{owner}, "crw"])
    $ ;property_info(o, "testprop").to_string()
    => [N{owner}, "rwc"]
    """
    )


def test_property_info_invalid_object(connect: Connect) -> None:
    connect().cram(
        """
    $ ;property_info(O(-1), "whee")
    => E_INVARG
    """
    )


def test_property_info_no_such_property(connect: Connect) -> None:
    connect().cram(
        """
    $ ;let o = create(N0, N0)
    $ ;property_info(o, "foobar")
    => E_PROPNF
    """
    )


@pytest.mark.xfail
def test_property_info_no_read_perm(connect: Connect) -> None:
    raise NotImplementedError()


# TODO test permissions for property manipulation
