"""Assemble the annotated ROM listings and compare them with the ROM images.

    python3 scripts/check_rom_listings.py            # all listings
    python3 scripts/check_rom_listings.py D3 page0   # only matching names

Exits non-zero if any listing fails to assemble or differs from its image.
Build rtvc-asm first: cargo build --release --bin rtvc-asm
"""

import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASM = os.path.join(ROOT, "target", "release", "rtvc-asm")

# listing, image, origin, offset and length within the image
LISTINGS = [
    ("roms/TVC12_D4.64K.asm", "roms/TVC12_D4.64K", 0xC000, 0, 0x2000),
    ("roms/TVC12_D3.64K.asm", "roms/TVC12_D3.64K", 0xE000, 0, 0x2000),
    ("roms/TVC12_D7.64K.asm", "roms/TVC12_D7.64K", 0xE000, 0, 0x2000),
] + [
    (f"roms/VT-DOS12-DISK.page{n}.asm", "roms/VT-DOS12-DISK.ROM", 0xC000, n * 0x1000, 0x1000)
    for n in range(4)
]


def main():
    wanted = sys.argv[1:]
    failed = 0
    for listing, image, origin, offset, length in LISTINGS:
        if wanted and not any(w in listing for w in wanted):
            continue
        with open(os.path.join(ROOT, image), "rb") as f:
            expected = f.read()[offset:offset + length]
        with tempfile.TemporaryDirectory() as tmp:
            out = os.path.join(tmp, "out.bin")
            run = subprocess.run([ASM, "--format", "bin", "--origin", f"{origin:04X}H",
                                  os.path.join(ROOT, listing), "-o", out],
                                 capture_output=True, text=True)
            if run.returncode != 0:
                print(f"{listing}: assembler error\n{run.stderr.strip()}")
                failed += 1
                continue
            with open(out, "rb") as f:
                got = f.read()
        if got == expected:
            print(f"{listing}: ok")
            continue
        failed += 1
        diff = next((i for i in range(min(len(got), len(expected))) if got[i] != expected[i]), None)
        if diff is None:
            print(f"{listing}: length {len(got)}, expected {len(expected)}")
        else:
            print(f"{listing}: first difference at {origin + diff:04X}H: "
                  f"{got[diff]:02X}H, expected {expected[diff]:02X}H")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
