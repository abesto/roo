"""
Manipulating MOO Values / Operations on Strings
https://www.sindome.org/moo-manual.html#operations-on-strings
"""

from .conftest import Connect


def test_string_hash(connect: Connect) -> None:
    # Difference from Moo: we use SHA512 instead of MD5
    connect().cram(
        """
        $ ;string_hash("wheee")
        => "6a659751d5b9b64921a307f505d674581e1446b938e68398672573c0bcc43ec4c2c5734784b27934f9e37c31d25f8296145a02513d77c2004b5622873b185b15"
        """
    )