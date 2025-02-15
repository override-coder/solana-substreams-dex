use crate::constants::PUMP_FUN_AMM_PROGRAM_ADDRESS;
use crate::swap::trade_instruction::TradeInstruction;
use crate::utils::WSOL_ADDRESS;
use borsh::{BorshDeserialize, BorshSerialize};
use substreams_solana::pb::sf::solana::r#type::v1::InnerInstructions;

const BUY_DISCRIMINATOR: u64 = u64::from_le_bytes([102, 6, 61, 18, 1, 218, 235, 234]);
const SELL_DISCRIMINATOR: u64 = u64::from_le_bytes([51, 230, 133, 164, 1, 127, 131, 173]);
const CREATE_DISCRIMINATOR: u64 = u64::from_le_bytes([24, 30, 200, 40, 5, 28, 7, 119]);

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct TradeEvent {
    pub mint: [u8; 32],
    pub sol_amount: u64,
    pub token_amount: u64,
    pub is_buy: bool,
    pub user: [u8; 32],
    pub timestamp: i64,
    pub virtual_sol_reserves: u64,
    pub virtual_token_reserves: u64,
    pub real_sol_reserves: u64,
    pub real_token_reserves: u64,
}

pub fn parse_trade_instruction(bytes_stream: &Vec<u8>, accounts: &Vec<&String>) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let discriminator: u64 = u64::from_le_bytes(disc_bytes.try_into().unwrap());

    match discriminator {
        BUY_DISCRIMINATOR | SELL_DISCRIMINATOR => {
            let name = if discriminator == BUY_DISCRIMINATOR {
                "Buy"
            } else {
                "Sell"
            };

            let amm = accounts.get(3);
            let vault_a = accounts.get(3);
            let vault_b = accounts.get(4);

            if let (Some(amm), Some(vault_a), Some(vault_b)) = (amm, vault_a, vault_b) {
                return Some(TradeInstruction {
                    program: PUMP_FUN_AMM_PROGRAM_ADDRESS.to_string(),
                    name: name.to_string(),
                    amm: amm.to_string(),
                    vault_a: vault_a.to_string(),
                    vault_b: vault_b.to_string(),
                    ..Default::default()
                });
            }
        }
        _ => {}
    }
    None
}

pub fn parse_reserves_instruction(
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    _: &Vec<String>,
    token0: &String,
    _: &String,
) -> (u64, u64) {
    for inner_instruction in inner_instructions {
        for inner_inst in &inner_instruction.instructions {
            let inner_program = &accounts[inner_inst.program_id_index as usize];
            if inner_program == PUMP_FUN_AMM_PROGRAM_ADDRESS {
                if let Some((pc_reserves, coin_reserves)) = parse_swap_event(&inner_inst.data) {
                    return if token0 == WSOL_ADDRESS {
                        (coin_reserves, pc_reserves)
                    } else {
                        (pc_reserves, coin_reserves)
                    };
                }
            }
        }
    }
    (0, 0)
}

fn parse_swap_event(data: &[u8]) -> Option<(u64, u64)> {
    let (_, rest) = data.split_at(16);
    let mut rest_slice = &mut &rest[..];
    TradeEvent::deserialize(&mut rest_slice)
        .ok()
        .map(|event| (event.real_token_reserves, event.real_sol_reserves))
}
