pub mod instructions;

use hex;
use std::collections::HashMap;
use std::io::Cursor;

use instructions::disassemble_next_byte;

pub use instructions::Instruction;

pub struct Disassembly {
    pub instructions: HashMap<usize, Instruction>,
}

impl Disassembly {
    pub fn from_hex_str(input: &str) -> Result<Self, hex::FromHexError> {
        let instructions = disassemble_hex_str(input)?;
        Ok(Self { instructions })
    }

    pub fn get(&self, addr: usize) -> Option<Instruction> {
        self.instructions.get(&addr).cloned()
    }
}

fn disassemble_hex_str(input: &str) -> Result<HashMap<usize, Instruction>, hex::FromHexError> {
    let input = if input[0..2] == *"0x" {
        &input[2..]
    } else {
        input
    };
    hex::decode(input).map(disassemble_bytes)
}

fn disassemble_bytes(bytes: Vec<u8>) -> HashMap<usize, Instruction> {
    let mut instructions = HashMap::new();
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

    #[test]
    fn simple_programm() {
        let program = "0x608040526002610100";
        let program_bytes = vec![0x60, 0x80, 0x40, 0x52, 0x60, 0x02, 0x61, 0x01, 0x00];
        let disas = hashmap! {
            0 => Instruction::Push(vec!(0x80)),
            2 => Instruction::Blockhash,
            3 => Instruction::MStore,
            4 => Instruction::Push(vec!(0x2)),
            6 => Instruction::Push(vec!(0x1, 0x00)),
        };

        assert_eq!(disassemble_hex_str(program).unwrap(), disas);
        assert_eq!(disassemble_bytes(program_bytes), disas);
    }
}
