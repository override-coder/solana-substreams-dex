use crate::utils::{TradeEvent};
use borsh::{ BorshDeserialize};
use substreams_solana::pb::sf::solana::r#type::v1::InnerInstructions;
use crate::constants::PUMP_FUN_AMM_PROGRAM_ADDRESS;
use crate::swap::trade_instruction::{CreatePoolInstruction, TradeInstruction};

const BUY_DISCRIMINATOR: u64 = u64::from_le_bytes([102, 6, 61, 18, 1, 218, 235, 234]);
const SELL_DISCRIMINATOR: u64 = u64::from_le_bytes([51, 230, 133, 164, 1, 127, 131, 173]);
const CREATE_DISCRIMINATOR: u64 = u64::from_le_bytes( [24, 30, 200, 40, 5, 28, 7, 119]);

pub fn parse_trade_instruction(
    bytes_stream: &Vec<u8>,
    accounts: Vec<String>,
) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let discriminator: u64 = u64::from_le_bytes(disc_bytes.try_into().unwrap());

    match discriminator {
        BUY_DISCRIMINATOR | SELL_DISCRIMINATOR => {
            let name = if discriminator == BUY_DISCRIMINATOR { "Buy" } else { "Sell" };

            let amm = accounts.get(3).cloned();
            let vault_a = accounts.get(4).cloned();
            let vault_b = accounts.get(6).cloned();

            if let (Some(amm), Some(vault_a), Some(vault_b)) = (amm, vault_a, vault_b) {
                return Some(TradeInstruction {
                    program: PUMP_FUN_AMM_PROGRAM_ADDRESS.to_string(),
                    name: name.to_string(),
                    amm,
                    vault_a,
                    vault_b,
                    ..Default::default()
                });
            }
        }
        _ => {}
    }
    None
}
pub fn parse_pool_instruction(
    bytes_stream: Vec<u8>,
    accounts: Vec<String>,
) -> Option<CreatePoolInstruction> {
    let (disc_bytes, _) = bytes_stream.split_at(8);
    let discriminator: u64 = u64::from_le_bytes(disc_bytes.try_into().unwrap());

    if discriminator == CREATE_DISCRIMINATOR {
        let amm = accounts.get(2)?.to_string();
        let pc_mint = accounts.get(0)?.to_string();
        let is_pump_fun = accounts.get(13)? == PUMP_FUN_AMM_PROGRAM_ADDRESS;

        return Some(CreatePoolInstruction {
            program: PUMP_FUN_AMM_PROGRAM_ADDRESS.to_string(),
            name: "create".to_string(),
            amm,
            coin_mint: String::new(),
            pc_mint,
            is_pump_fun,
            ..Default::default()
        });
    }
    None
}


pub fn parse_reserves_instruction(
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    _: &Vec<String>,
    _: &String,
    _: &String
) -> (u64, u64) {
    for inner_instruction in inner_instructions {
        for inner_inst in &inner_instruction.instructions {
            let inner_program = &accounts[inner_inst.program_id_index as usize];
            if inner_program == PUMP_FUN_AMM_PROGRAM_ADDRESS {
                if let Some((reserves0, reserves1)) = parse_swap_event(&inner_inst.data) {
                    return (reserves0, reserves1);
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
