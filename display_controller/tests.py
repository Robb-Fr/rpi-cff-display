import unittest
import os
from lib import create_to_display_image, parse_api_result, font
from PIL import Image, ImageChops

TEST_RESULT_FILENAME = "api_result_test.tsv"
TEST_IMAGE_FILENAME = "test_display.bmp"
IMAGE_WIDTH = 400
IMAGE_HEIGHT = 300

test_result_filepath = os.path.join(
    os.path.dirname(os.path.dirname(os.path.realpath(__file__))),
    "api_fetcher",
    TEST_RESULT_FILENAME,
)

test_image_filepath = os.path.join(
    os.path.dirname(os.path.realpath(__file__)), TEST_IMAGE_FILENAME
)


class TestDisplayController(unittest.TestCase):
    def test_parse_result(self):
        expected = [
            "6   Gen..age 10:46+1",
            "3   Gra..tti 10:46+1",
            "9   Ver..urs 10:46",
            "6   Ver..age 10:47",
            "10  Gen..ive 10:47",
        ]
        self.assertEqual(parse_api_result(test_result_filepath), expected)

    def test_generate_image(self):
        to_display = parse_api_result(test_result_filepath)

        test_image = create_to_display_image(
            to_display, IMAGE_WIDTH, IMAGE_HEIGHT, font
        )

        with Image.open(test_image_filepath) as expected_image:
            diff = ImageChops.difference(test_image, expected_image)
            self.assertIsNone(diff.getbbox())


if __name__ == "__main__":
    unittest.main()
