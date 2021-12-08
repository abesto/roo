from .conftest import Connect


def test_highest_object_number(connect: Connect) -> None:
    connect().cram(
        """
        $ ;create(N0, N0)
        => N2
        $ ;create(N0, N0)
        => N3
        $ ;get_highest_object_number()
        => 3
        """
    )

    connect().cram(
        """
        $ ;get_highest_object_number()
        => 3
        """
    )


def test_valid(connect: Connect) -> None:
    connect().cram(
        """
        $ ;valid(toobj(-1))
        => false
        """
    )

    connect().cram(
        """
        $ ;valid(N0)
        => true
        """
    )

    connect().cram(
        """
        $ ;valid(create(N0, N0))
        => true
        """
    )
