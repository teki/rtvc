#!/usr/bin/env python3
"""Convert standard ZX Spectrum .tap CODE blocks to rtvc TOML segments."""

import argparse
import hashlib
import os
import sys

from rtvc_toml import document_to_toml


RAM_START = 0x4000
RAM_END = 0x10000

SPECTRUM_TOKENS = {
    0xA5: "RND",
    0xA6: "INKEY$",
    0xA7: "PI",
    0xA8: "FN",
    0xA9: "POINT",
    0xAA: "SCREEN$",
    0xAB: "ATTR",
    0xAC: "AT",
    0xAD: "TAB",
    0xAE: "VAL$",
    0xAF: "CODE",
    0xB0: "VAL",
    0xB1: "LEN",
    0xB2: "SIN",
    0xB3: "COS",
    0xB4: "TAN",
    0xB5: "ASN",
    0xB6: "ACS",
    0xB7: "ATN",
    0xB8: "LN",
    0xB9: "EXP",
    0xBA: "INT",
    0xBB: "SQR",
    0xBC: "SGN",
    0xBD: "ABS",
    0xBE: "PEEK",
    0xBF: "IN",
    0xC0: "USR",
    0xC1: "STR$",
    0xC2: "CHR$",
    0xC3: "NOT",
    0xC4: "BIN",
    0xC5: "OR",
    0xC6: "AND",
    0xC7: "<=",
    0xC8: ">=",
    0xC9: "<>",
    0xCA: "LINE",
    0xCB: "THEN",
    0xCC: "TO",
    0xCD: "STEP",
    0xCE: "DEF FN",
    0xCF: "CAT",
    0xD0: "FORMAT",
    0xD1: "MOVE",
    0xD2: "ERASE",
    0xD3: "OPEN #",
    0xD4: "CLOSE #",
    0xD5: "MERGE",
    0xD6: "VERIFY",
    0xD7: "BEEP",
    0xD8: "CIRCLE",
    0xD9: "INK",
    0xDA: "PAPER",
    0xDB: "FLASH",
    0xDC: "BRIGHT",
    0xDD: "INVERSE",
    0xDE: "OVER",
    0xDF: "OUT",
    0xE0: "LPRINT",
    0xE1: "LLIST",
    0xE2: "STOP",
    0xE3: "READ",
    0xE4: "DATA",
    0xE5: "RESTORE",
    0xE6: "NEW",
    0xE7: "BORDER",
    0xE8: "CONTINUE",
    0xE9: "DIM",
    0xEA: "REM",
    0xEB: "FOR",
    0xEC: "GO TO",
    0xED: "GO SUB",
    0xEE: "INPUT",
    0xEF: "LOAD",
    0xF0: "LIST",
    0xF1: "LET",
    0xF2: "PAUSE",
    0xF3: "NEXT",
    0xF4: "POKE",
    0xF5: "PRINT",
    0xF6: "PLOT",
    0xF7: "RUN",
    0xF8: "SAVE",
    0xF9: "RANDOMIZE",
    0xFA: "IF",
    0xFB: "CLS",
    0xFC: "DRAW",
    0xFD: "CLEAR",
    0xFE: "RETURN",
    0xFF: "COPY",
}


class TapError(ValueError):
    pass


def read_word(data, offset):
    return data[offset] | (data[offset + 1] << 8)


def read_word_be(data, offset):
    return (data[offset] << 8) | data[offset + 1]


def parse_tap(data):
    blocks = []
    offset = 0
    while offset < len(data):
        if offset + 2 > len(data):
            raise TapError("truncated TAP block length")
        length = read_word(data, offset)
        offset += 2
        if length == 0:
            raise TapError("TAP block length must not be zero")
        end = offset + length
        if end > len(data):
            raise TapError("truncated TAP block payload")
        payload = data[offset:end]
        offset = end
        checksum = 0
        for byte in payload:
            checksum ^= byte
        blocks.append(
            {
                "flag": payload[0],
                "data": payload[1:-1],
                "checksum": payload[-1],
                "checksum_ok": checksum == 0,
                "tap_len": length,
            }
        )
    return blocks


def spectrum_name(raw_name):
    return bytes(raw_name).decode("ascii", errors="replace").rstrip()


def header_from_block(block):
    data = block["data"]
    if block["flag"] != 0x00 or len(data) != 17:
        return None
    block_type = data[0]
    type_name = {
        0: "program",
        1: "number_array",
        2: "character_array",
        3: "code",
    }.get(block_type, f"unknown_{block_type}")
    return {
        "type": type_name,
        "type_id": block_type,
        "name": spectrum_name(data[1:11]),
        "length": read_word(data, 11),
        "param1": read_word(data, 13),
        "param2": read_word(data, 15),
        "checksum_ok": block["checksum_ok"],
    }


