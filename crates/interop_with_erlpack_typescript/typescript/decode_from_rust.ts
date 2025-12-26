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

// Reads ETF-encoded bytes from stdin (with 4-byte length prefix), decodes with erlpack

import * as erlpack from "erlpack";

async function readStdin(): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) {
    chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}

async function main(): Promise<void> {
  const input = await readStdin();
  let offset = 0;
  let count = 0;

  while (offset < input.length) {
    // Read 4-byte big-endian length
    const len = input.readUInt32BE(offset);
    offset += 4;

    // Read the term data
    const termData = input.subarray(offset, offset + len);
    offset += len;

    // Decode and print
    const decoded = erlpack.unpack(termData);
    count++;
    console.log(`Term ${count}: ${JSON.stringify(decoded)}`);
  }

  console.log(`\nDecoded ${count} terms successfully`);
}

main().catch((err) => {
  console.error("Error:", err);
  process.exit(1);
});
