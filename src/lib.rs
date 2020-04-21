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
pub mod error;
pub mod instructions;

use hex;
use std::collections::BTreeMap;
use std::io::Cursor;

use instructions::{assemble_instruction, disassemble_next_byte};

pub use error::DisassemblyError;
pub use instructions::Instruction;

#[derive(Clone, Debug)]
pub struct Disassembly {
    pub instructions: BTreeMap<usize, Instruction>,
}

impl Disassembly {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DisassemblyError> {
        let instructions = disassemble_bytes(bytes)?;
        Ok(Self { instructions })
    }

    pub fn from_hex_str(input: &str) -> Result<Self, DisassemblyError> {
        let instructions = disassemble_hex_str(input)?;
        Ok(Self { instructions })
    }

    pub fn get(&self, addr: usize) -> Option<Instruction> {
        self.instructions.get(&addr).cloned()
    }
}

pub fn assemble_instructions(disassembly: Vec<Instruction>) -> Vec<u8> {
    let mut result = Vec::new();
    for disas in disassembly {
        result.extend(assemble_instruction(disas));
    }
    result
}

fn disassemble_hex_str(input: &str) -> Result<BTreeMap<usize, Instruction>, DisassemblyError> {
    let input = if input[0..2] == *"0x" {
        &input[2..]
    } else {
        input
    };
    let bytes = hex::decode(input)?;
    disassemble_bytes(&bytes)
}

fn disassemble_bytes(bytes: &[u8]) -> Result<BTreeMap<usize, Instruction>, DisassemblyError> {
    let mut instructions = BTreeMap::new();
    let mut cursor = Cursor::new(bytes);
    loop {
        let result = disassemble_next_byte(&mut cursor);
        match result {
            Err(DisassemblyError::IOError(..)) => break,
            Ok((offset, instruction)) => {
                instructions.insert(offset, instruction);
            }
            Err(err) => {
                if let DisassemblyError::TooFewBytesForPush = err {
                    // the solidity compiler sometimes puts push instructions at the end, however,
                    // this is considered normal behaviour
                    break;
                }
                return Err(err);
            }
        }
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use quickcheck::{quickcheck, TestResult};
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
        assert_eq!(disassemble_bytes(&program_bytes).unwrap(), disas);
    }

    #[test]
    fn longer_program() {
        let prog = "6080604052348015600f57600080fd5b506004361060285760003560e01c806318b969a014602d575b600080fd5b60336049565b6040518082815260200191505060405180910390f35b6000600260016003600054020181605c57fe5b04600081905550633b9aca0060005481607157fe5b06600081905550600043905060004490506000429050600045905060008183858760005401010101905060006025828160a657fe5b0690508096505050505050509056fea265627a7a72315820352cd5f3ce6a4befb464c84b845f54a57437fd835fffaf9b4d17a089ea70d25f64736f6c634300050d0032";
        disassemble_hex_str(prog).unwrap();
    }
}
