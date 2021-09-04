"""
Manipulating MOO Values / General Operations Applicable to all Values
https://www.sindome.org/moo-manual.html#properties-on-objects
"""

from .conftest import Connect

def test_type_of(connect: Connect) -> None:
    connect().cram("""
    $ ;type_of(3)
    => "i64"
    $ ;type_of("foobar")
    => "string"
    $ ;type_of('c')
    => "char"
    $ ;type_of(3.14)
    => "f64"
    $ ;type_of([1, "foo"])
    => "array"
    $ ;type_of(N0)
    => "Object"
    $ ;type_of(E_INVARG)
    => "Error"
    """)


def test_toliteral(connect: Connect) -> None:
    connect().cram(
        """
        $ ;toliteral(17)
        => "17"
        $ ;toliteral(1.0/3.0)
        => "0.3333333333333333"
        $ ;toliteral(N17)
        => "N17"
        $ ;toliteral("foo")
        => "\\"foo\\""
        $ ;toliteral([1, 2])
        => "[1, 2]"
        $ ;toliteral(E_PERM)
        => "E_PERM"
        $ ;toliteral([1, "foo", [N20, E_DIV]])
        => "[1, \\"foo\\", [N20, E_DIV]]"
        """
    )


def test_tostr(connect: Connect) -> None:
    connect().cram(
        """
        $ ;tostr([17])
        => "17"
        $ ;tostr([1.0/3.0])
        => "0.3333333333333333"
        $ ;tostr([N17])
        => "N17"
        $ ;tostr(["foo"])
        => "foo"
        $ ;tostr([[1, 2]])
        => "[list]"
        $ ;tostr([E_PERM])
        => "Permission denied"
        $ ;tostr(["3 + 4 = ", 3 + 4])
        => "3 + 4 = 7"
        """
    )