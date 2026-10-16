"""Minimal Z80 instruction length / control-flow decoder for ROM tracing.

decode(mem, addr) -> (length, kind, target). mem(addr) returns a byte or None.
kinds: 'n' normal (HALT included: it falls through), 'jp' unconditional jump,
'cj' conditional jump or DJNZ, 'call' (also conditional), 'ret', 'cret'
(conditional return), 'rst' (target = restart address), 'ind' (JP (HL),
JP (IX), JP (IY)). Lengths were checked against every instruction rtvc-asm
emits for the TVC12 listings.
"""


def decode(mem, a):
    """Return (length, kind, target) for the instruction at address a.

    mem is a callable returning the byte at an address (or None)."""
    op = mem(a)
    if op is None:
        return None
    if op == 0xCB:
        return (2, 'n', None)
    if op == 0xED:
        o2 = mem(a + 1)
        if o2 is None:
            return None
        if o2 in (0x43, 0x4B, 0x53, 0x5B, 0x63, 0x6B, 0x73, 0x7B):
            return (4, 'n', None)
        if o2 in (0x45, 0x4D, 0x55, 0x5D, 0x65, 0x6D, 0x75, 0x7D):
            return (2, 'ret', None)  # RETN / RETI
        return (2, 'n', None)
    if op in (0xDD, 0xFD):
        o2 = mem(a + 1)
        if o2 is None:
            return None
        if o2 == 0xCB:
            return (4, 'n', None)
        if o2 == 0xE9:
            return (2, 'ind', None)
        if o2 in (0x21, 0x22, 0x2A):
            return (4, 'n', None)
        if o2 == 0x36:
            return (4, 'n', None)
        if o2 in (0x34, 0x35) or (0x46 <= o2 <= 0x7E and (o2 & 7) == 6 and o2 != 0x76) \
                or (0x70 <= o2 <= 0x77 and o2 != 0x76) or o2 in (0x86, 0x8E, 0x96, 0x9E, 0xA6, 0xAE, 0xB6, 0xBE):
            return (3, 'n', None)
        if o2 in (0xDD, 0xFD, 0xED):
            return (1, 'n', None)  # prefix ignored
        # other DD xx behave like the unprefixed opcode with one extra byte
        r = decode(mem, a + 1)
        if r is None:
            return None
        return (r[0] + 1, r[1], r[2])

    def w(x):
        lo, hi = mem(x), mem(x + 1)
        if lo is None or hi is None:
            return None
        return lo | (hi << 8)

    def rel():
        d = mem(a + 1)
        if d is None:
            return None
        if d >= 128:
            d -= 256
        return (a + 2 + d) & 0xFFFF

    if op == 0x76:
        return (1, 'n', None)
    if op == 0x18:
        return (2, 'jp', rel())
    if op in (0x20, 0x28, 0x30, 0x38, 0x10):
        return (2, 'cj', rel())
    if op == 0xC3:
        return (3, 'jp', w(a + 1))
    if (op & 0xC7) == 0xC2:
        return (3, 'cj', w(a + 1))
    if op == 0xCD:
        return (3, 'call', w(a + 1))
    if (op & 0xC7) == 0xC4:
        return (3, 'call', w(a + 1))
    if op == 0xC9:
        return (1, 'ret', None)
    if (op & 0xC7) == 0xC0:
        return (1, 'cret', None)
    if (op & 0xC7) == 0xC7:
        return (1, 'rst', op & 0x38)
    if op == 0xE9:
        return (1, 'ind', None)
    # lengths for the rest
    if op in (0x01, 0x11, 0x21, 0x31, 0x22, 0x2A, 0x32, 0x3A):
        return (3, 'n', None)
    if (op & 0xC7) == 0x06:
        return (2, 'n', None)
    if op in (0xC6, 0xCE, 0xD6, 0xDE, 0xE6, 0xEE, 0xF6, 0xFE, 0xD3, 0xDB):
        return (2, 'n', None)
    return (1, 'n', None)
