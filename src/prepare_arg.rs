use crate::instructions::parser::{parse_instruction, Instruction};
use crate::pb::sf::solana::dex::meta::v1::Arg;

pub fn prepare_arg(instruction_data: Vec<u8>, tx_id: String) -> Arg {
    let mut arg: Arg = Arg::default();
    let mut instruction: Instruction = parse_instruction(instruction_data);

    arg.instruction_type = instruction.instructionType;

    match arg.instruction_type.as_str() {
        "CreateMetadataAccount" => {
            arg.create_metadata_account_args =
                Some(instruction.createMetadataAccountArgs.to_proto_struct());
        }
        "CreateMetadataAccountV2" => {
            arg.create_metadata_account_args_v2 =
                Some(instruction.createMetadataAccountArgsV2.to_proto_struct());
        }
        "CreateMetadataAccountV3" => {
            arg.create_metadata_account_args_v3 =
                Some(instruction.createMetadataAccountArgsV3.to_proto_struct());
        }
        _ => {}
    }

    return arg;
}
