use crate::constants::{METEORA_POOL_PROGRAM_ADDRESS, MOONSHOT_MIGRATION, RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS};
use crate::pool_creations::pool_instruction::CreatePoolInstruction;

const initialize: u64 = u64::from_le_bytes([7, 166, 138, 171, 206, 171, 236, 244]);

pub fn parse_trade_instruction(bytes_stream: &Vec<u8>, input_accounts: &Vec<&String>) -> Option<CreatePoolInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut td = CreatePoolInstruction::default();
    let mut result = None;

    match discriminator {
        initialize => {
            td.program = METEORA_POOL_PROGRAM_ADDRESS.to_string();
            td.name = "initializePermissionlessConstantProductPoolWithConfig".to_string();
            td.amm = input_accounts.get(0)?.to_string();
            td.coin_mint = input_accounts.get(3)?.to_string();
            td.pc_mint = input_accounts.get(4)?.to_string();
            td.is_moonshot = input_accounts.get(18)? == &MOONSHOT_MIGRATION;
            result = Some(td);
        }
        _ => {}
    }

    return result;
}
