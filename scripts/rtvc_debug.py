#!/usr/bin/env python3
import socket
import json
import sys
import argparse
import cmd
import readline # Enables command history and line editing

class RtvcShell(cmd.Cmd):
    intro = "Welcome to the RTVC Debugger Shell. Type help or ? to list commands.\n"
    prompt = "rtvc> "

    def __init__(self, host, port):
        super().__init__()
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            self.sock.connect((host, port))
        except ConnectionRefusedError:
            print(f"Error: Could not connect to emulator at {host}:{port}")
            sys.exit(1)
        self.sock.settimeout(5.0)
        print(f"Connected to emulator debugger at {host}:{port}")
        # Print initial status
        self.do_status("")

    def _send_cmd(self, cmd_dict):
        try:
            line = json.dumps(cmd_dict) + "\n"
            self.sock.sendall(line.encode("utf-8"))
            
            resp_bytes = b""
            while b"\n" not in resp_bytes:
                data = self.sock.recv(1)
                if not data:
                    break
                resp_bytes += data
            
            if not resp_bytes:
                print("Error: Connection closed by server.")
                return None
            
            return json.loads(resp_bytes.decode("utf-8").strip())
        except Exception as e:
            print(f"Communication Error: {e}")
            return None

    def do_status(self, arg):
        """Show Z80 CPU registers, cycles, running and halted status."""
        resp = self._send_cmd({"cmd": "status"})
        if not resp:
            return
        if resp.get("status") != "ok":
            print(f"Error: {resp.get('message')}")
            return
        
        print(f"AF: 0x{resp.get('af', 0):04X}   BC: 0x{resp.get('bc', 0):04X}   DE: 0x{resp.get('de', 0):04X}   HL: 0x{resp.get('hl', 0):04X}")
        print(f"IX: 0x{resp.get('ix', 0):04X}   IY: 0x{resp.get('iy', 0):04X}   SP: 0x{resp.get('sp', 0):04X}   PC: 0x{resp.get('pc', 0):04X}")
        print(f"Halted: {resp.get('halted')}   Running: {resp.get('running')}   Cycles: {resp.get('cycles')}")

    def do_step(self, arg):
        """Step the Z80 CPU. Usage: step [count]"""
        count = 1
        if arg:
            try:
                count = int(arg)
            except ValueError:
                print("Error: count must be an integer.")
                return
        
        resp = self._send_cmd({"cmd": "step", "count": count})
        if resp and resp.get("status") == "ok":
            self.do_status("")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_continue(self, arg):
        """Resume execution in real time."""
        resp = self._send_cmd({"cmd": "continue"})
        if resp and resp.get("status") == "ok":
            print("Emulator resumed.")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_pause(self, arg):
        """Pause CPU execution."""
        resp = self._send_cmd({"cmd": "pause"})
        if resp and resp.get("status") == "ok":
            print("Emulator paused.")
            self.do_status("")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_reset(self, arg):
        """Reset the emulator state."""
        resp = self._send_cmd({"cmd": "reset"})
        if resp and resp.get("status") == "ok":
            print("Emulator reset.")
            self.do_status("")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_bp(self, arg):
        """Manage breakpoints.
        Usage:
          bp list
          bp add <address>
          bp rm <address>
        """
        parts = arg.split()
        if not parts:
            print("Usage: bp [list | add <addr> | rm <addr>]")
            return
        
        subcmd = parts[0].lower()
        if subcmd == "list":
            resp = self._send_cmd({"cmd": "breakpoint_list"})
            if resp and resp.get("status") == "ok":
                bps = resp.get("breakpoints", [])
                if bps:
                    print("Breakpoints: " + ", ".join(f"0x{bp:04X}" for bp in bps))
                else:
                    print("No active breakpoints.")
            elif resp:
                print(f"Error: {resp.get('message')}")
        elif subcmd in ("add", "rm", "remove"):
            if len(parts) < 2:
                print("Error: Missing address.")
                return
            try:
                addr_str = parts[1]
                addr = int(addr_str, 16) if addr_str.lower().startswith("0x") else int(addr_str)
            except ValueError:
                print("Error: Address must be an integer (decimal or hex starting with 0x).")
                return
            
            cmd_name = "breakpoint_add" if subcmd == "add" else "breakpoint_remove"
            resp = self._send_cmd({"cmd": cmd_name, "addr": addr})
            if resp and resp.get("status") == "ok":
                print(f"Breakpoint {'added' if subcmd == 'add' else 'removed'} at 0x{addr:04X}")
            elif resp:
                print(f"Error: {resp.get('message')}")
        else:
            print("Unknown bp command. Use list, add, or rm.")

    def do_read(self, arg):
        """Read memory from mapped view or bank.
        Usage: read <address> <length> [bank]
        Examples:
          read 0x0000 32
          read 0x0000 64 sys
        """
        parts = arg.split()
        if len(parts) < 2:
            print("Usage: read <addr> <len> [bank]")
            return
        try:
            addr = int(parts[0], 16) if parts[0].lower().startswith("0x") else int(parts[0])
            length = int(parts[1], 16) if parts[1].lower().startswith("0x") else int(parts[1])
        except ValueError:
            print("Error: Address and length must be integers.")
            return
        
        bank = parts[2] if len(parts) >= 3 else None
        
        cmd_dict = {"cmd": "read_memory", "addr": addr, "len": length}
        if bank:
            cmd_dict["bank"] = bank
            
        resp = self._send_cmd(cmd_dict)
        if resp and resp.get("status") == "ok":
            data = resp.get("data", [])
            print(self._hex_dump(addr, data))
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_disasm(self, arg):
        """Disassemble instructions.
        Usage: disasm <address> <byte_length>
        Example:
          disasm 0x0000 32
        """
        parts = arg.split()
        if len(parts) < 2:
            print("Usage: disasm <addr> <len>")
            return
        try:
            addr = int(parts[0], 16) if parts[0].lower().startswith("0x") else int(parts[0])
            length = int(parts[1], 16) if parts[1].lower().startswith("0x") else int(parts[1])
        except ValueError:
            print("Error: Address and length must be integers.")
            return
            
        resp = self._send_cmd({"cmd": "disassemble", "addr": addr, "len": length})
        if resp and resp.get("status") == "ok":
            for inst in resp.get("instructions", []):
                inst_addr = inst.get("addr", 0)
                bytes_list = inst.get("bytes", [])
                bytes_str = " ".join(f"{b:02X}" for b in bytes_list).ljust(12)
                text = inst.get("text", "")
                print(f"0x{inst_addr:04X}:  {bytes_str} {text}")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_save(self, arg):
        """Save a snapshot zip file. Usage: save <path>"""
        if not arg:
            print("Usage: save <snapshot_file_path>")
            return
        resp = self._send_cmd({"cmd": "save_snapshot", "path": arg})
        if resp and resp.get("status") == "ok":
            print(f"Snapshot saved to {arg}")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_load(self, arg):
        """Load a snapshot zip file. Usage: load <path>"""
        if not arg:
            print("Usage: load <snapshot_file_path>")
            return
        resp = self._send_cmd({"cmd": "load_snapshot", "path": arg})
        if resp and resp.get("status") == "ok":
            print(f"Snapshot loaded from {arg}")
            self.do_status("")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_screenshot(self, arg):
        """Capture screen as PNG. Usage: screenshot <path>"""
        if not arg:
            print("Usage: screenshot <png_file_path>")
            return
        resp = self._send_cmd({"cmd": "save_screenshot", "path": arg})
        if resp and resp.get("status") == "ok":
            print(f"Screenshot saved to {arg}")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_key(self, arg):
        """Send keyboard events.
        Usage:
          key press <char>
          key down <keycode>
          key up <keycode>
        Examples:
          key press A
          key down 65
          key up 65
        """
        parts = arg.split()
        if len(parts) < 2:
            print("Usage: key [press <char> | down <keycode> | up <keycode>]")
            return
        action = parts[0].lower()
        if action == "press":
            char = parts[1]
            resp = self._send_cmd({"cmd": "key", "action": "press", "char": char})
        elif action in ("down", "up"):
            try:
                code = int(parts[1])
            except ValueError:
                print("Error: keycode must be an integer.")
                return
            resp = self._send_cmd({"cmd": "key", "action": action, "code": code})
        else:
            print("Unknown action. Use press, down, or up.")
            return

        if resp and resp.get("status") == "ok":
            print("Key command sent.")
        elif resp:
            print(f"Error: {resp.get('message')}")

    def do_exit(self, arg):
        """Exit the debugger shell."""
        print("Goodbye.")
        return True

    def do_EOF(self, arg):
        """Exit the debugger shell on EOF (Ctrl-D)."""
        print()
        return self.do_exit(arg)

    # Aliases
    do_s = do_status
    do_t = do_step
    do_c = do_continue
    do_p = do_pause
    do_m = do_read
    do_d = do_disasm
    do_q = do_exit

    def _hex_dump(self, addr_start, data):
        lines = []
        for i in range(0, len(data), 16):
            chunk = data[i:i+16]
            hex_str = " ".join(f"{b:02X}" for b in chunk)
            # Add extra space after 8 bytes for readability
            if len(chunk) > 8:
                # 8 bytes = 23 chars (8 * 3 - 1)
                hex_str = hex_str[:23] + "  " + hex_str[24:]
            hex_str = hex_str.ljust(48)
            ascii_str = "".join(chr(b) if 32 <= b <= 126 else "." for b in chunk)
            lines.append(f"0x{addr_start + i:04X}:  {hex_str}  |{ascii_str}|")
        return "\n".join(lines)

def main():
    parser = argparse.ArgumentParser(description="Videoton TV Computer Headless Debugger Client")
    parser.add_argument("--host", default="127.0.0.1", help="Debugger host IP (default: 127.0.0.1)")
    parser.add_argument("--port", "-p", type=int, default=8080, help="Debugger port (default: 8080)")
    args = parser.parse_args()

    shell = RtvcShell(args.host, args.port)
    shell.cmdloop()

if __name__ == "__main__":
    main()
