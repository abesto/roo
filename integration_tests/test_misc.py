from .conftest import Connect


def test_highest_object_number(connect: Connect) -> None:
    connect().cram(
        """
        $ ;create(N0, N0)
        => N1
        $ ;create(N0, N0)
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


def test_valid(connect: Connect) -> None:
    connect().cram(
        """
        $ ;valid(O(-1))
        => false
        $ ;valid(create(N0, N0))
        => true
        """
    )


