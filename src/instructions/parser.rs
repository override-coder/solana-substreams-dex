extern crate bs58;

use borsh::BorshDeserialize;

use super::structs::{
    CreateMetadataAccountArgsLayout,
    CreateMetadataAccountArgsV2Layout,
    CreateMetadataAccountArgsV3Layout,
};

#[derive(Debug, Default)]
pub struct Instruction {
    pub instructionType: String,
    pub createMetadataAccountArgs: CreateMetadataAccountArgsLayout,
    pub createMetadataAccountArgsV2: CreateMetadataAccountArgsV2Layout,
    pub createMetadataAccountArgsV3: CreateMetadataAccountArgsV3Layout,
}

pub fn parse_instruction(bytes_stream: Vec<u8>) -> Instruction {
    let mut result: Instruction = Instruction::default();

    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let discriminator: u8 = u8::from(disc_bytes[0]);
    let rest_bytes = &mut rest.clone();

    match discriminator {
        0 => {
            result.instructionType = "CreateMetadataAccount".to_string();
            if rest_bytes.len() > 0 {
                result.createMetadataAccountArgs =
                    CreateMetadataAccountArgsLayout::deserialize(rest_bytes).unwrap();
            }
        }
        16 => {
            result.instructionType = "CreateMetadataAccountV2".to_string();
            if rest_bytes.len() > 0 {
                result.createMetadataAccountArgsV2 =
                    CreateMetadataAccountArgsV2Layout::deserialize(rest_bytes).unwrap();
            }
        }
        33 => {
            result.instructionType = "CreateMetadataAccountV3".to_string();
            if rest_bytes.len() > 0 {
                result.createMetadataAccountArgsV3 =
                    CreateMetadataAccountArgsV3Layout::deserialize(rest_bytes).unwrap();
            }
        }
        _ => {}
    }

    return result;
}
