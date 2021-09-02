
from .conftest import Connect


def test_simple_rhai(connect: Connect):
    c1 = connect()
    c2 = connect()

    c1.send(";40 + 2")
    c1.expect_lines_exact("42")

    c1.send(";let x = 20")
    c1.send(";x")
    c1.expect_lines_exact("20")

    # Verify variables don't leak between connections
    c2.send(";x")
    c2.expect_exact('ErrorVariableNotFound("x", 1:1)')