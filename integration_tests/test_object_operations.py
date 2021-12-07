"""
https://www.sindome.org/moo-manual.html#fundamental-operations-on-objects
Manipulating Objects / Fundamental Operations on Objects
"""

from .conftest import Connect


def test_create(connect: Connect):
    connect().cram(
        """
        $ ;create(N0, N0)
        => N1
        $ ;create(N0, N0)
        => N2
        """
    )


def test_parent(connect: Connect):
    connect().cram(
        """
        $ ;create(N0, N0)
        => N1
        $ ;create(N1, N0)
        => N2
        $ ;parent(N2)
        => N1
        """
    )


def test_chparent(connect: Connect):
    connect().cram(
        """
        $ ;create(N0, N0)
        => N1
        $ ;create(N0, N0)
        => N2
        $ ;chparent(N1, N2)
        $ ;parent(N1)
        => N2
        """
    )
