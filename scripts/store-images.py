"""Draws the Microsoft Store listing images: the R of the app icon on cobalt, over a wall of tiles
in the shades of cobalt, and the Store name in Segoe UI Light where an image may carry it. Run with
`python scripts/store-images.py` (needs Pillow, on Windows for the font); writes dist/store-images.

Images that must not carry the product name (the 16:9 super hero, the Xbox featured promo square)
show the R alone.
"""

import importlib.util
import os
import random
import re
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location("uwp_icons", ROOT / "scripts" / "uwp-icons.py")
icons = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(icons)


def store_name() -> str:
    """The name the Store reserved, which is not part of the repository: it comes from
    `scripts/store.local.ps1` (or RUBI_STORE_NAME), the same file the packager reads."""
    if os.environ.get("RUBI_STORE_NAME"):
        return os.environ["RUBI_STORE_NAME"]
    local = Path(__file__).resolve().with_name("store.local.ps1")
    if local.exists():
        found = re.search(r"\$storeDisplayName\s*=\s*'([^']+)'", local.read_text(encoding="utf-8-sig"))
        if found:
            return found.group(1)
    raise SystemExit("需要 scripts/store.local.ps1（或环境变量 RUBI_STORE_NAME）里的商店名。")


NAME = store_name()
FONT = Path("C:/Windows/Fonts/segoeuil.ttf")
# Tiles a little darker and lighter than cobalt, as a Start screen seen from afar.
SHADES = [(0x00, 0x46, 0xD6), (0x00, 0x4A, 0xE0), (0x0A, 0x5C, 0xF5), (0x14, 0x66, 0xFA)]


def letter(pen, left, top, height):
    """The white R, `height` tall, its box's top-left at (`left`, `top`)."""
    raw = [(x, y) for block in icons.BLOCKS for x, y in block]
    raw_left, raw_top = min(p[0] for p in raw), min(p[1] for p in raw)
    scale = height / (max(p[1] for p in raw) - raw_top)
    for block in icons.BLOCKS:
        pen.polygon([(left + (x - raw_left) * scale, top + (y - raw_top) * scale) for x, y in block], fill=icons.WHITE)


def letter_width(height):
    raw = [(x, y) for block in icons.BLOCKS for x, y in block]
    return height * (max(p[0] for p in raw) - min(p[0] for p in raw)) / (max(p[1] for p in raw) - min(p[1] for p in raw))


def tiles(image, cell, seed):
    """A wall of square tiles over the whole image, a few of them in other shades of cobalt."""
    pen = ImageDraw.Draw(image)
    gap = max(2, cell // 24)
    chance = random.Random(seed)
    for top in range(0, image.height, cell):
        for left in range(0, image.width, cell):
            if chance.random() < 0.35:
                shade = chance.choice(SHADES)
                pen.rectangle([left + gap, top + gap, left + cell - gap - 1, top + cell - gap - 1], fill=shade)


def plate(width, height, cell=None, seed=0):
    image = Image.new("RGBA", (width, height), icons.COBALT)
    if cell:
        tiles(image, cell, seed)
    return image


def title(pen, text, x, y, size, anchor):
    pen.text((x, y), text, font=ImageFont.truetype(str(FONT), size), fill=icons.WHITE, anchor=anchor)


def mark(width, height, letter_height, name_size=0, cell=None, seed=0, across=False, rise=0.5):
    """The R, and the name under it (or beside it, `across`), as one group whose middle sits at
    `rise` of the height."""
    scale = icons.OVERSAMPLE
    image = plate(width * scale, height * scale, cell and cell * scale, seed)
    pen = ImageDraw.Draw(image)
    h, w = letter_height * scale, letter_width(letter_height) * scale
    size = name_size * scale
    middle = height * scale * rise
    if not name_size:
        letter(pen, (width * scale - w) / 2, middle - h / 2, h)
    elif across:
        font = ImageFont.truetype(str(FONT), size)
        text_width = pen.textlength(NAME, font=font)
        gap = h * 0.45
        left = (width * scale - (w + gap + text_width)) / 2
        letter(pen, left, middle - h / 2, h)
        title(pen, NAME, left + w + gap, middle, size, "lm")
    else:
        gap = h * 0.3
        top = middle - (h + gap + size) / 2
        letter(pen, (width * scale - w) / 2, top, h)
        title(pen, NAME, width * scale / 2, top + h + gap, size, "mt")
    return image.resize((width, height), Image.LANCZOS)


def main():
    out = ROOT / "dist" / "store-images"
    out.mkdir(parents=True, exist_ok=True)
    images = {
        # Store display images: the app's tile, as in the package.
        "store-tile-300.png": icons.draw(300, 300, 177),
        "store-tile-150.png": icons.draw(150, 150, 88),
        "store-tile-71.png": icons.draw(71, 71, 42),
        # Xbox poster (9:16 slot), and box art.
        "poster-1440x2160.png": mark(1440, 2160, 560, 190, cell=240, seed=1, rise=0.42),
        "poster-720x1080.png": mark(720, 1080, 280, 95, cell=120, seed=1, rise=0.42),
        "box-art-2160.png": mark(2160, 2160, 760, 220, cell=270, seed=2),
        "box-art-1080.png": mark(1080, 1080, 380, 110, cell=135, seed=2),
        # 16:9 super hero: no product name.
        "super-hero-3840x2160.png": mark(3840, 2160, 900, cell=240, seed=3, rise=0.45),
        "super-hero-1920x1080.png": mark(1920, 1080, 450, cell=120, seed=3, rise=0.45),
        # Xbox: the title inside the top three quarters.
        "xbox-branded-key-art-584x800.png": mark(584, 800, 220, 72, cell=73, seed=4, rise=0.36),
        "xbox-titled-hero-1920x1080.png": mark(1920, 1080, 340, 200, cell=120, seed=5, across=True, rise=0.4),
        # Xbox featured promo square: no product name.
        "xbox-featured-promo-1080.png": mark(1080, 1080, 420, cell=135, seed=6),
    }
    for name, image in images.items():
        image.convert("RGB").save(out / name, optimize=True)
        print(f"{name}: {image.size[0]}x{image.size[1]}")


if __name__ == "__main__":
    main()
