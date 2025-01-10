use crate::constants::{MOONSHOT_ADDRESS, MOONSHOT_MIGRATION};
use crate::pool_creations::pool_instruction::CreatePoolInstruction;
use crate::utils::WSOL_ADDRESS;

const TOKEN_MINT: u64 = u64::from_le_bytes([3, 44, 164, 184, 123, 13, 245, 179]);

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
        TOKEN_MINT => {
            td.program = MOONSHOT_ADDRESS.to_string();
            td.name = "token_mint".to_string();
            td.amm =  input_accounts.get(2)?.to_string();
            td.coin_mint = WSOL_ADDRESS.to_string();
            td.pc_mint = input_accounts.get(3)?.to_string();
            td.is_moonshot = input_accounts.get(17)? == MOONSHOT_ADDRESS;
            result = Some(td);
        }
        _ => {}
    }

    return result;
}
