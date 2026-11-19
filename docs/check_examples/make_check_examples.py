"""Example pictures for the guided point check (src/assets/ck_*.jpg).

Source: our own standing scan stand_L2 (18 Nov 2026, left foot), its contact map ("Peta kontak kiri.jpg",
written by `scan_cli pictures`) and the app's own contact-based Clarke points for that scan, which
follow Pita-Fernandez et al. 2015 (J Epidemiol 25(2):148-154): line 1 touches the inner edge of the
ball of the foot and of the heel; line 2 runs from the ball point to the deepest point of the inner
arch curve of the pressed skin.

For each point (left foot, and mirrored with suffix _r for the right foot): one picture with the right place (tick) and one with a common mistake (cross).
Rebuild:  python docs/check_examples/make_check_examples.py <Peta kontak kiri.jpg>
"""
import os, sys
from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "..", "src", "assets")
SRC = sys.argv[1] if len(sys.argv) > 1 else r"D:\SmartFoot_FrameCheck\ref\L2\Peta kontak kiri.jpg"

# Clarke points of stand_L2 in full-scan pixels (scan_cli measure: clarke_source "kontak", 43.8 deg)
FULL = {"fore": (910.9, 852.8), "heel": (1172.9, 2706.9), "arch": (1578.9, 1379.1)}
# common mistakes, full-scan pixels: big toe instead of the ball, back of the heel instead of its
# inner edge, the arch point on the outer outline instead of the edge of the green pressed skin
WRONG = {"fore": (1150, 230), "heel": (1330, 3050), "arch": (985, 1379)}

ORANGE, GREEN, RED, WHITE = (255, 183, 3), (31, 157, 107), (201, 59, 59), (255, 255, 255)
S = 1275 / 2550          # contact map is half the scan size
BOX = (180, 60, 1150, 1600)  # crop round the foot, contact-map pixels
W = 300                  # output width


def mark(d, x, y, ok, r=26):
    d.ellipse([x - r, y - r, x + r, y + r], fill=GREEN if ok else RED, outline=WHITE, width=3)
    if ok:
        d.line([(x - 12, y), (x - 3, y + 10), (x + 13, y - 10)], fill=WHITE, width=6, joint="curve")
    else:
        d.line([(x - 9, y - 9), (x + 9, y + 9)], fill=WHITE, width=6)
        d.line([(x + 9, y - 9), (x - 9, y + 9)], fill=WHITE, width=6)


def picture(base, step, ok, mirror=False):
    im = base.transpose(Image.FLIP_LEFT_RIGHT) if mirror else base.copy()
    d = ImageDraw.Draw(im)
    k = W / (BOX[2] - BOX[0])
    P = {n: ((x * S - BOX[0]) * k, (y * S - BOX[1]) * k) for n, (x, y) in FULL.items()}
    if not ok:
        x, y = WRONG[step]
        P[step] = ((x * S - BOX[0]) * k, (y * S - BOX[1]) * k)
    if mirror:  # right foot: the same example seen mirrored, badge stays readable
        P = {n: (im.width - x, y) for n, (x, y) in P.items()}
    order = ["fore", "heel", "arch"]
    done = order[: order.index(step) + 1]
    # lines once both of their points are placed
    if "heel" in done:
        f, h = P["fore"], P["heel"]
        dx, dy = h[0] - f[0], h[1] - f[1]
        d.line([(f[0] - dx * 0.15, f[1] - dy * 0.15), (h[0] + dx * 0.08, h[1] + dy * 0.08)], fill=ORANGE, width=3)
    if "arch" in done:
        d.line([P["fore"], P["arch"]], fill=ORANGE, width=3)
    for n in done:
        x, y = P[n]
        r = 12 if n == step else 7
        d.ellipse([x - r, y - r, x + r, y + r], outline=ORANGE if n == step else (200, 160, 40), width=4 if n == step else 2)
    mark(d, W - 34, 34, ok)
    return im


def main():
    cm = Image.open(SRC).convert("RGB").crop(BOX)
    base = cm.resize((W, round(cm.height * W / cm.width)), Image.LANCZOS)
    for step in ("fore", "heel", "arch"):
        for ok in (True, False):
            for mirror in (False, True):
                p = os.path.join(OUT, f"ck_{step}_{'ok' if ok else 'no'}{'_r' if mirror else ''}.jpg")
                picture(base, step, ok, mirror).save(p, quality=84)
                print(p)


if __name__ == "__main__":
    main()
