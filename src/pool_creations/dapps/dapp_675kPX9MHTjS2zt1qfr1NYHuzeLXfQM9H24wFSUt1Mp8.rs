use crate::constants::{MOONSHOT_MIGRATION, PUMP_FUN_RAYDIUM_MIGRATION, RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS};
use crate::pool_creations::pool_instruction::CreatePoolInstruction;

pub const INSTRUCTION_TYPE_INITIALIZE: &str = "initialize";
pub const INSTRUCTION_TYPE_INITIALIZE2: &str = "initialize2";

const INITIALIZE: u8 = 0;
const INITIALIZE2: u8 = 1;

pub fn parse_trade_instruction(
    bytes_stream: Vec<u8>,
    input_accounts: Vec<String>,
) -> Option<CreatePoolInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let disc_bytes_arr: [u8; 1] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u8 = u8::from_le_bytes(disc_bytes_arr);

    let mut td = CreatePoolInstruction::default();
    let mut result = None;

    match discriminator {
        INITIALIZE | INITIALIZE2 => {
            td.program = RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS.to_string();
            td.name = match discriminator {
                                0 => String::from(INSTRUCTION_TYPE_INITIALIZE),
                                1 => String::from(INSTRUCTION_TYPE_INITIALIZE2),
                                _ => "Unknown".parse().unwrap(),
                            };
            td.amm =  input_accounts.get(4)?.to_string();
            td.coin_mint =  input_accounts.get(8)?.to_string();
            td.pc_mint = input_accounts.get(9)?.to_string();
            td.is_pump_fun = input_accounts.get(17)? == PUMP_FUN_RAYDIUM_MIGRATION;
            td.is_moonshot = input_accounts.get(17)? == MOONSHOT_MIGRATION;
            result = Some(td);
        }
        _ => {}
    }
    return result;
}