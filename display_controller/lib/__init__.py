from typing import List
import logging
import os
from PIL import Image, ImageFont, ImageDraw
import sys

MAX_NB_COLS = 4
MAX_DISPLAYED_LINES = 5

picdir = os.path.join(
    os.path.dirname(os.path.dirname(os.path.realpath(__file__))), "pic"
)
font = ImageFont.truetype(os.path.join(picdir, "Menlo.ttc"), 31)

libdir = os.path.join(
    os.path.dirname(os.path.dirname(os.path.realpath(__file__))), "lib"
)
if os.path.exists(libdir):
    sys.path.append(libdir)


def parse_api_result(result_filepath: str) -> List[str]:
    if not os.path.isfile(result_filepath):
        raise IOError(f"could not find the file {result_filepath}")

    # Open the file and read its content.
    with open(result_filepath) as f:
        content = f.read().splitlines()

    to_display = []
    for line in content[:MAX_DISPLAYED_LINES]:
        cols = line.split("\t")
        logging.info(cols)
        if len(cols) != MAX_NB_COLS:
            raise ValueError(f"the file contains {len(cols)} instead of {MAX_NB_COLS}")
        line_direction = cols[1]
        if len(line_direction) > 8:
            line_direction = cols[1][:3] + ".." + cols[1][-3:]
        to_append = (
            cols[0]
            + " " * (3 - len(cols[0]))
            + " "
            + line_direction
            + " "
            + cols[2]
            + ("" if cols[3] == "0" else "+" + cols[3])
        )
        logging.info(f"appending {to_append}")
        to_display.append(to_append)
    return to_display


def create_to_display_image(
    text_to_display: List[str], width: int, height: int, font: ImageFont.FreeTypeFont
) -> Image.Image:
    # Drawing the next transport departures
    logging.info("Drawing the next transport departures...")
    Bimage = Image.new("1", (width, height), 255)  # 255: clear the frame
    draw = ImageDraw.Draw(Bimage)
    LINE_HEIGHT = 2
    PADDING_WITH_LINE = 16
    FONT_SIZE = font.size
    for i, s in enumerate(text_to_display):
        draw.text(
            (0, i * (FONT_SIZE + LINE_HEIGHT + PADDING_WITH_LINE * 2)),
            s,
            font=font,
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
    return Bimage
