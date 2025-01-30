use crate::constants::METEORA_POOL_PROGRAM_ADDRESS;
use crate::swap::trade_instruction::TradeInstruction;

const SWAP_WITH_PARTNER_DISCRIMINATOR: u64 = u64::from_le_bytes([248, 198, 158, 145, 225, 117, 135, 200]);

pub fn parse_trade_instruction(bytes_stream: &Vec<u8>, accounts: &Vec<&String>) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut result = None;

    match discriminator {
        SWAP_WITH_PARTNER_DISCRIMINATOR => {
            result = Some(TradeInstruction {
                program: String::from(METEORA_POOL_PROGRAM_ADDRESS),
                name: String::from("Swap"),
                amm: accounts.get(0).unwrap().to_string(),
                vault_a: accounts.get(5).unwrap().to_string(),
                vault_b: accounts.get(6).unwrap().to_string(),
                ..Default::default()
            });
        }
        _ => {}
    }

    return result;
}
