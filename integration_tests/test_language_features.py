"""
Test stuff equivalent to what's part of the Moo language syntax,
and basic Rhai integration
"""

from .conftest import Connect


def test_simple_rhai(connect: Connect) -> None:
    c = connect()
    c.cram(
        """
    $ ;40 + 2
    => 42
    $ ;"foo"
    => "foo"
    """
    )


def test_rhai_variables(connect: Connect) -> None:
    c = connect()
    c.cram(
        """
    $ ;let x = 20
    $ ;x
    => 20
    """
    )


def test_object_N_notation(connect: Connect) -> None:
    connect().cram(
        """
        $ ;N42
        => N42
        """
    )


def test_rhai_variable_isolation(connect: Connect) -> None:
    c1 = connect()
    c2 = connect()

    c1.send(";let x = 'foo'")
    c2.cram(
        """
    $ ;x
    Variable not found: x (line 1, position 1) in call to function eval (line 1, position 16)
    """
    )


def test_equality(connect: Connect) -> None:
    # Differences from Moo:
    #   In Moo `==` is case INsensitive. I have opinions about that, so it's NOT the case here.
    #   In Moo 3 != 3.0, it IS here
    connect().cram(
        """
        $ ;3 == 4
        => false
        $ ;3 != 4
        => true
        $ ;3 == 3.0
        => true
        $ ;3 == 3.0001
        => false
        $ ;"foo" == "foo"
        => true
        $ ;"foo" == "Foo"
        => false
        $ ;N34 != N34
        => false
        $ ;N10 == N10
        => true
        $ ;N10 == N34
        => false
        $ ;[1, N34, "foo"] == [  1,N34,   "foo" ]
        => true
        $ ;[1, 2] == [1, 3]
        => false
        $ ;E_DIV == E_TYPE
        => false
        $ ;E_INVARG == E_INVARG
        => true
        $ ;3 != "foo"
        => true
        """
    )


def test_ordering(connect: Connect) -> None:
    # Differences from Moo: all types can be compared.
    # Comparison of incompatible types will always evalate to `false`.
    connect().cram(
        """
        $ ;3 < 4
        => true
        $ ;3 < 4.0
        => true
        $ ;N34 >= N32
        => true
        $ ;"foo" <= "Boo"
        => false
        $ ;E_DIV > E_TYPE
        => true
        $ ;300 > E_INVIND
        => false
        $ ;3 > "foo"
        => false
        $ ;3 < "foo"
        => false
        $ ;[1] > 0
        => false
        $ ;[1] < 0
        => false
        """
    )


def test_spread_assignment(connect: Connect) -> None:
    connect().cram(
        """
    $ ;lets [a, b] = [1, 2]
    $ ;a
    => 1
    $ ;b
    => 2
    """
    )

    # TODO the first test should really tell us it's E_INVARG
    do = "let b = 17; let c = 17; let e = 17; lets [a, OPT_b, c = 8, REST_d, OPT_e = 9, f] = args; [a, b, c, d, e, f]"
    connect().cram(
        f"""
        $ ;let args = [1]; {do}
        !! E_INVARG
        $ ;let args = [1, 2]; {do}
        => [1, 17, 8, [], 9, 2]
        $ ;let args = [1, 2, 3]; {do}
        => [1, 2, 8, [], 9, 3]
        $ ;let args = [1, 2, 3, 4]; {do}
        => [1, 2, 3, [], 9, 4]
        $ ;let args = [1, 2, 3, 4, 5]; {do}
        => [1, 2, 3, [], 4, 5]
        $ ;let args = [1, 2, 3, 4, 5, 6]; {do}
        => [1, 2, 3, [4], 5, 6]
        $ ;let args = [1, 2, 3, 4, 5, 6, 7]; {do}
        => [1, 2, 3, [4, 5], 6, 7]
        $ ;let args = [1, 2, 3, 4, 5, 6, 7, 8]; {do}
        => [1, 2, 3, [4, 5, 6], 7, 8]
     """
    )


def test_corified_references(connect: Connect) -> None:
    connect().cram(
        """
        $ ;add_property(N0, "test", 42, [N0, ""])
        $ ;Ctest
        => 42
        """
    )
