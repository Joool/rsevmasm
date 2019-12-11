// Copyright 2019 Joel Frank
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod instructions;

use hex;
use std::collections::BTreeMap;
use std::io::Cursor;

use instructions::disassemble_next_byte;

pub use instructions::Instruction;

#[derive(Clone, Debug)]
pub struct Disassembly {
    pub instructions: BTreeMap<usize, Instruction>,
}

impl Disassembly {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let instructions = disassemble_bytes(bytes);
        Self { instructions }
    }

    pub fn from_hex_str(input: &str) -> Result<Self, hex::FromHexError> {
        let instructions = disassemble_hex_str(input)?;
        Ok(Self { instructions })
    }

    pub fn get(&self, addr: usize) -> Option<Instruction> {
        self.instructions.get(&addr).cloned()
    }
}

fn disassemble_hex_str(input: &str) -> Result<BTreeMap<usize, Instruction>, hex::FromHexError> {
    let input = if input[0..2] == *"0x" {
        &input[2..]
    } else {
        input
    };
    hex::decode(input).map(|bytes| disassemble_bytes(&bytes))
}

fn disassemble_bytes(bytes: &[u8]) -> BTreeMap<usize, Instruction> {
    let mut instructions = BTreeMap::new();
    let mut cursor = Cursor::new(bytes);
    while let Ok((offset, instruction)) = disassemble_next_byte(&mut cursor) {
        instructions.insert(offset, instruction);
    }

    instructions
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use std::iter::FromIterator;

    #[test]
    fn simple_programm() {
        let program = "0x608040526002610100";
        let program_bytes = vec![0x60, 0x80, 0x40, 0x52, 0x60, 0x02, 0x61, 0x01, 0x00];
        let disas = BTreeMap::from_iter(hashmap! {
            0 => Instruction::Push(vec!(0x80)),
            2 => Instruction::Blockhash,
            3 => Instruction::MStore,
            4 => Instruction::Push(vec!(0x2)),
            6 => Instruction::Push(vec!(0x1, 0x00)),
        });

        assert_eq!(disassemble_hex_str(program).unwrap(), disas);
        assert_eq!(disassemble_bytes(&program_bytes), disas);
    }
}
