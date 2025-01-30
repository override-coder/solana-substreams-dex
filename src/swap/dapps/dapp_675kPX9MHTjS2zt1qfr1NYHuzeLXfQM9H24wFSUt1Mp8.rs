use std::f32::consts::E;

use crate::constants::RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS;
use crate::swap::trade_instruction::TradeInstruction;
use crate::utils::get_mint;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use substreams_solana::pb::sf::solana::r#type::v1::TokenBalance;

use super::EMPTY_STRING;

pub const INSTRUCTION_TYPE_INITIALIZE: &str = "initialize";
pub const INSTRUCTION_TYPE_INITIALIZE2: &str = "initialize2";
pub const INSTRUCTION_TYPE_SWAPBASE_IN: &str = "SwapBaseIn";
pub const INSTRUCTION_TYPE_SWAPBASE_OUT: &str = "SwapBaseOut";

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
    input_accounts: &Vec<&String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> Option<TradeInstruction> {
    let discriminator = bytes_stream[0];

    match discriminator {
        9 => Some(TradeInstruction {
            program: String::from(RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS),
            name: String::from(INSTRUCTION_TYPE_SWAPBASE_IN),
            amm: input_accounts.get(1)?.to_string(),
            vault_a: get_vault_a(input_accounts, post_token_balances, accounts),
            vault_b: get_vault_b(input_accounts, post_token_balances, accounts),
            ..Default::default()
        }),
        11 => Some(TradeInstruction {
            program: String::from(RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS),
            name: String::from(INSTRUCTION_TYPE_SWAPBASE_OUT),
            amm: input_accounts.get(1)?.to_string(),
            vault_a: get_vault_a(&input_accounts, post_token_balances, accounts),
            vault_b: get_vault_b(&input_accounts, post_token_balances, accounts),
            ..Default::default()
        }),
        _ => None,
    }
}

fn get_vault_a(
    input_accounts: &Vec<&String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> String {
    let mut vault_a = input_accounts.get(4).unwrap().to_string();
    let mint_a = get_mint(&vault_a, post_token_balances, accounts, &EMPTY_STRING);

    if mint_a.is_empty() {
        vault_a = input_accounts.get(5).unwrap().to_string();
    }

    return vault_a;
}

fn get_vault_b(
    input_accounts: &Vec<&String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> String {
    let mut vault_a_index = 4;

    let mut vault_a = input_accounts.get(4).unwrap().to_string();
    let mint_a = get_mint(&vault_a, post_token_balances, accounts, &EMPTY_STRING);

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

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct SwapEvent {
    pool_coin: String,
    pool_pc: String,
}

pub fn parse_logs(log_messages: &Vec<String>) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    for log_message in log_messages {
        if log_message.starts_with("Program log: ") && log_message.contains("ray_log") {
            let swap_log_value = log_message.replace("Program log: ray_log: ", "").trim().to_string();
            results.push(swap_log_value);
        }
    }
    results
}

fn is_matching(amount_in: u64, amount_out: u64, amount0: &String, amount1: &String) -> bool {
    let amount0_parsed = amount0.trim_start_matches('-').parse::<u64>().unwrap_or(0);
    let amount1_parsed = amount1.trim_start_matches('-').parse::<u64>().unwrap_or(0);
    let amount0_comparison = if amount0.starts_with('-') {
        amount_out == amount0_parsed
    } else {
        amount_in == amount0_parsed
    };
    let amount1_comparison = if amount1.starts_with('-') {
        amount_out == amount1_parsed
    } else {
        amount_in == amount1_parsed
    };
    amount0_comparison && amount1_comparison
}
