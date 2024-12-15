extern crate chrono;

use std::collections::HashSet;
use borsh::{BorshSerialize, BorshDeserialize};
use chrono::format::parse;
use chrono::prelude::*;
use substreams_solana::pb::sf::solana::r#type::v1::{InnerInstructions, TokenBalance};
use crate::constants::{JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS, PUMP_FUN_AMM_PROGRAM_ADDRESS};
use crate::pb::sf::solana::dex::trades::v1::TradeData;

pub const WSOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";
pub const USDT_ADDRESS: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const USDC_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

const VALID_POOLS: [&str; 6] = [
    "7XawhbbxtsRcQA8KTkHT9f9nc6d69UwqCDh6U5EEbEmX",
    "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2",
    "3nMFwZXwY1s1M5s8vYAHqd4wGs4iSxXE4LRoUMMYqEgF",
    "8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj",
    "ExcBWu8fGPdJiaF1b1z3iEef38sjQJks8xvj6M85pPY6",
    "CYbD9RaToYMtWKA7QZyoLahnHdWq553Vm62Lh6qWtuxq",
];

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct TransferLayout {
    amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct PumpEventLayout {
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

pub fn convert_to_date(ts: i64) -> String {
    let nt = NaiveDateTime::from_timestamp_opt(ts, 0);
    let dt: DateTime<Utc> = DateTime::from_naive_utc_and_offset(nt.unwrap(), Utc);
    let res = dt.format("%Y-%m-%d");
    return res.to_string();
}

pub fn get_mint(
    address: &String,
    token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
    dapp_address: String,
) -> String {
    if dapp_address.eq(PUMP_FUN_AMM_PROGRAM_ADDRESS)
    {
        return WSOL_ADDRESS.to_string();
    }
    let index = accounts.iter().position(|r| r == address).unwrap();
    let mut result: String = String::new();
    token_balances
        .iter()
        .filter(|token_balance| token_balance.account_index == index as u32)
        .for_each(|token_balance| {
            result = token_balance.mint.clone();
        });
    return result;
}

pub fn get_amt(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    dapp_address: String,
) -> (String,u32) {
    let mut result: String = "".to_string();
    let mut expont: u32 = 9;

    let source_transfer_amt = get_token_transfer(
        address,
        input_inner_idx,
        inner_instructions,
        accounts,
        "source".to_string(),
        dapp_address.clone(),
    );

    let destination_transfer_amt = get_token_transfer(
        address,
        input_inner_idx,
        inner_instructions,
        accounts,
        "destination".to_string(),
        dapp_address.clone(),
    );

    if source_transfer_amt != "" && source_transfer_amt != "0"{
        result = source_transfer_amt;
    } else if destination_transfer_amt != "" && destination_transfer_amt != "0" {
        result = destination_transfer_amt;
    }

    if result != "" && result != "0"  {
        let index = accounts.iter().position(|r| r == address).unwrap();
        post_token_balances
            .iter()
            .filter(|token_balance| token_balance.account_index == index as u32)
            .for_each(|token_balance: &TokenBalance| {
                let decimals = token_balance.ui_token_amount.clone().unwrap().decimals;
                expont = decimals;
            });
    }

    (result,expont)
}

pub fn get_decimals(
    in_mint: &String,
    out_mint: &String,
    post_token_balances: &Vec<TokenBalance>,
) -> (u32, u32) {
    fn find_decimals(mint: &String, balances: &Vec<TokenBalance>) -> u32 {
        balances
            .iter()
            .find(|token_balance| token_balance.mint == *mint)
            .and_then(|token_balance| token_balance.ui_token_amount.as_ref())
            .map_or(0, |ui_token_amount| ui_token_amount.decimals)
    }
    let in_decimals = find_decimals(in_mint, post_token_balances);
    let out_decimals = find_decimals(out_mint, post_token_balances);
    (in_decimals, out_decimals)
}

pub fn get_token_transfer(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    account_name_to_check: String,
    dapp_address: String,
) -> String {
    if dapp_address.eq(PUMP_FUN_AMM_PROGRAM_ADDRESS)
    {
        return get_system_program_transfer(
            address,
            input_inner_idx,
            inner_instructions,
            accounts,
            account_name_to_check,
        );
    }

    let mut result = "".to_string();
    let mut result_assigned = false;

    inner_instructions.iter().for_each(|inner_instruction| {
        inner_instruction
            .instructions
            .iter()
            .enumerate()
            .for_each(|(inner_idx, inner_inst)| {
                let inner_program = &accounts[inner_inst.program_id_index as usize];

                if inner_program
                    .as_str()
                    .eq("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")
                {
                    let (discriminator_bytes, rest) = inner_inst.data.split_at(1);
                    let discriminator: u8 = u8::from(discriminator_bytes[0]);

                    match discriminator {
                        3 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(1).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = format!("-{}", data.amount.to_string());
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = data.amount.to_string();
                                    result_assigned = true;
                                }
                            }
                        }
                        12 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(2).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = format!("-{}", data.amount.to_string());
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = data.amount.to_string();
                                    result_assigned = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
    });

    if !result_assigned {
        let _result = get_token_22_transfer(
            address,
            input_inner_idx,
            inner_instructions,
            accounts,
            account_name_to_check,
        );
        if _result.is_some() {
            result = _result.unwrap();
        }
    }

    result
}

pub fn get_token_22_transfer(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    account_name_to_check: String,
) -> Option<String> {
    let mut result = None;
    let mut result_assigned = false;

    inner_instructions.iter().for_each(|inner_instruction| {
        inner_instruction
            .instructions
            .iter()
            .enumerate()
            .for_each(|(inner_idx, inner_inst)| {
                let inner_program = &accounts[inner_inst.program_id_index as usize];

                if inner_program
                    .as_str()
                    .eq("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
                {
                    let (discriminator_bytes, rest) = inner_inst.data.split_at(1);
                    let discriminator: u8 = u8::from(discriminator_bytes[0]);

                    match discriminator {
                        3 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(1).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(format!("-{}", data.amount.to_string()));
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(data.amount.to_string());
                                    result_assigned = true;
                                }
                            }
                        }
                        12 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(2).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(format!("-{}", data.amount.to_string()));
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(data.amount.to_string());
                                    result_assigned = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
    });

    result
}

fn get_system_program_transfer(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    account_name_to_check: String,
) -> String {
    let mut result = "".to_string();
    let mut result_assigned = false;

    inner_instructions.iter().for_each(|inner_instruction| {
        inner_instruction
            .instructions
            .iter()
            .enumerate()
            .for_each(|(inner_idx, inner_inst)| {
                let inner_program = &accounts[inner_inst.program_id_index as usize];
                if inner_program
                    .as_str()
                    .eq("11111111111111111111111111111111")
                {
                    let (discriminator_bytes, rest) = inner_inst.data.split_at(4);

                    let disc_bytes_arr: [u8; 4] = discriminator_bytes.to_vec().try_into().unwrap();
                    let discriminator: u32 = u32::from_le_bytes(disc_bytes_arr);

                    match discriminator {
                        2 => {
                            let input_accounts =
                                prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(1).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = format!("-{}", data.amount.to_string());
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = data.amount.to_string();
                                    result_assigned = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }else if inner_program
                    .as_str()
                    .eq(PUMP_FUN_AMM_PROGRAM_ADDRESS)  {
                    let (_, rest) = inner_inst.data.split_at(16);
                    let mut rest_slice = &mut &rest[..];
                    match PumpEventLayout::deserialize(&mut rest_slice) {
                        Ok(event) => {
                            if !result_assigned {
                                result = event.sol_amount.to_string();
                                result_assigned = true;

                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to deserialize TradeEvent: {}", e);
                            return;
                        }
                    };
                }
            })
    });
    result
}

pub fn prepare_input_accounts(account_indices: &Vec<u8>, accounts: &Vec<String>) -> Vec<String> {
    let mut instruction_accounts: Vec<String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        instruction_accounts.push(accounts.as_slice()[el as usize].to_string());
    }
    return instruction_accounts;
}

pub fn get_b58_string(data: [u8; 32]) -> Option<String> {
    return Some(bs58::encode(data).into_string());
}

pub fn is_not_soltoken(token0: &String, token1: &String) -> bool{
   return  token0.to_string() != WSOL_ADDRESS.to_string() && token1.to_string() != WSOL_ADDRESS.to_string()
}

pub fn get_wsol_price(pool_address:&str, base_mint: &str, quote_mint: &str,base_amount: &String, quote_amount: &String) -> Option<f64> {
    if !validation_pool(pool_address){
        return None;
    }
    match (base_mint, quote_mint) {
        (USDT_ADDRESS | USDC_ADDRESS, WSOL_ADDRESS) => {
            calculate_wsol_price(base_amount, quote_amount, 1e6, 1e9)
        }
        (WSOL_ADDRESS, USDT_ADDRESS | USDC_ADDRESS) => {
            calculate_wsol_price(quote_amount, base_amount, 1e6, 1e9)
        }
        _ => None,
    }
}

fn calculate_wsol_price(base_amount: &str, quote_amount: &str, base_divisor: f64, quote_divisor: f64) -> Option<f64> {
    let base = base_amount.parse::<f64>().unwrap_or(0.0);
    let quote = quote_amount.parse::<f64>().unwrap_or(0.0);
    if base == 0.0 || quote == 0.0 {
        return None;
    }
    Some((base / base_divisor) / (quote / quote_divisor))
}

pub fn validation_pool(pool_address: &str) -> bool {
    static VALID_POOL_SET: once_cell::sync::Lazy<HashSet<&'static str>> = once_cell::sync::Lazy::new(|| {
        VALID_POOLS.iter().cloned().collect()
    });
    VALID_POOL_SET.contains(pool_address)
}

pub fn calculate_price_and_amount_usd(
    base_mint:  &str,
    quote_mint: &str,
    base_amount: &String,
    quote_amount: &String,
    base_decimals: u32,
    quote_decimals: u32,
    wsol_price: f64,
) -> (f64, f64, f64) {
        if base_mint != WSOL_ADDRESS && quote_mint != WSOL_ADDRESS {
            return (0.0, 0.0, 0.0)
        }
        let base_amount_normalized = base_amount.parse::<f64>().unwrap_or(0.0) / 10f64.powi(base_decimals as i32);
        let quote_amount_normalized = quote_amount.parse::<f64>().unwrap_or(0.0) / 10f64.powi(quote_decimals as i32);
        let amount_usd = if base_mint == WSOL_ADDRESS {
            base_amount_normalized * wsol_price
        } else if quote_mint == WSOL_ADDRESS {
            quote_amount_normalized * wsol_price
        } else {
            0.0
        };
        let price = if base_mint == WSOL_ADDRESS {
            amount_usd / quote_amount_normalized
        } else if quote_mint == WSOL_ADDRESS {
            amount_usd / base_amount_normalized
        } else {
            0.0
        };
        return (price.abs(), amount_usd.abs(), wsol_price.abs())
}

pub fn find_sol_stable_coin_trade(data: &Vec<TradeData>) -> Option<&TradeData> {
    data.iter().find(|trade| {
        let (base_mint, quote_mint) = (&trade.base_mint, &trade.quote_mint);

        let is_usdc_usdt_to_sol = (base_mint == USDT_ADDRESS || base_mint == USDC_ADDRESS) && quote_mint == WSOL_ADDRESS;

        let is_sol_to_usdc_usdt = base_mint == WSOL_ADDRESS && (quote_mint == USDT_ADDRESS || quote_mint == USDC_ADDRESS);

        is_usdc_usdt_to_sol || is_sol_to_usdc_usdt
    })
}