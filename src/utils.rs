extern crate chrono;

use crate::constants::{METEORA_POOL_PROGRAM_ADDRESS, MOONSHOT_ADDRESS, PUMP_FUN_AMM_PROGRAM_ADDRESS};
use crate::pb::sf::solana::dex::trades::v1::TradeData;
use borsh::{BorshDeserialize, BorshSerialize};
use chrono::prelude::*;
use substreams_solana::pb::sf::solana::r#type::v1::{InnerInstructions, TokenBalance};

pub const WSOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";
pub const USDT_ADDRESS: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const USDC_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

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
    dapp_address: &String,
) -> String {
    if dapp_address.eq(PUMP_FUN_AMM_PROGRAM_ADDRESS) || dapp_address.eq(MOONSHOT_ADDRESS) {
        return WSOL_ADDRESS.to_string();
    }
    let index = accounts.iter().position(|r| r == address).unwrap();
    let mut result: String = String::new();
    token_balances
        .iter()
        .filter(|token_balance| token_balance.account_index == index as u32)
        .for_each(|token_balance| {
            if token_balance.owner == "GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ" {
                return;
            }
            result = token_balance.mint.clone();
        });
    return result;
}

pub fn get_amt(
    amm: &String,
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
    dapp_address: &String,
    pre_balances: &Vec<u64>,
    post_balances: &Vec<u64>,
) -> (String, u32) {
    let mut result: String = "".to_string();
    let mut expont: u32 = 9;

    let source_transfer_amt = get_token_transfer(
        amm,
        address,
        input_inner_idx,
        inner_instructions,
        accounts,
        dapp_address,
        pre_balances,
        post_balances,
    );

    let destination_transfer_amt = get_token_transfer(
        amm,
        address,
        input_inner_idx,
        inner_instructions,
        accounts,
        dapp_address,
        pre_balances,
        post_balances,
    );

    if source_transfer_amt != "" && source_transfer_amt != "0" {
        result = source_transfer_amt;
    } else if destination_transfer_amt != "" && destination_transfer_amt != "0" {
        result = destination_transfer_amt;
    }

    if result != "" && result != "0" {
        let index = accounts.iter().position(|r| r == address).unwrap();
        post_token_balances
            .iter()
            .filter(|token_balance| token_balance.account_index == index as u32)
            .for_each(|token_balance: &TokenBalance| {
                let decimals = token_balance
                    .ui_token_amount
                    .as_ref()
                    .map(|amount| amount.decimals)
                    .unwrap();
                expont = decimals;
            });
    }

    (result, expont)
}

