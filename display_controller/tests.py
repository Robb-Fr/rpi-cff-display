import unittest
import os
from lib import parse_api_result

TEST_RESULT_FILENAME = "api_result_test.tsv"


class TestParse(unittest.TestCase):
    def test_parse_result(self):
        test_result_filepath = os.path.join(
            os.path.dirname(os.path.dirname(os.path.realpath(__file__))),
            "api_fetcher",
            TEST_RESULT_FILENAME,
        )
        expected = [
            "6 Gen..age 10:46+1",
            "3 Gra..tti 10:46+1",
            "9 Ver..urs 10:46",
            "6 Ver..age 10:47",
            "10 Gen..ive 10:47",
        ]
        self.assertEqual(parse_api_result(test_result_filepath), expected)


if __name__ == "__main__":
    unittest.main()
