#!/usr/bin/python3
# -*- coding:utf-8 -*-
import os
import logging
from . import picdir
from .lib import parse_api_result
from waveshare_epd import epd4in2
from PIL import Image, ImageDraw, ImageFont

logging.basicConfig(level=logging.DEBUG)

RESULT_FILENAME = "api_result.tsv"

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

    font31 = ImageFont.truetype(os.path.join(picdir, "Menlo.ttc"), 31)

    # Drawing the next transport departures
    logging.info("Drawing the next transport departures...")
    Bimage = Image.new("1", (epd.width, epd.height), 255)  # 255: clear the frame
    draw = ImageDraw.Draw(Bimage)
    LINE_HEIGHT = 2
    PADDING_WITH_LINE = 16
    FONT_SIZE = 31
    for i, s in enumerate(to_display):
        draw.text(
            (0, i * (FONT_SIZE + LINE_HEIGHT + PADDING_WITH_LINE * 2)),
            s,
            font=font31,
            fill=0,
        )
        draw.rectangle(
            (
                20,
                (FONT_SIZE + PADDING_WITH_LINE)
                + i * (LINE_HEIGHT + PADDING_WITH_LINE * 2 + FONT_SIZE),
                400 - 20,
                (FONT_SIZE + PADDING_WITH_LINE + LINE_HEIGHT)
                + i * (LINE_HEIGHT + PADDING_WITH_LINE * 2 + FONT_SIZE),
            ),
            fill=0,
        )
    epd.display(epd.getbuffer(Bimage))

    logging.info("Goto Sleep...")
    epd.sleep()

except IOError as e:
    logging.info(e)

except KeyboardInterrupt:
    logging.info("ctrl + c:")
    epd4in2.epdconfig.module_exit(cleanup=True)  # type: ignore
    exit()