pub fn get_decimals(in_mint: &String, out_mint: &String, post_token_balances: &Vec<TokenBalance>) -> (u32, u32) {
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

struct TransferAmount {
    amount: u64,
    negative: bool,
}

impl TransferAmount {
    pub fn new(amount: u64, negative: bool) -> TransferAmount {
        TransferAmount { amount, negative }
    }
}

impl ToString for TransferAmount {
    fn to_string(&self) -> String {
        if self.negative {
            format!("-{}", self.amount)
        } else {
            self.amount.to_string()
        }
    }
}

pub fn get_token_transfer(
    amm: &String,
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    dapp_address: &String,
    pre_balances: &Vec<u64>,
    post_balances: &Vec<u64>,
) -> String {
    if dapp_address.eq(PUMP_FUN_AMM_PROGRAM_ADDRESS) || dapp_address.eq(MOONSHOT_ADDRESS) {
        return get_system_program_transfer(
            amm,
            address,
            input_inner_idx,
            inner_instructions,
            accounts,
            pre_balances,
            post_balances,
        );
    }

    let mut result: Option<TransferAmount> = None;

    inner_instructions.iter().for_each(|inner_instruction| {
        inner_instruction
            .instructions
            .iter()
            .enumerate()
            .for_each(|(inner_idx, inner_inst)| {
                let inner_program = &accounts[inner_inst.program_id_index as usize];

                if inner_program.as_str().eq("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
                    let (discriminator_bytes, rest) = inner_inst.data.split_at(1);
                    let discriminator: u8 = u8::from(discriminator_bytes[0]);

                    match discriminator {
                        3 => {
                            let input_accounts = prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap();
                            let destination = input_accounts.get(1).unwrap();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(*source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if result.is_none() {
                                    result = Some(TransferAmount::new(data.amount, true));
                                }
                            }

                            if condition && address.eq(*destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if result.is_none() {
                                    result = Some(TransferAmount::new(data.amount, false));
                                }
                            }
                        }
                        12 => {
                            let input_accounts = prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap();
                            let destination = input_accounts.get(2).unwrap();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(*source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if result.is_none() {
                                    result = Some(TransferAmount::new(data.amount, true));
                                }
                            }

                            if condition && address.eq(*destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if result.is_none() {
                                    result = Some(TransferAmount::new(data.amount, false));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
    });

    if result.is_none() {
        result = get_token_22_transfer(address, input_inner_idx, inner_instructions, accounts);
    }

    result.map(|r| r.to_string()).unwrap_or_default()
}

pub fn get_token_22_transfer(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
) -> Option<TransferAmount> {
    let mut result: Option<TransferAmount> = None;

    inner_instructions.iter().for_each(|inner_instruction| {
        inner_instruction
            .instructions
            .iter()
            .enumerate()
            .for_each(|(inner_idx, inner_inst)| {
                let inner_program = &accounts[inner_inst.program_id_index as usize];

                if inner_program.as_str().eq("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb") {
                    let (discriminator_bytes, rest) = inner_inst.data.split_at(1);
                    let discriminator: u8 = u8::from(discriminator_bytes[0]);

                    match discriminator {
                        3 => {
                            let input_accounts = prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(1).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if result.is_none() {
                                    result = Some(TransferAmount::new(data.amount, true));
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if result.is_none() {
                                    result = Some(TransferAmount::new(data.amount, false));
                                }
                            }
                        }
                        12 => {
                            let input_accounts = prepare_input_accounts(&inner_inst.accounts, accounts);

                            let source = input_accounts.get(0).unwrap().to_string();
                            let destination = input_accounts.get(2).unwrap().to_string();

                            let condition = if input_inner_idx > 0 {
                                inner_idx as u32 > input_inner_idx
                            } else {
                                true
                            };

                            if condition && address.eq(&source) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if result.is_none() {
                                    result = Some(TransferAmount::new(data.amount, true));
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if result.is_none() {
                                    result = Some(TransferAmount::new(data.amount, false));
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
    amm: &String,
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    pre_balances: &Vec<u64>,
    post_balances: &Vec<u64>,
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
                if inner_program.as_str().eq("11111111111111111111111111111111") {
                    let (discriminator_bytes, rest) = inner_inst.data.split_at(4);

                    let disc_bytes_arr: [u8; 4] = discriminator_bytes.to_vec().try_into().unwrap();
                    let discriminator: u32 = u32::from_le_bytes(disc_bytes_arr);

                    match discriminator {
                        2 => {
                            let input_accounts = prepare_input_accounts(&inner_inst.accounts, accounts);

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
                } else if inner_program.as_str().eq(PUMP_FUN_AMM_PROGRAM_ADDRESS)
                    || inner_program.as_str().eq(MOONSHOT_ADDRESS)
                {
                    let (_, rest) = inner_inst.data.split_at(16);
                    let mut rest_slice = &mut &rest[..];
                    match PumpEventLayout::deserialize(&mut rest_slice) {
                        Ok(event) => {
                            if !result_assigned {
                                result = event.sol_amount.to_string();
                                result_assigned = true;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to deserialize TradeEvent: {}", e);
                            return;
                        }
                    };
                }
            })
    });

    if !result_assigned {
        let index = accounts.iter().position(|r| r == amm).unwrap();
        let _result = post_balances[index] as f64 - pre_balances[index] as f64;
        result = _result.to_string()
    }
    result
}

pub fn parse_reserves_instruction(
    dapp_address: String,
    amm: &String,
    accounts: &Vec<String>,
    token_balances: &Vec<TokenBalance>,
    post_balances: &Vec<u64>,
    vault_a: &String,
    vault_b: &String,
    token0: &String,
    token1: &String,
) -> (u64, u64) {
    let index_a = accounts.iter().position(|r| r == vault_a).unwrap_or_else(|| {
        panic!("Vault A not found in accounts");
    });
    let index_b = accounts.iter().position(|r| r == vault_b).unwrap_or_else(|| {
        panic!("Vault B not found in accounts");
    });

    let mut reserves0: u64 = 0;
    let mut reserves1: u64 = 0;

    token_balances
        .iter()
        .filter(|token_balance| {
            token_balance.account_index == index_a as u32 || token_balance.account_index == index_b as u32
        })
        .filter(|token_balance| {
            dapp_address.eq(METEORA_POOL_PROGRAM_ADDRESS)
                || (dapp_address.ne(METEORA_POOL_PROGRAM_ADDRESS) && token_balance.owner == amm.clone())
        }) // 仅处理匹配 amm 的记录
        .for_each(|token_balance| {
            if let Some(ref ui_token_amount) = token_balance.ui_token_amount {
                if token_balance.mint == token0.clone() {
                    reserves0 = ui_token_amount.amount.parse::<u64>().unwrap_or(0);
                } else if token_balance.mint == token1.clone() {
                    reserves1 = ui_token_amount.amount.parse::<u64>().unwrap_or(0);
                }
            }
        });
    if dapp_address.eq(MOONSHOT_ADDRESS) {
        let index = accounts.iter().position(|r| r == amm).unwrap_or_else(|| {
            panic!("amm not found in accounts");
        });
        if reserves0 == 0 {
            reserves0 = post_balances[index];
        }
        if reserves1 == 0 {
            reserves1 = post_balances[index];
        }
    }
    (reserves0, reserves1)
}

pub fn prepare_input_accounts<'a>(account_indices: &'_ Vec<u8>, accounts: &'a Vec<String>) -> Vec<&'a String> {
    let mut instruction_accounts: Vec<&String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        instruction_accounts.push(&accounts.get(el as usize).unwrap());
    }
    return instruction_accounts;
}

pub fn get_b58_string(data: [u8; 32]) -> Option<String> {
    return Some(bs58::encode(data).into_string());
}

pub fn is_not_soltoken(token0: &String, token1: &String) -> bool {
    return token0.to_string() != WSOL_ADDRESS.to_string() && token1.to_string() != WSOL_ADDRESS.to_string();
}

pub fn find_sol_stable_coin_trade(data: &Vec<TradeData>) -> Option<&TradeData> {
    data.iter().find(|trade| {
        let (base_mint, quote_mint) = (&trade.base_mint, &trade.quote_mint);

        let is_usdc_usdt_to_sol =
            (base_mint == USDT_ADDRESS || base_mint == USDC_ADDRESS) && quote_mint == WSOL_ADDRESS;

        let is_sol_to_usdc_usdt =
            base_mint == WSOL_ADDRESS && (quote_mint == USDT_ADDRESS || quote_mint == USDC_ADDRESS);

        is_usdc_usdt_to_sol || is_sol_to_usdc_usdt
    })
}
