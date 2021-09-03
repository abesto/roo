from .conftest import Connect


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


def test_get_property(connect: Connect) -> None:
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


def test_set_property(connect: Connect) -> None:
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
    connect().cram(f"""
    $ ;O({id})
    #{id}
    $ ;O({id}).name
    first
    $ ;let o = O({id})
    $ ;o.name = "second"
    $ ;O({id}).name
    second
    """)

    connect().cram(f"""
    $ ;O({id}).name
    second
    """)