"""Small scripted client for the rtvc TCP debugger.

Meant for experiments that check what ROM code does: run a short test
program, stop at breakpoints and read registers, type BASIC lines, take
screenshots. Start the emulator first, for example:

    ./target/release/rtvc --headless --port 8765

then:

    from rtvc_probe import Probe
    p = Probe(8765)
    p.wait_boot()
    out = p.run_asm('''
            LD A,42H
            LD (6300H),A
    ''')
    print(p.mem(0x6300, 1))

Notes from use:
- Test programs run at 6000H by default; run_asm sets SP itself, because
  a program started from wherever the CPU was paused inherits a random SP.
- Breakpoint hits arrive as extra {"event": "breakpoint"} lines on the
  socket; cmd() skips them, continue_to_break() waits for one.
- The instruction_trace_list command can hang on large traces; prefer
  breakpoints plus status.
- Typing too fast drops keys while BASIC is busy; keep type_text's delay.
"""

import json
import os
import socket
import subprocess
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASM = os.path.join(ROOT, "target", "release", "rtvc-asm")


class Probe:
    def __init__(self, port=8765, host="127.0.0.1", timeout=20.0):
        self.sock = socket.create_connection((host, port), timeout=timeout)
        self.buf = b""

    # ------------------------------------------------------------ protocol
    def _readline(self):
        while b"\n" not in self.buf:
            data = self.sock.recv(1 << 16)
            if not data:
                raise ConnectionError("debugger closed the connection")
            self.buf += data
        line, self.buf = self.buf.split(b"\n", 1)
        return json.loads(line)

    def cmd(self, request):
        """Send one command and return its reply, skipping event lines."""
        self.sock.sendall((json.dumps(request) + "\n").encode())
        while True:
            reply = self._readline()
            if "event" not in reply:
                return reply

    # ------------------------------------------------------------ helpers
    def status(self):
        return self.cmd({"cmd": "status"})

    def pause(self):
        self.cmd({"cmd": "pause"})

    def resume(self):
        self.cmd({"cmd": "continue"})

    def mem(self, addr, length=1):
        return self.cmd({"cmd": "read_memory", "addr": addr, "len": length})["data"]

    def word(self, addr):
        lo, hi = self.mem(addr, 2)
        return lo | hi << 8

    def write(self, addr, data):
        self.cmd({"cmd": "write_memory", "addr": addr, "data": list(data)})

    def set_reg(self, name, value):
        self.cmd({"cmd": "set_register", "name": name, "value": value})

    def wait_boot(self, seconds=2.5, warp=False):
        """Let BASIC reach its prompt, then run at normal speed."""
        time.sleep(seconds)
        self.cmd({"cmd": "warp", "enabled": warp})
        self.resume()

    def clear_breakpoints(self):
        for addr in self.cmd({"cmd": "breakpoint_list"}).get("breakpoints", []):
            self.cmd({"cmd": "breakpoint_remove", "addr": addr})

    def breakpoint(self, addr):
        self.cmd({"cmd": "breakpoint_add", "addr": addr})

    def continue_to_break(self, timeout=5.0):
        """Continue and wait for a breakpoint; return status, or None on timeout."""
        self.sock.sendall(b'{"cmd":"continue"}\n')
        old = self.sock.gettimeout()
        self.sock.settimeout(timeout)
        try:
            while True:
                reply = self._readline()
                if reply.get("event") == "breakpoint":
                    break
        except (socket.timeout, TimeoutError):
            return None
        finally:
            self.sock.settimeout(old)
        return self.status()

    # ------------------------------------------------------------ programs
    def assemble(self, source, origin=0x6000):
        """Assemble source (no ORG needed) and return the bytes."""
        with tempfile.TemporaryDirectory() as tmp:
            src = os.path.join(tmp, "probe.asm")
            out = os.path.join(tmp, "probe.bin")
            with open(src, "w") as f:
                f.write(f"        ORG {origin:04X}H\n{source}\n")
            subprocess.run([ASM, "--format", "bin", "--origin", f"{origin:04X}H", src, "-o", out],
                           check=True)
            with open(out, "rb") as f:
                return f.read()

    def run_asm(self, source, origin=0x6000, stack=0x6E00, done=0x6FFF, timeout=10.0):
        """Run a test program until it writes AAH to `done`.

        The program is wrapped with LD SP,stack at the start and a done
        marker plus JR $ at the end. Returns True if it finished.
        """
        body = (f"        LD SP,{stack:04X}H\n{source}\n"
                f"        LD A,0AAH\n        LD ({done:04X}H),A\n"
                f"PROBE_END: JR PROBE_END\n")
        code = self.assemble(body, origin)
        self.pause()
        self.write(origin, code)
        self.write(done, [0])
        self.set_reg("pc", origin)
        self.resume()
        end = time.time() + timeout
        while time.time() < end:
            time.sleep(0.05)
            if self.mem(done)[0] == 0xAA:
                self.pause()
                return True
        self.pause()
        return False

    # ------------------------------------------------------------ keyboard / screen
    def type_text(self, text, delay=0.15):
        """Type text; "\\n" presses RETURN. Keep the delay or keys get dropped."""
        for ch in text:
            if ch == "\n":
                self.cmd({"cmd": "key", "action": "down", "code": 13})
                time.sleep(delay)
                self.cmd({"cmd": "key", "action": "up", "code": 13})
            else:
                self.cmd({"cmd": "key", "action": "press", "char": ch})
            time.sleep(delay)

    def key_tap(self, code, mods=(), hold=0.15):
        """Press a key by JS key code (16 SHIFT, 17 CTRL, 18 ALT, 20 LOCK, ...)."""
        for m in mods:
            self.cmd({"cmd": "key", "action": "down", "code": m})
            time.sleep(0.06)
        self.cmd({"cmd": "key", "action": "down", "code": code})
        time.sleep(hold)
        self.cmd({"cmd": "key", "action": "up", "code": code})
        for m in reversed(mods):
            self.cmd({"cmd": "key", "action": "up", "code": m})
        time.sleep(0.1)

    def screenshot(self, path):
        return self.cmd({"cmd": "save_screenshot", "path": os.path.abspath(path)})
