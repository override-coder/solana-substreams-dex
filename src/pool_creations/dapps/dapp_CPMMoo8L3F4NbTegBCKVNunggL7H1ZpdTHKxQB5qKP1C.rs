use crate::constants::{ RAYDIUM_CPMM_ADDRESS};
use crate::pool_creations::pool_instruction::CreatePoolInstruction;

const INITIALIZE: u64 = u64::from_le_bytes([175, 175, 109, 31, 13, 152, 155, 237]);

pub fn parse_trade_instruction(
    bytes_stream: Vec<u8>,
    input_accounts: Vec<String>,
) -> Option<CreatePoolInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut td = CreatePoolInstruction::default();
    let mut result = None;

    match discriminator {
        INITIALIZE => {
            td.program = RAYDIUM_CPMM_ADDRESS.to_string();
            td.name = "initialize".to_string();
            td.amm =  input_accounts.get(3)?.to_string();
            td.coin_mint =  input_accounts.get(4)?.to_string();
            td.pc_mint = input_accounts.get(5)?.to_string();
            result = Some(td);
        }
        _ => {}
    }
    return result;
}
