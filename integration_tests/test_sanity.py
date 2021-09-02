
from .conftest import Connect


def test_echo(connect: Connect):
    c1 = connect()
    c2 = connect()

    c1.send("c1 says hi")
    c1.expect_exact("c1 says hi")

    c2.send("c2 says hi")
    c2.expect_exact("c2 says hi")