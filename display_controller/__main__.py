#!/usr/bin/python3
# -*- coding:utf-8 -*-
import os
import logging
from .lib import create_to_display_image, parse_api_result, font
from waveshare_epd import epd4in2

logging.basicConfig(level=logging.DEBUG)

TEST_IMAGE_GEN = False
RESULT_FILENAME = "api_result.tsv"
if TEST_IMAGE_GEN:
    RESULT_FILENAME = "api_result_test.tsv"

result_filepath = os.path.join(
    os.path.dirname(os.path.dirname(os.path.realpath(__file__))),
    "api_fetcher",
    RESULT_FILENAME,
)

try:
    to_display = parse_api_result(result_filepath)
except (IOError, ValueError) as e:
    logging.error(e)
    exit(1)

try:
    logging.info("Starting to display the next departures")

    epd = epd4in2.EPD()
    logging.info("init and Clear")
    epd.init()
    epd.Clear()

    Bimage = create_to_display_image(to_display, epd.width, epd.height, font)
    if TEST_IMAGE_GEN:
        Bimage.save("display_controller/test_display.bmp")
    epd.display(epd.getbuffer(Bimage))

    logging.info("Goto Sleep...")
    epd.sleep()

except IOError as e:
    logging.info(e)

except KeyboardInterrupt:
    logging.info("ctrl + c:")
    epd4in2.epdconfig.module_exit(cleanup=True)  # type: ignore
    exit()
