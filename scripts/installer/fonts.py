#!/usr/bin/env python3
import argparse
import math
import struct
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


def jis_x_0208_codepoints():
    codepoints = set(range(0x20, 0x7F))
    codepoints.update(range(0xFF61, 0xFFA0))
    for lead in range(0xA1, 0xFF):
        for trail in range(0xA1, 0xFF):
            try:
                text = bytes((lead, trail)).decode("euc_jp")
            except UnicodeDecodeError:
                continue
            if len(text) == 1:
                codepoints.add(ord(text))
    return sorted(codepoints)


def set_regular_weight(font):
    axes = font.get_variation_axes()
    if len(axes) != 1:
        raise ValueError("expected the pinned Noto Sans JP font to have one variable axis")
    values = [axis["default"] for axis in axes]
    values[0] = 400
    font.set_variation_by_axes(values)


def make_font(source, output, size, codepoints):
    font = ImageFont.truetype(str(source), size=size)
    set_regular_weight(font)
    row_bytes = math.ceil(size / 8)
    bytes_per_glyph = row_bytes * size
    bitmaps = bytearray()
    unicode_table = bytearray()
    draw_image = Image.new("L", (size, size), 0)
    draw = ImageDraw.Draw(draw_image)

    for codepoint in codepoints:
        character = chr(codepoint)
        draw_image.paste(0, (0, 0, size, size))
        left, top, right, bottom = draw.textbbox((0, 0), character, font=font)
        x = (size - (right - left)) // 2 - left
        y = (size - (bottom - top)) // 2 - top
        draw.text((x, y), character, font=font, fill=255)

        for row in range(size):
            packed = 0
            for column in range(size):
                if draw_image.getpixel((column, row)) >= 128:
                    packed |= 0x80 >> (column % 8)
                if column % 8 == 7:
                    bitmaps.append(packed)
                    packed = 0
            if size % 8:
                bitmaps.append(packed)
        unicode_table.extend(character.encode("utf-8"))
        unicode_table.append(0xFF)

    header = struct.pack(
        "<8I",
        0x864AB572,
        0,
        32,
        1,
        len(codepoints),
        bytes_per_glyph,
        size,
        size,
    )
    output.write_bytes(header + bitmaps + unicode_table)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("font", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    codepoints = jis_x_0208_codepoints()
    for size in (16, 24, 32):
        make_font(args.font, args.output / f"noto-jp-{size}.psf", size, codepoints)
    print(f"Generated three PSF2 fonts for {len(codepoints)} codepoints")


if __name__ == "__main__":
    main()