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
    c = connect()

    c.cram(
        """
    $ ;echo("foo bar")
    foo bar
    """
    )
