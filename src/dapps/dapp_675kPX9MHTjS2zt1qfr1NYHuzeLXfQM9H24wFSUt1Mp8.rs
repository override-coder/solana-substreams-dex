use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use substreams_solana::pb::sf::solana::r#type::v1::{InnerInstructions, TokenBalance};

use crate::trade_instruction::{CreatePoolInstruction, TradeInstruction};

use crate::utils::{get_mint, WSOL_ADDRESS};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SwapBaseInLog {
    pub log_type: u8,
    pub amount_in: u64,
    pub minimum_out: u64,
    pub direction: u64,
    pub user_source: u64,
    pub pool_coin: u64,
    pub pool_pc: u64,
    pub out_amount: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SwapBaseOutLog {
    pub log_type: u8,
    pub max_in: u64,
    pub amount_out: u64,
    pub direction: u64,
    pub user_source: u64,
    pub pool_coin: u64,
    pub pool_pc: u64,
    pub deduct_in: u64,
}

pub fn parse_trade_instruction(
    bytes_stream: &Vec<u8>,
    input_accounts: Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let discriminator: u8 = u8::from(disc_bytes[0]);

    let mut result = None;

    match discriminator {
        9 => {
            result = Some(TradeInstruction {
                program: String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                name: String::from("SwapBaseIn"),
                amm: input_accounts.get(1).unwrap().to_string(),
                vault_a: get_vault_a(&input_accounts, post_token_balances, accounts),
                vault_b: get_vault_b(&input_accounts, post_token_balances, accounts),
                ..Default::default()
            });
        }
        11 => {
            result = Some(TradeInstruction {
                program: String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                name: String::from("SwapBaseOut"),
                amm: input_accounts.get(1).unwrap().to_string(),
                vault_a: get_vault_a(&input_accounts, post_token_balances, accounts),
                vault_b: get_vault_b(&input_accounts, post_token_balances, accounts),
                ..Default::default()
            });
        }
        _ => {}
    }

    return result;
}

fn get_vault_a(
    input_accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> String {
    let mut vault_a = input_accounts.get(4).unwrap().to_string();
    let mint_a = get_mint(&vault_a, post_token_balances, accounts);

    if mint_a.is_empty() {
        vault_a = input_accounts.get(5).unwrap().to_string();
    }

    return vault_a;
}

fn get_vault_b(
    input_accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> String {
    let mut vault_a_index = 4;

    let mut vault_a = input_accounts.get(4).unwrap().to_string();
    let mint_a = get_mint(&vault_a, post_token_balances, accounts);

    if mint_a.is_empty() {
        vault_a_index += 1;
        vault_a = input_accounts.get(vault_a_index).unwrap().to_string();
    }

    let mut vault_b_index = vault_a_index + 1;
    let mut vault_b = input_accounts.get(vault_b_index).unwrap().to_string();

    if vault_a == vault_b {
        vault_b_index += 1;
        vault_b = input_accounts.get(vault_b_index).unwrap().to_string();
    }
    return vault_b;
}


pub fn parse_pool_instruction(
    bytes_stream: Vec<u8>,
    input_accounts: Vec<String>,
) -> Option<CreatePoolInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let discriminator: u8 = u8::from(disc_bytes[0]);
    let mut result = None;
    match discriminator {
        0 => {
            result = Some(CreatePoolInstruction {
                program: String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                name: String::from("initialize"),
                amm: input_accounts.get(4).unwrap().to_string(),
                coin_mint: input_accounts.get(8).unwrap().to_string(),
                pc_mint: input_accounts.get(9).unwrap().to_string() ,
                is_pump_fun: input_accounts.get(17).unwrap().to_string() == "39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg",
                ..Default::default()
            });
        }
        1 => {
            result = Some(CreatePoolInstruction {
                program: String::from("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
                name: String::from("initialize2"),
                amm: input_accounts.get(4).unwrap().to_string(),
                coin_mint: input_accounts.get(8).unwrap().to_string(),
                pc_mint: input_accounts.get(9).unwrap().to_string() ,
                is_pump_fun: input_accounts.get(17).unwrap().to_string() == "39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg",
                ..Default::default()
            });
        }
        _ => {}
    }
    return result;
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct SwapEvent {
    pool_coin: String,
    pool_pc : String
}

pub fn parse_reserves_instruction(
    _: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    log_messages: &Vec<String>,
    token0: &String,
    token1: &String,
) -> (u64, u64) {
    if let Some(message) = parse_logs(log_messages) {
        if let Ok(bytes) = base64::decode_config(message, base64::STANDARD) {
            match bytes.get(0) {
                Some(3) => {
                    if let Ok(log) = bincode::deserialize::<SwapBaseInLog>(&bytes) {
                        return get_reserves_for_token(token0, token1, &log.pool_coin, &log.pool_pc);
                    }
                }
                Some(4) => {
                    if let Ok(log) = bincode::deserialize::<SwapBaseOutLog>(&bytes) {
                        return get_reserves_for_token(token0, token1, &log.pool_coin, &log.pool_pc);
                    }
                }
                _ => {}
            }
        }
    }
    (0, 0)
}

fn get_reserves_for_token(
    token0: &String,
    token1: &String,
    pool_coin: &u64,
    pool_pc: &u64,
) -> (u64, u64) {
    if token0 == WSOL_ADDRESS {
        (*pool_coin, *pool_pc)
    } else if token1 == WSOL_ADDRESS {
        (*pool_pc, *pool_coin)
    } else {
        (0, 0)
    }
}

pub fn parse_logs(log_messages: &Vec<String>) -> Option<String> {
    let mut result: Option<String> = None;
    for log_message in log_messages {
        if log_message.starts_with("Program log: ") & log_message.contains("ray_log") {
            let swap_log_value = log_message
                .replace("Program log: ray_log: ", "")
                .trim()
                .to_string();
            result = Some(swap_log_value);
        }
    }
    return result;
}