def basic_usr_numbers(program):
    results = []
    offset = 0
    while offset + 4 <= len(program):
        line_no = read_word_be(program, offset)
        line_len = read_word(program, offset + 2)
        offset += 4
        line_end = offset + line_len
        if line_end > len(program):
            break
        line = program[offset:line_end]
        offset = line_end

        for index, byte in enumerate(line):
            if byte != 0xC0:  # USR token
                continue
            number = number_after_usr(line, index + 1)
            if number is not None:
                results.append({"line": line_no, "addr": number})
    return results


def decode_basic_program(program):
    lines = []
    offset = 0
    while offset + 4 <= len(program):
        line_no = read_word_be(program, offset)
        line_len = read_word(program, offset + 2)
        offset += 4
        line_end = offset + line_len
        if line_end > len(program):
            break
        line = program[offset:line_end]
        offset = line_end
        lines.append(f"{line_no} {decode_basic_line(line)}".rstrip())
    return lines


def decode_basic_line(line):
    out = []
    index = 0
    in_string = False
    while index < len(line):
        byte = line[index]
        if byte == 0x0D:
            break
        if byte == 0x0E:
            index += 6
            continue
        if byte == ord('"'):
            out.append('"')
            in_string = not in_string
        elif byte in SPECTRUM_TOKENS and not in_string:
            token = SPECTRUM_TOKENS[byte]
            if out and needs_space_before_token(out[-1], token):
                out.append(" ")
            out.append(token)
            if token[-1:].isalnum() or token.endswith("$"):
                out.append(" ")
        elif 0x20 <= byte <= 0x7E:
            out.append(chr(byte))
        else:
            out.append(f"{{0x{byte:02X}}}")
        index += 1
    text = "".join(out).strip()
    for punct in (":", ";", ",", ")"):
        text = text.replace(f" {punct}", punct)
    return text.replace("( ", "(")


def needs_space_before_token(previous, token):
    return previous and not previous[-1].isspace() and previous[-1] not in ':;,(' and token[0].isalnum()


def number_after_usr(line, offset):
    digits = []
    index = offset
    while index < len(line):
        byte = line[index]
        if byte == 0x0E:
            index += 6
            continue
        if 0x30 <= byte <= 0x39:
            digits.append(chr(byte))
        elif digits:
            break
        elif byte not in (0x20, ord("(")):
            break
        index += 1
    if not digits:
        return None
    value = int("".join(digits), 10)
    if 0 <= value <= 0xFFFF:
        return value
    return None


def build_tap_order(headers, segments, data_blocks, raw_blocks):
    order = []
    for header in headers:
        order.append(
            {
                "block_index": header["block_index"],
                "section": "headers",
                "kind": "header",
                "name": header.get("name", ""),
                "type": header.get("type", ""),
            }
        )
    for block in data_blocks:
        order.append(
            {
                "block_index": block["source_block"],
                "section": "data_blocks",
                "kind": block.get("type", "data"),
                "name": block.get("name", ""),
                "header_name": block.get("header_name", ""),
            }
        )
    for segment in segments:
        order.append(
            {
                "block_index": segment["source_block"],
                "section": "segments",
                "kind": "code",
                "name": segment.get("name", ""),
                "header_name": segment.get("header_name", ""),
                "addr": segment.get("addr"),
                "len": segment.get("len"),
            }
        )
    for block in raw_blocks:
        order.append(
            {
                "block_index": block["block_index"],
                "section": "raw_blocks",
                "kind": "raw",
                "role": block.get("role", ""),
                "flag": block.get("flag"),
                "payload_len": block.get("payload_len"),
            }
        )
    return sorted(order, key=lambda item: item["block_index"])


