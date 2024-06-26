from unittest import TestCase
from demoparser2 import DemoParser
import pandas as pd
import unittest

class SignatureTest(TestCase):

    def test_parse_header_signature(self):
        parser = DemoParser("tests/data/test.dem")
        header = parser.parse_header()
        self.assertTrue(isinstance(header, dict))
        for key, value in header.items():
            self.assertTrue(isinstance(key, str))
            self.assertTrue(isinstance(value, str))


if __name__ == '__main__':
    unittest.main()