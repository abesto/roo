"""
https://www.sindome.org/moo-manual.html#fundamental-operations-on-objects
Manipulating Objects / Fundamental Operations on Objects
"""

from .conftest import Connect


def test_create_wizard(connect: Connect):
    connect().cram(
        """
        $ ;create(N0, N0)
        => N2
        $ ;create(N0, N0)
        => N3
        """
    )


def test_create_nonwizard(connect: Connect):
    player = "N2"
    fertile = "N3"

    # Setup: create non-wizard player and a fertile object, then drop to the player permissions
    connect().cram(
        f"""
        $ ;create(Cnothing, Cnothing)
        => {player}
        $ ;create(Cnothing, Cnothing)
        => {fertile}
        $ ;{fertile}.f
        => false
        $ ;{fertile}.f = true
        => true
        """
    )

    # Happy paths
    connect().cram(
        f"""
        $ ;set_task_perms({player})
        $ ;create(Cnothing)
        => N4
        $ ;create(Cnothing, {player})
        => N5
        $ ;{fertile}.f
        => true
        $ ;create({fertile})
        => N6
        $ ;create({fertile}, {player})
        => N7
        """
    )

    # Bad parent
    connect().cram(
        """
        $ ;set_task_perms({player})
        $ ;create(N1)
        !! E_PERM
        """
    )


def test_parent(connect: Connect):
    connect().cram(
        """
        $ ;create(N0, N0)
        => N2
        $ ;create(N2, N0)
        => N3
        $ ;parent(N3)
        => N2
        """
    )


def test_chparent(connect: Connect):
    connect().cram(
        """
        $ ;create(N0, N0)
        => N2
        $ ;create(N0, N0)
        => N3
        $ ;parent(N2)
        => N0
        $ ;chparent(N2, N3)
        $ ;parent(N2)
        => N3
        """
    )
