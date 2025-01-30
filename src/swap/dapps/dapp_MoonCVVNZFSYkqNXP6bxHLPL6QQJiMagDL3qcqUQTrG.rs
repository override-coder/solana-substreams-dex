use crate::constants::MOONSHOT_ADDRESS;
use crate::swap::trade_instruction::TradeInstruction;

const BUY_DISCRIMINATOR: u64 = u64::from_le_bytes([102, 6, 61, 18, 1, 218, 235, 234]);
const SELL_DISCRIMINATOR: u64 = u64::from_le_bytes([51, 230, 133, 164, 1, 127, 131, 173]);

pub fn parse_trade_instruction(bytes_stream: &Vec<u8>, accounts: &Vec<&String>) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut result = None;

    match discriminator {
        BUY_DISCRIMINATOR => {
            result = Some(TradeInstruction {
                program: String::from(MOONSHOT_ADDRESS),
                name: String::from("Buy"),
                amm: accounts.get(2).unwrap().to_string(),
                vault_a: accounts.get(2).unwrap().to_string(),
                vault_b: accounts.get(3).unwrap().to_string(),
                ..Default::default()
            });
        }

        SELL_DISCRIMINATOR => {
            result = Some(TradeInstruction {
                program: String::from(MOONSHOT_ADDRESS),
                name: String::from("Sell"),
                amm: accounts.get(2).unwrap().to_string(),
                vault_a: accounts.get(2).unwrap().to_string(),
                vault_b: accounts.get(3).unwrap().to_string(),
                ..Default::default()
            });
        }
        _ => {}
    }

    return result;
}
