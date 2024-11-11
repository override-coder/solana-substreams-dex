extern crate chrono;
use borsh::{BorshSerialize, BorshDeserialize};
use chrono::prelude::*;
use serde::Deserialize;
use substreams_solana::pb::sf::solana::r#type::v1::{InnerInstructions, TokenBalance};

pub const WSOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";
pub const USDT_ADDRESS: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const USDC_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct TransferLayout {
    amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
struct TradeEvent {
    mint: [u8; 32],
    sol_amount: u64,
    token_amount: u64,
    is_buy: bool,
    user: [u8; 32],
    timestamp: i64,
    virtual_sol_reserves: u64,
    virtual_token_reserves: u64,
    real_sol_reserves: u64,
    real_token_reserves: u64,
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
) -> String {
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
) -> (i64,u32) {
    let mut result: i64 = 0;
    let mut expont: u32 = 9;

    let source_transfer_amt = get_token_transfer(
        address,
        input_inner_idx,
        inner_instructions,
        accounts,
        "source".to_string(),
    );

    let destination_transfer_amt = get_token_transfer(
        address,
        input_inner_idx,
        inner_instructions,
        accounts,
        "destination".to_string(),
    );

    if source_transfer_amt != 0 {
        result = source_transfer_amt;
    } else if destination_transfer_amt != 0 {
        result = destination_transfer_amt;
    }

    if result != 0 {
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

pub fn get_token_transfer(
    address: &String,
    input_inner_idx: u32,
    inner_instructions: &Vec<InnerInstructions>,
    accounts: &Vec<String>,
    account_name_to_check: String,
) -> i64 {
    let mut result = 0;
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
                                    let amount_i64 = data.amount as i64;
                                    result = -amount_i64;
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = data.amount as i64;
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
                                    let amount_i64 = data.amount as i64;
                                    result = -amount_i64;
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = data.amount as i64;
                                    result_assigned = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }else if inner_program
                    .as_str()
                    .eq("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P")  {
                    let (_, rest) = inner_inst.data.split_at(16);
                    let mut rest_slice = &mut &rest[..];
                    match TradeEvent::deserialize(&mut rest_slice) {
                        Ok(event) => {
                            if !result_assigned {
                                result = event.sol_amount as i64;
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
) -> Option<i64> {
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
                                    let amount_i64 = data.amount as i64;
                                    result = Some(-amount_i64);
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(data.amount as i64);
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
                                    let amount_i64 = data.amount as i64;
                                    result = Some(-amount_i64);
                                    result_assigned = true;
                                }
                            }

                            if condition && address.eq(&destination) {
                                let data = TransferLayout::deserialize(&mut rest.clone()).unwrap();
                                if !result_assigned {
                                    result = Some(data.amount as i64);
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

fn prepare_input_accounts(account_indices: &Vec<u8>, accounts: &Vec<String>) -> Vec<String> {
    let mut instruction_accounts: Vec<String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        instruction_accounts.push(accounts.as_slice()[el as usize].to_string());
    }
    return instruction_accounts;
}

pub fn is_not_soltoken(token0: &String, token1: &String) -> bool{
   return  token0.to_string() != WSOL_ADDRESS.to_string() && token1.to_string() != WSOL_ADDRESS.to_string()
}

pub fn get_wsol_price(base_mint: &str, quote_mint: &str,base_amount: i64, quote_amount: i64) -> f64 {
    if (base_mint == USDT_ADDRESS || base_mint == USDC_ADDRESS) && quote_mint == WSOL_ADDRESS { return  (base_amount as f64 / 1e6) / (quote_amount as f64 / 1e9);
    } else if base_mint == WSOL_ADDRESS && (quote_mint == USDT_ADDRESS || quote_mint == USDC_ADDRESS) { return (quote_amount as f64 / 1e6) / (base_amount as f64 / 1e9);
    } else { return 0.0; }
}

pub fn calculate_price_and_amount_usd(
    base_mint: &str,
    quote_mint: &str,
    base_amount: i64,
    quote_amount: i64,
    base_decimals: u32,
    quote_decimals: u32,
    wsol_price: f64,
) -> (String, String) {
    let base_amount_normalized = base_amount as f64 / 10f64.powi(base_decimals as i32);
    let quote_amount_normalized = quote_amount as f64 / 10f64.powi(quote_decimals as i32);
    let amount_usd = if base_mint == WSOL_ADDRESS {
        (base_amount as f64 / 1e9) *  wsol_price
    } else  {
        (quote_amount as f64 / 1e9) *  wsol_price
    };
    let price = if base_mint == WSOL_ADDRESS {
        (amount_usd / quote_amount_normalized).to_string()
    } else  {
        (amount_usd / base_amount_normalized).to_string()
    };
    (price, amount_usd.to_string())
}
