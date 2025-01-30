use crate::constants::{METEORA_PROGRAM_ADDRESS, RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS};
use crate::pool_creations::pool_instruction::CreatePoolInstruction;

const INITIALIZE_LB_PAIR: u64 = u64::from_le_bytes([45, 154, 237, 210, 221, 15, 166, 92]);

pub fn parse_trade_instruction(bytes_stream: &Vec<u8>, input_accounts: &Vec<&String>) -> Option<CreatePoolInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut td = CreatePoolInstruction::default();
    let mut result = None;

    match discriminator {
        INITIALIZE_LB_PAIR => {
            td.program = METEORA_PROGRAM_ADDRESS.to_string();
            td.name = "initializeLbPair".to_string();
            td.amm = input_accounts.get(0)?.to_string();
            td.coin_mint = input_accounts.get(2)?.to_string();
            td.pc_mint = input_accounts.get(3)?.to_string();
            result = Some(td);
        }
        _ => {}
    }

    return result;
}
