from .conftest import Connect
import pytest


def test_get_name(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create(N0, N0)
        $ ;o.name
        => ""
        $ ;N9999.name
        !! E_INVIND
        """
    )


def test_set_name(connect: Connect) -> None:
    c1 = connect()

    c1.cram(
        """
        $ ;let o = create(N0, N0)
        $ ;o.name = "first"
        $ ;o.name
        => "first"
        """
    )

    c1.send(";o.to_string()")
    id = c1.readline().strip('=> N"').rstrip().rstrip('"')
    connect().cram(
        f"""
    $ ;N{id}.name
    => "first"
    $ ;let o = toobj({id}); o.name = "second"
    $ ;N{id}.name
    => "second"
    """
    )

    connect().cram(
        f"""
    $ ;N{id}.name
    => "second"
    """
    )


def test_set_missing_property(connect: Connect) -> None:
    connect().cram(
        """
    $ ;let o = create(N0, N0)
    $ ; o.x = 3
    !! E_PROPNF
    """
    )


def test_add_property_check_value(connect: Connect) -> None:
    connect().cram(
        """
    $ ;let o1 = create(N0, N0)
    $ ;let o2 = create(N0, N0)
    $ ;add_property(o2, "testprop", "testval", [o1, "rw"])
    $ ;o2.testprop
    => "testval"
    """
    )


def test_add_property_object_wrong_type(connect: Connect) -> None:
    connect().cram(
        """
        $ ;add_property("this-is-not-an-obj", "testprop", "testval", [N0, ""])
        Function not found: add_property (&str | ImmutableString | String, &str | ImmutableString | String, &str | ImmutableString | String, array) (line 1, position 1) in call to function eval (line 1, position 16)
        """
    )


def test_add_property_invalid_object(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let owner = create(N0, N0)
        $ ;let o = toobj(get_highest_object_number() + 1)
        $ ;add_property(o, "testprop", "testval", [owner, ""])
        !! E_INVARG
        """
    )


def test_add_property_invalid_owner(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create(N0, N0)
        $ ;let owner = toobj(get_highest_object_number() + 1)
        $ ;add_property(o, "testprop", "testval", [owner, ""])
        !! E_INVARG
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
        !! E_INVARG
        $ ;o.testprop
        => "testval1"
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
        !! E_INVARG
        $ ;o.gprop
        => "val"
        """
    )
    # TODO verify correct owner, perms


def test_add_property_invalid_perms(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let owner = create(N0, N0)
        $ ;let o = create(N0, N0)
        $ ;add_property(o, "testprop", "testval", [owner, "qxa"])
        !! E_INVARG
        """
    )


def test_add_property_check_property_info(connect: Connect) -> None:
    c = connect()
    c.send(";create(N0, N0)")
    owner = c.readline().lstrip('=> N"').rstrip().rstrip('"')

    connect().cram(
        f"""
    $ ;let o = create(N0, N0)
    $ ;add_property(o, "testprop", "testval", [N{owner}, "crw"])
    $ ;property_info(o, "testprop")
    => [N{owner}, "rwc"]
    """
    )


def test_property_info_invalid_object(connect: Connect) -> None:
    connect().cram(
        """
    $ ;property_info(toobj(-1), "whee")
    !! E_INVARG
    """
    )


def test_property_info_no_such_property(connect: Connect) -> None:
    connect().cram(
        """
    $ ;let o = create(N0, N0)
    $ ;property_info(o, "foobar")
    !! E_PROPNF
    """
    )


def test_add_set_get_property(connect: Connect) -> None:
    connect().cram(
        """
        $ ;let o = create(N0, N0)
        $ ;add_property(o, "x", [1, 2], [N0, ""])
        $ ;o.x
        => [1, 2]
        $ ;o.x = "foobar"
        $ ;o.x
        => "foobar"
        """
    )


@pytest.mark.xfail
def test_property_info_no_read_perm(connect: Connect) -> None:
    raise NotImplementedError()


# TODO test permissions for property manipulation
