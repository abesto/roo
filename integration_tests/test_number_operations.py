"""
Manipulating MOO Values / Operations on Numbers
https://www.sindome.org/moo-manual.html#operations-on-numbers
"""

from .conftest import Connect


def test_random(connect: Connect) -> None:
    c = connect()
    for n in range(20):
        c.send(";random(5)")
        n = int(c.readline().lstrip("=> ").strip())
        assert 0 <= n
        assert n <= 5


def test_min(connect: Connect) -> None:
    connect().cram(
        """
        $ ;min([])
        !! E_INVARG
        $ ;min([3, 10, 5, -2])
        => -2
        $ ;min([1.0, 2.3, 0.2])
        => 0.2
        $ ;min([1, 2, 1.2])
        !! E_TYPE
        $ ;min([E_INVARG])
        !! E_TYPE
        """
    )


def test_max(connect: Connect) -> None:
    connect().cram(
        """
        $ ;max([])
        !! E_INVARG
        $ ;max([3, 10, 5, -2])
        => 10
        $ ;max([1.0, 2.3, 0.2])
        => 2.3
        $ ;max([1, 2, 1.2])
        !! E_TYPE
        $ ;max([E_INVARG])
        !! E_TYPE
        """
    )


def test_abs(connect: Connect) -> None:
    connect().cram(
        """
        $ ;abs(10)
        => 10
        $ ;abs(-20)
        => 20
        $ ;abs(3.2)
        => 3.2
        $ ;abs(-5.1)
        => 5.1
        $ ;abs(E_INVARG)
        !! E_TYPE
        $ ;abs("foo")
        !! E_TYPE
        """
    )
