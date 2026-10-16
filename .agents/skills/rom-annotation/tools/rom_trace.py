"""Recursive-descent Z80 code tracer for ROM images.

Finds which bytes are instructions by following control flow from entry
points, and compares the result with an rtvc-asm listing to show where the
listing decodes code as data (DB), data as code, or code off its real
instruction boundaries. This is the tool behind the first, mechanical pass
over the TVC12 listings.

    python3 rom_trace.py IMAGE --origin C000H [--offset 0 --length 1000H]
        --entry C21CH --entry ... [--counted-table C00DH ...]
        [--word-table ADDR:COUNT ...] [--listing roms/X.asm]

Built in: RST 08H takes one inline error byte (and only continues for 00H),
RST 18H takes a bytecode run ending with a bit-7 byte (operation 5 takes one
more byte), RST 30H takes one function byte. These are the TVC BASIC 1.2
conventions; other ROMs may need different handlers (Image.rst_handlers) and
inline-argument calls (Image.inline_calls). HALT falls through (EI; HALT
waits for an interrupt in TVC tape code).

Overlapping instructions (the "LD A,n" / "LD BC,nn" skip idiom) are traced
on both paths; an rtvc-asm listing shows the hiding bytes as DB.
"""

import argparse
import re
import subprocess
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from z80_flow import decode  # noqa: E402


def rst08(img, nxt):
    img.mark_inline(nxt, 1, "RST 08H error code")
    return nxt + 1 if img.mem(nxt) == 0 else None


def rst18(img, nxt):
    start = nxt
    while True:
        b = img.mem(nxt)
        if b is None:
            return None
        nxt += 1
        if (b & 0x7F) == 5:
            nxt += 1
        if b & 0x80:
            break
    img.mark_inline(start, nxt - start, "RST 18H bytecode")
    return nxt


def rst30(img, nxt):
    img.mark_inline(nxt, 1, "RST 30H function")
    return nxt + 1


class Image:
    def __init__(self, data, org):
        self.data, self.org = data, org
        self.code = {}           # addr -> instruction length
        self.inline = {}         # addr -> (length, reason)
        self.callers = {}        # target -> set of branch sources
        self.conflicts = []
        self.work = []
        self.inline_calls = {}   # callee -> handler(img, next_addr) -> next or None
        self.rst_handlers = {0x08: rst08, 0x18: rst18, 0x30: rst30}

    def mem(self, a):
        o = a - self.org
        return self.data[o] if 0 <= o < len(self.data) else None

    def inside(self, a):
        return a is not None and 0 <= a - self.org < len(self.data)

    def add(self, a):
        if self.inside(a):
            self.work.append(a)

    def mark_inline(self, a, n, why):
        if n > 0:
            self.inline[a] = (n, why)

    def inline_owner(self, a):
        for s, (n, _) in self.inline.items():
            if s <= a < s + n:
                return s
        return None

    def run(self):
        while self.work:
            self.flow(self.work.pop())

    def flow(self, a):
        while self.inside(a) and a not in self.code:
            r = decode(self.mem, a)
            if r is None:
                return
            n, kind, t = r
            if any(self.inline_owner(a + k) is not None for k in range(n)):
                self.conflicts.append(a)
                return
            self.code[a] = n
            nxt = a + n
            if kind in ("jp", "cj", "call"):
                if t is not None:
                    self.callers.setdefault(t, set()).add(a)
                    self.add(t)
                if kind == "call" and t in self.inline_calls:
                    nxt = self.inline_calls[t](self, nxt)
                    if nxt is None:
                        return
                if kind == "jp":
                    return
            elif kind == "rst":
                handler = self.rst_handlers.get(t)
                if handler:
                    nxt = handler(self, nxt)
                    if nxt is None:
                        return
                elif t == 0x00:
                    return
            elif kind in ("ret", "ind"):
                return
            a = nxt

    def word(self, a):
        return self.mem(a) | self.mem(a + 1) << 8


def listing_lines(path):
    """(line, addr, len, is_db, source) for every emitted line of a listing."""
    asm = os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", "target", "release", "rtvc-asm")
    toml = subprocess.run([asm, "--format", "toml", path], capture_output=True, text=True, check=True).stdout
    out = []
    for m in re.finditer(r'\[\[lines\]\]\nline = (\d+)\naddr = 0x([0-9A-F]+)\nlen = (\d+)\nsource = "(.*)"', toml):
        src = m.group(4).strip()
        if src.startswith("ORG") or int(m.group(3)) == 0:
            continue
        out.append((int(m.group(1)), int(m.group(2), 16), int(m.group(3)),
                    src.split()[0] in ("DB", "DW", "DS", "DEFB", "DEFW", "DEFS"), src))
    return out


def compare(img, path):
    lines = listing_lines(path)
    owner = {}
    for ln, a, n, db, src in lines:
        for x in range(a, a + n):
            owner[x] = (ln, a, db)
    covered = set()
    for a, n in img.code.items():
        covered.update(range(a, a + n))
    for a, (n, _) in img.inline.items():
        covered.update(range(a, a + n))
    for a in sorted(img.code):
        o = owner.get(a)
        if o and o[2]:
            print(f"{a:04X}  line {o[0]}: traced code inside a data line")
        elif o and o[1] != a:
            print(f"{a:04X}  line {o[0]}: traced instruction starts inside a listing instruction")
    for a in sorted(img.inline):
        o = owner.get(a)
        if o and not o[2]:
            print(f"{a:04X}  line {o[0]}: inline data ({img.inline[a][1]}) decoded as code")
    untraced = [(ln, a, src) for ln, a, n, db, src in lines
                if not db and not any(x in covered for x in range(a, a + n))]
    print(f"{len(untraced)} listing instructions not reached by the trace "
          "(data decoded as code, or code reached only indirectly)")


def number(text):
    text = text.strip()
    return int(text[:-1], 16) if text.upper().endswith("H") else int(text, 0)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("image")
    ap.add_argument("--origin", required=True)
    ap.add_argument("--offset", default="0")
    ap.add_argument("--length")
    ap.add_argument("--entry", action="append", default=[])
    ap.add_argument("--counted-table", action="append", default=[],
                    help="count byte followed by that many words")
    ap.add_argument("--word-table", action="append", default=[], help="ADDR:COUNT")
    ap.add_argument("--listing")
    args = ap.parse_args()
    with open(args.image, "rb") as f:
        data = f.read()
    off = number(args.offset)
    length = number(args.length) if args.length else len(data) - off
    img = Image(data[off:off + length], number(args.origin))
    for e in args.entry:
        img.add(number(e))
    for t in args.counted_table:
        a = number(t)
        for i in range(img.mem(a)):
            img.add(img.word(a + 1 + 2 * i))
    for t in args.word_table:
        a, count = t.split(":")
        a = number(a)
        for i in range(number(count)):
            img.add(img.word(a + 2 * i))
    img.run()
    print(f"{len(img.code)} instructions, {len(img.inline)} inline data runs, "
          f"{len(img.conflicts)} conflicts")
    if args.listing:
        compare(img, args.listing)


if __name__ == "__main__":
    main()
