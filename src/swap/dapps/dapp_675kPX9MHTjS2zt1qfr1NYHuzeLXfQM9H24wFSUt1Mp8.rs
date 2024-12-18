use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use substreams_solana::pb::sf::solana::r#type::v1::{InnerInstructions, TokenBalance};
use crate::constants::{PUMP_FUN_RAYDIUM_MIGRATION, RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS};
use crate::swap::trade_instruction::{CreatePoolInstruction, TradeInstruction};
use crate::utils::{get_mint, WSOL_ADDRESS};

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
    input_accounts: Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> Option<TradeInstruction> {
    let discriminator = bytes_stream[0];

    let amm = input_accounts.get(1)?.to_string();
    let vault_a = get_vault_a(&input_accounts, post_token_balances, accounts);
    let vault_b = get_vault_b(&input_accounts, post_token_balances, accounts);

    match discriminator {
        9 => Some(TradeInstruction {
            program: String::from(RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS),
            name: String::from(INSTRUCTION_TYPE_SWAPBASE_IN),
            amm,
            vault_a,
            vault_b,
            ..Default::default()
        }),
        11 => Some(TradeInstruction {
            program: String::from(RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS),
            name: String::from(INSTRUCTION_TYPE_SWAPBASE_OUT),
            amm,
            vault_a,
            vault_b,
            ..Default::default()
        }),
        _ => None,
    }
}


fn get_vault_a(
    input_accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> String {
    let mut vault_a = input_accounts.get(4).unwrap().to_string();
    let mint_a = get_mint(&vault_a, post_token_balances, accounts,"".to_string());

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
    let mint_a = get_mint(&vault_a, post_token_balances, accounts,"".to_string());

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
    let discriminator = bytes_stream[0];

    let amm = input_accounts.get(4)?.to_string();
    let coin_mint = input_accounts.get(8)?.to_string();
    let pc_mint = input_accounts.get(9)?.to_string();
    let is_pump_fun = input_accounts.get(17)?.to_string() == PUMP_FUN_RAYDIUM_MIGRATION;

    match discriminator {
        0 | 1 => Some(CreatePoolInstruction {
            program: String::from(RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS),
            name: match discriminator {
                0 => String::from(INSTRUCTION_TYPE_INITIALIZE),
                1 => String::from(INSTRUCTION_TYPE_INITIALIZE2),
                _ => "Unknown".parse().unwrap(),
            },
            amm,
            coin_mint,
            pc_mint,
            is_pump_fun,
            ..Default::default()
        }),
        _ => None,
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct SwapEvent {
    pool_coin: String,
    pool_pc : String
}

pub fn parse_reserves_instruction(
    _: &Vec<InnerInstructions>,
    _: &Vec<String>,
    log_messages: &Vec<String>,
    amount0: &String,
    amount1: &String,
) -> (u64, u64) {
    let mut pool_coin = 0u64;
    let mut pool_pc = 0u64;

    let logs = parse_logs(log_messages);
    for log in logs {
        if let Ok(bytes) = base64::decode_config(&log, base64::STANDARD) {
            match bytes.get(0) {
                Some(3) => {
                    if let Ok(log) = bincode::deserialize::<SwapBaseInLog>(&bytes) {
                        pool_coin = log.pool_coin;
                        pool_pc = log.pool_pc;
                        let amount_in = log.amount_in;
                        let amount_out = log.out_amount;
                        if is_matching(amount_in,amount_out, amount0, amount1) {
                            return (pool_coin, pool_pc);
                        }
                    }
                }
                Some(4) => {
                    if let Ok(log) = bincode::deserialize::<SwapBaseOutLog>(&bytes) {
                        pool_coin = log.pool_coin;
                        pool_pc = log.pool_pc;
                        let amount_in = log.max_in;
                        let amount_out = log.amount_out;
                        if is_matching(amount_in,amount_out, amount0, amount1) {
                            return (pool_coin, pool_pc);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    (0, 0)
}

pub fn parse_logs(log_messages: &Vec<String>) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    for log_message in log_messages {
        if log_message.starts_with("Program log: ") && log_message.contains("ray_log") {
            let swap_log_value = log_message
                .replace("Program log: ray_log: ", "")
                .trim()
                .to_string();
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