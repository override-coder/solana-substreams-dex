use crate::trade_instruction::{CreatePoolInstruction, TradeInstruction};
use crate::utils::{TradeEvent};
use borsh::{ BorshDeserialize};
use substreams_solana::pb::sf::solana::r#type::v1::InnerInstructions;
use crate::constants::PUMP_FUN_AMM_PROGRAM_ADDRESS;

const BUY_DISCRIMINATOR: u64 = u64::from_le_bytes([102, 6, 61, 18, 1, 218, 235, 234]);
const SELL_DISCRIMINATOR: u64 = u64::from_le_bytes([51, 230, 133, 164, 1, 127, 131, 173]);
const CREATE_DISCRIMINATOR: u64 = u64::from_le_bytes( [24, 30, 200, 40, 5, 28, 7, 119]);


pub fn parse_trade_instruction(
    bytes_stream: &Vec<u8>,
    accounts: Vec<String>,
) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);
    let mut result = None;
    match discriminator {
        BUY_DISCRIMINATOR => {
            result = Some(TradeInstruction {
                program: String::from(PUMP_FUN_AMM_PROGRAM_ADDRESS.to_string()),
                name: String::from("buy"),
               amm: accounts.get(3).unwrap().to_string(),
               vault_a: accounts.get(4).unwrap().to_string(),
               vault_b: accounts.get(6).unwrap().to_string(),
                ..Default::default()
            });
        }
        SELL_DISCRIMINATOR => {
            result = Some(TradeInstruction {
                program: String::from(PUMP_FUN_AMM_PROGRAM_ADDRESS.to_string()),
                name: String::from("sell"),
                amm: accounts.get(3).unwrap().to_string(),
                vault_a: accounts.get(4).unwrap().to_string(),
                vault_b: accounts.get(6).unwrap().to_string(),
                ..Default::default()
            });
        }
        _ => {}
    }
    return result;
}

pub fn parse_pool_instruction(
    bytes_stream: Vec<u8>,
    accounts: Vec<String>,
) -> Option<CreatePoolInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut result = None;

    match discriminator {
        CREATE_DISCRIMINATOR => {
            result = Some(CreatePoolInstruction {
                program: String::from(PUMP_FUN_AMM_PROGRAM_ADDRESS.to_string()),
                name: String::from("create"),
                amm: accounts.get(2).unwrap().to_string(),
                coin_mint: "".to_string(),
                pc_mint: accounts.get(0).unwrap().to_string() ,
                is_pump_fun: accounts.get(13).unwrap().to_string() == PUMP_FUN_AMM_PROGRAM_ADDRESS.to_string(),
                ..Default::default()
            });
        }
        _ => {}
    }
    return result;
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
