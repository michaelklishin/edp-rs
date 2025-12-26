#!/usr/bin/env python3
# Copyright (C) 2025-2026 Michael S. Klishin and Contributors
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""Reads ETF-encoded bytes from stdin, decodes with erlpack, prints result."""

import json
import struct
import sys

import erlpack


def read_length_prefixed_terms(stream):
    """Read terms with 4-byte big-endian length prefix."""
    terms = []
    while True:
        len_bytes = stream.read(4)
        if not len_bytes:
            break
        if len(len_bytes) < 4:
            raise ValueError(f"Incomplete length prefix: {len(len_bytes)} bytes")

        length = struct.unpack(">I", len_bytes)[0]
        data = stream.read(length)
        if len(data) < length:
            raise ValueError(f"Incomplete term data: expected {length}, got {len(data)}")

        term = erlpack.unpack(data)
        terms.append(term)

    return terms


def term_to_json_safe(term):
    """Convert erlpack term to JSON-serializable format."""
    if isinstance(term, bytes):
        try:
            return term.decode("utf-8")
        except UnicodeDecodeError:
            return list(term)
    elif isinstance(term, erlpack.Atom):
        return {"__atom__": str(term)}
    elif isinstance(term, list):
        return [term_to_json_safe(item) for item in term]
    elif isinstance(term, tuple):
        return {"__tuple__": [term_to_json_safe(item) for item in term]}
    elif isinstance(term, dict):
        return {
            term_to_json_safe(k): term_to_json_safe(v)
            for k, v in term.items()
        }
    else:
        return term


def main():
    terms = read_length_prefixed_terms(sys.stdin.buffer)

    for i, term in enumerate(terms, 1):
        json_safe = term_to_json_safe(term)
        print(f"Term {i}: {json.dumps(json_safe)}")

    print(f"\nDecoded {len(terms)} terms successfully")


if __name__ == "__main__":
    main()