def load_tap(data):
    blocks = parse_tap(data)
    headers = []
    segments = []
    data_blocks = []
    raw_blocks = []
    warnings = []
    pending_header = None
    entry_candidates = []

    for index, block in enumerate(blocks):
        header = header_from_block(block)
        if header is not None:
            header["block_index"] = index
            headers.append(header)
            pending_header = header
            continue

        if block["flag"] != 0xFF:
            raw_blocks.append(
                {
                    "block_index": index,
                    "flag": block["flag"],
                    "payload_len": len(block["data"]),
                    "tap_len": block["tap_len"],
                    "checksum": block["checksum"],
                    "checksum_ok": block["checksum_ok"],
                    "role": "multiload_or_custom_data",
                    "bytes": list(block["data"]),
                }
            )
            warnings.append(
                f"block {index} has non-standard data flag 0x{block['flag']:02X}; preserved in raw_blocks"
            )
            pending_header = None
            continue

        payload = block["data"]
        if pending_header is None:
            raw_blocks.append(
                {
                    "block_index": index,
                    "flag": block["flag"],
                    "payload_len": len(payload),
                    "tap_len": block["tap_len"],
                    "checksum": block["checksum"],
                    "checksum_ok": block["checksum_ok"],
                    "role": "headerless_data",
                    "bytes": list(payload),
                }
            )
            warnings.append(f"block {index} is a data block without a preceding header; preserved in raw_blocks")
            continue

        declared_len = pending_header["length"]
        if len(payload) != declared_len:
            warnings.append(
                f"block {index} data length {len(payload)} does not match header length {declared_len}"
            )

        if pending_header["type"] == "code":
            addr = pending_header["param1"]
            end = addr + len(payload)
            if addr < RAM_START or end > RAM_END:
                warnings.append(
                    f"CODE block {index} at 0x{addr:04X} length {len(payload)} is not wholly in 48K RAM"
                )
            start_clip = max(addr, RAM_START)
            end_clip = min(end, RAM_END)
            if start_clip < end_clip:
                data_start = start_clip - addr
                data_end = end_clip - addr
                chunk = payload[data_start:data_end]
                segments.append(
                    {
                        "name": f"tap_code_{len(segments)}_{pending_header['name'] or 'unnamed'}",
                        "addr": start_clip,
                        "len": len(chunk),
                        "bytes": list(chunk),
                        "source_block": index,
                        "header_name": pending_header["name"],
                    }
                )
        elif pending_header["type"] == "program":
            entry_candidates.extend(basic_usr_numbers(payload))
            data_blocks.append(
                {
                    "name": f"tap_program_{len(data_blocks)}_{pending_header['name'] or 'unnamed'}",
                    "type": pending_header["type"],
                    "source_block": index,
                    "header_name": pending_header["name"],
                    "len": len(payload),
                    "line": pending_header["param1"],
                    "variables_offset": pending_header["param2"],
                    "basic_lines": decode_basic_program(payload),
                    "bytes": list(payload),
                }
            )
        else:
            data_blocks.append(
                {
                    "name": f"tap_{pending_header['type']}_{len(data_blocks)}_{pending_header['name'] or 'unnamed'}",
                    "type": pending_header["type"],
                    "source_block": index,
                    "header_name": pending_header["name"],
                    "len": len(payload),
                    "param1": pending_header["param1"],
                    "param2": pending_header["param2"],
                    "bytes": list(payload),
                }
            )

        pending_header = None

    entry = None
    if entry_candidates:
        entry = entry_candidates[-1]["addr"]

    return {
        "format": "rtvc-zx-tap-v1",
        "machine": "zx-spectrum-48k",
        "block_count": len(blocks),
        "tap_order": build_tap_order(headers, segments, data_blocks, raw_blocks),
        "headers": headers,
        "segments": segments,
        "data_blocks": data_blocks,
        "raw_blocks": raw_blocks,
        "entry": entry,
        "entry_candidates": entry_candidates,
        "warnings": warnings,
        "tap_sha256": hashlib.sha256(data).hexdigest(),
        "tvc_bridge": {
            "main_map_port_02": 0xB4,
            "video_page_port_0c": 0x00,
            "video_mode_port_06": 0x00,
            "segment_mapping": [
                {"addr_start": 0x4000, "addr_end": 0x7FFF, "suggested_tvc_bank": "vid0"},
                {"addr_start": 0x8000, "addr_end": 0xBFFF, "suggested_tvc_bank": "u2"},
                {"addr_start": 0xC000, "addr_end": 0xFFFF, "suggested_tvc_bank": "u3"},
            ],
        },
    }


def parse_args(argv):
    parser = argparse.ArgumentParser(
        description="Convert standard ZX Spectrum .tap CODE blocks to rtvc TOML."
    )
    parser.add_argument("input", help=".tap file to convert")
    parser.add_argument("-o", "--output", help="output TOML path; defaults to stdout")
    parser.add_argument(
        "--compact",
        action="store_true",
        help="accepted for compatibility; TOML output is always human-readable",
    )
    parser.add_argument(
        "--no-source-path",
        action="store_true",
        help="omit the input filename from the generated TOML",
    )
    return parser.parse_args(argv)


def main(argv=None):
    args = parse_args(argv or sys.argv[1:])
    try:
        with open(args.input, "rb") as tap_file:
            data = tap_file.read()
        document = load_tap(data)
    except (OSError, TapError) as err:
        print(f"tap_to_toml: {err}", file=sys.stderr)
        return 1

    if not args.no_source_path:
        document["source"] = os.path.basename(args.input)

    output = document_to_toml(document)

    if args.output:
        try:
            with open(args.output, "w", encoding="utf-8") as output_file:
                output_file.write(output)
        except OSError as err:
            print(f"tap_to_toml: could not write {args.output}: {err}", file=sys.stderr)
            return 1
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
