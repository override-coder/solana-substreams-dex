use crate::constants::{PUMP_FUN_AMM_PROGRAM_ADDRESS, PUMP_FUN_RAYDIUM_MIGRATION};
use crate::pool_creations::pool_instruction::CreatePoolInstruction;
use crate::utils::WSOL_ADDRESS;

const CREATE: u64 = u64::from_le_bytes([24, 30, 200, 40, 5, 28, 7, 119]);

pub fn parse_trade_instruction(
    bytes_stream: Vec<u8>,
    input_accounts: Vec<String>,
) -> Option<CreatePoolInstruction> {
    let (disc_bytes, _) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut td = CreatePoolInstruction::default();
    let mut result = None;

    match discriminator {
        CREATE => {
            td.program = PUMP_FUN_AMM_PROGRAM_ADDRESS.to_string();
            td.name = "create".to_string();
            td.amm =  input_accounts.get(2)?.to_string();
            td.coin_mint = WSOL_ADDRESS.to_string();
            td.pc_mint = input_accounts.get(0)?.to_string();
            td.is_pump_fun = input_accounts.get(13)? == PUMP_FUN_AMM_PROGRAM_ADDRESS;
            result = Some(td);
        }
        _ => {}
    }
    return result;
}
