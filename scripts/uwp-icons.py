"""Draws the UWP package images from the Android launcher icon (方案 B): a white R of right-angled
blocks on cobalt. Run with `python scripts/uwp-icons.py` (needs Pillow); writes windows/uwp/Assets.

The package is registered from loose files without a resource index, so each image is one file at
the largest size the shell asks for, and the shell scales it down.
"""

from pathlib import Path

from PIL import Image, ImageDraw

COBALT = (0x00, 0x50, 0xEF, 255)
WHITE = (255, 255, 255, 255)
# Drawn this many times larger, then scaled down, for clean edges.
OVERSAMPLE = 4

# The R as in ic_launcher_foreground.xml, in its 108-unit box; each image centres and sizes it.
BLOCKS = [
    [(28, 22), (40, 22), (40, 78), (28, 78)],
    [(28, 22), (68, 22), (68, 34), (28, 34)],
    [(56, 22), (68, 22), (68, 54), (56, 54)],
    [(28, 44), (68, 44), (68, 54), (28, 54)],
    [(44, 54), (58, 54), (74, 78), (60, 78)],
]


def draw(width, height, letter_height, plate=True):
    """An image `width` x `height` with the R `letter_height` tall in its middle."""
    big = Image.new("RGBA", (width * OVERSAMPLE, height * OVERSAMPLE), COBALT if plate else (0, 0, 0, 0))
    pen = ImageDraw.Draw(big)
    raw = [(x, y) for block in BLOCKS for x, y in block]
    raw_left, raw_top = min(p[0] for p in raw), min(p[1] for p in raw)
    raw_right, raw_bottom = max(p[0] for p in raw), max(p[1] for p in raw)
    scale = letter_height * OVERSAMPLE / (raw_bottom - raw_top)
    x0 = (width * OVERSAMPLE - (raw_right - raw_left) * scale) / 2
    y0 = (height * OVERSAMPLE - (raw_bottom - raw_top) * scale) / 2
    for block in BLOCKS:
        pen.polygon([(x0 + (x - raw_left) * scale, y0 + (y - raw_top) * scale) for x, y in block], fill=WHITE)
    return big.resize((width, height), Image.LANCZOS)


def main():
    out = Path(__file__).resolve().parent.parent / "windows" / "uwp" / "Assets"
    out.mkdir(parents=True, exist_ok=True)
    images = {
        # Start list, taskbar, title bar: the R fills most of it, as the launcher icon does.
        "Square44x44Logo.png": draw(176, 176, 104),
        "Square150x150Logo.png": draw(600, 600, 240),
        "Wide310x150Logo.png": draw(1240, 600, 240),
        "StoreLogo.png": draw(200, 200, 118),
        # Over the splash's own cobalt.
        "SplashScreen.png": draw(1240, 600, 240, plate=False),
    }
    for name, image in images.items():
        image.save(out / name, optimize=True)
        print(f"{name}: {image.size[0]}x{image.size[1]}")


if __name__ == "__main__":
    main()
