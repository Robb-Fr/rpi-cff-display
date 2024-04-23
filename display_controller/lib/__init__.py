from typing import List
import logging
import os

MAX_NB_COLS = 4
MAX_DISPLAYED_LINES = 5


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
            + " "
            + line_direction
            + " "
            + cols[2]
            + ("" if cols[3] == "0" else "+" + cols[3])
        )
        logging.info(f"appending {to_append}")
        to_display.append(to_append)
    return to_display
