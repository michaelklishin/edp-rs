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

//! Reads ETF-encoded terms from stdin (with 4-byte length prefix) and decodes them.

use std::io;
use std::io::Read;

use erltf::decode;

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut count = 0;
    loop {
        let mut len_buf = [0u8; 4];
        match handle.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }

        let len = u32::from_be_bytes(len_buf) as usize;
        let mut data = vec![0u8; len];
        handle.read_exact(&mut data)?;

        match decode(&data) {
            Ok(term) => {
                count += 1;
                println!("Term {count}: {term:?}");
            }
            Err(e) => {
                eprintln!("Failed to decode term {}: {:?}", count + 1, e);
                eprintln!("Raw bytes: {:?}", &data);
            }
        }
    }

    println!("\nDecoded {count} terms successfully");
    Ok(())
}
