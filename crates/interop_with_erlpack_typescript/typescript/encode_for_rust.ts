// Copyright (C) 2025-2026 Michael S. Klishin and Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Encodes test data with erlpack, writes ETF bytes to stdout

import * as erlpack from "erlpack";

const testCases = [
  // Nil
  null,
  // Booleans (encoded as atoms)
  true,
  false,
  // Integers
  0,
  42,
  -100,
  255,
  256,
  65535,
  2147483647,
  -2147483648,
  // Float
  1.23456,
  // Strings/Binaries
  "hello",
  "unicode: \u00e9\u00e8\u00ea",
  // Arrays (Lists)
  [],
  [1, 2, 3],
  ["a", "b", "c"],
  // Nested structures
  { key: "value" },
  { nested: { deep: { value: 42 } } },
  [1, [2, [3]]],
  // Mixed
  { list: [1, 2, 3], num: 42, str: "test" },
];

function main(): void {
  for (const testCase of testCases) {
    const encoded = erlpack.pack(testCase);
    // Write length as 4-byte big-endian, then the data
    const lenBuf = Buffer.alloc(4);
    lenBuf.writeUInt32BE(encoded.length, 0);
    process.stdout.write(lenBuf);
    process.stdout.write(encoded);
  }
}

main();
