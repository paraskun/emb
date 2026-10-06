#!/usr/bin/env python3

import argparse
import math
import sys

import serial

W = H = 8

BRIGHTNESS = 0.35
GAMMA = 2.2
PEAK_HOLD = 12
PEAK_FALL = 0.15
PEAK_COLOR = (0.7, 0.7, 0.9)


def row_color(y):
    t = y / (H - 1)
    return (min(1.0, 2 * t), min(1.0, 2 * (1 - t)), 0.0)


ROW_COLORS = [row_color(y) for y in range(H)]


def to_grb(rgb, level=1.0):
    r, g, b = (int(255 * BRIGHTNESS * (c * level) ** GAMMA + 0.5) for c in rgb)
    return bytes((g, r, b))


def pixel_offset(x, y, serpentine):
    row = H - 1 - y                       # frame rows run top to bottom
    col = W - 1 - x if (serpentine and row % 2) else x
    return (row * W + col) * 3


def main():
    ap = argparse.ArgumentParser(description="CAVA -> 8x8 WS2812 over USB serial")
    ap.add_argument("--port", default="/dev/ttyACM0")
    ap.add_argument("--serpentine", action="store_true",
                    help="remap for zig-zag wiring (skip if the firmware already does it)")
    ap.add_argument("--flip", action="store_true", help="mirror left/right")
    args = ap.parse_args()

    ser = serial.Serial(args.port)        # baud rate is ignored by USB CDC
    src = sys.stdin.buffer
    peaks = [0.0] * W
    hold = [0] * W

    try:
        while True:
            data = src.read(W)            # one CAVA frame = 8 bytes
            if len(data) < W:
                break                     # CAVA exited
            frame = bytearray(W * H * 3)

            for i, v in enumerate(data):
                x = W - 1 - i if args.flip else i
                h = v / 255 * H           # bar height in rows (fractional)

                # bar body; the topmost pixel is dimmed by its fractional part
                for y in range(min(H, math.ceil(h))):
                    level = min(1.0, h - y)
                    off = pixel_offset(x, y, args.serpentine)
                    frame[off:off + 3] = to_grb(ROW_COLORS[y], level)

                # peak dot with hold time and gravity
                if h >= peaks[i]:
                    peaks[i], hold[i] = h, PEAK_HOLD
                elif hold[i]:
                    hold[i] -= 1
                else:
                    peaks[i] = max(0.0, peaks[i] - PEAK_FALL)

                py = min(int(peaks[i]), H - 1)
                if peaks[i] >= 1 and py >= math.ceil(h):
                    off = pixel_offset(x, py, args.serpentine)
                    frame[off:off + 3] = to_grb(PEAK_COLOR)

            ser.write(frame)
    except KeyboardInterrupt:
        pass
    finally:
        try:
            ser.write(bytes(W * H * 3))
        finally:
            ser.close()


if __name__ == "__main__":
    main()
