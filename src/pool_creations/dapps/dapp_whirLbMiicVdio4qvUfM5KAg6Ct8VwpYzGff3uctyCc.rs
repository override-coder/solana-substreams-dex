use crate::constants::ORCA_PROGRAM_ADDRESS;
use crate::pool_creations::pool_instruction::CreatePoolInstruction;

const INITIALIZE_POOL: u64 = u64::from_le_bytes([95, 180, 10, 172, 84, 174, 232, 40]);
const INITIALIZE_POOL_V2: u64 = u64::from_le_bytes([207, 45, 87, 242, 27, 63, 204, 67]);

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
        INITIALIZE_POOL => {
            td.program = ORCA_PROGRAM_ADDRESS.to_string();
            td.name = "initializePool".to_string();
            td.amm =  input_accounts.get(4)?.to_string();
            td.coin_mint =  input_accounts.get(1)?.to_string();
            td.pc_mint = input_accounts.get(2)?.to_string();
            result = Some(td);
        }
        INITIALIZE_POOL_V2 => {
            td.program = ORCA_PROGRAM_ADDRESS.to_string();
            td.name = "initializePoolV2".to_string();
            td.amm =  input_accounts.get(6)?.to_string();
            td.coin_mint =  input_accounts.get(1)?.to_string();
            td.pc_mint = input_accounts.get(2)?.to_string();
            result = Some(td);
        }
        _ => {}
    }
    return result;
}






