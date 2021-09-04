"""
Manipulating MOO Values / General Operations Applicable to all Values
https://www.sindome.org/moo-manual.html#properties-on-objects
"""

from .conftest import Connect

def test_type_of(connect: Connect) -> None:
    connect().cram("""
    $ ;type_of(3)
    => i64
    $ ;type_of("foobar")
    => string
    $ ;type_of('c')
    => char
    $ ;type_of(3.14)
    => f64
    $ ;type_of([1, "foo"])
    => array
    $ ;type_of(N0)
    => Object
    $ ;type_of(E_INVARG)
    => Error
    """)