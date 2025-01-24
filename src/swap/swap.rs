use crate::constants::{
    METEORA_POOL_PROGRAM_ADDRESS, METEORA_PROGRAM_ADDRESS, MOONSHOT_ADDRESS, ORCA_PROGRAM_ADDRESS,
    PUMP_FUN_AMM_PROGRAM_ADDRESS, RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS, RAYDIUM_CPMM_ADDRESS,
    RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS,
};
use crate::pb::sf::solana::dex::trades::v1::{Pool, Pools, Swaps, TradeData};
use crate::swap::dapps;
use crate::swap::trade_instruction::TradeInstruction;
use crate::utils::{
    find_sol_stable_coin_trade, get_amt, get_mint, is_not_soltoken, parse_reserves_instruction, prepare_input_accounts,
    WSOL_ADDRESS,
};

static EMPTY_STRING: String = String::new();

use std::string::ToString;
use substreams::errors::Error;
use substreams::log;
use substreams_solana::pb::sf::solana::r#type::v1::{Block, InnerInstructions, TokenBalance};

#[substreams::handlers::map]
pub fn map_swap_block(block: Block) -> Result<Swaps, Error> {
    process_block(block)
}

fn process_block(block: Block) -> Result<Swaps, Error> {
    let slot = block.slot;
    let timestamp = block.block_time.as_ref();
    let mut data: Vec<TradeData> = vec![];
    if timestamp.is_none() {
        log::info!("block at slot {} has no timestamp", slot);
        return Ok(Swaps { data });
    }
    let timestamp = timestamp.unwrap().timestamp;
    // iter txs
    for trx in block.transactions_owned() {
        // get txs amounts
        let accounts: Vec<String> = trx
            .resolved_accounts()
            .iter()
            .map(|account| bs58::encode(account).into_string())
            .collect();
        if trx.transaction.is_none() {
            continue;
        }
        let transaction = trx.transaction.unwrap();

        let meta = trx.meta.unwrap();
        let pre_token_balances = meta.pre_token_balances;
        let post_token_balances = meta.post_token_balances;
        let post_balances = meta.post_balances;
        let pre_balances = meta.pre_balances;
        let message = transaction.message.unwrap();
        for (idx, inst) in message.instructions.into_iter().enumerate() {
            let inner_instructions: Vec<InnerInstructions> =
                filter_inner_instructions(&meta.inner_instructions, idx as u32);
            let program = &accounts[inst.program_id_index as usize];
            let log_message = &meta.log_messages;
            let trade_data =
                get_trade_instruction(program, &inst.data, &inst.accounts, &accounts, &post_token_balances);
            if trade_data.is_some() {
                let td = trade_data.unwrap();
                let td_name = td.name;
                let td_dapp_address = td.program;

                let mut token0 = get_mint(&td.vault_a, &post_token_balances, &accounts, &td_dapp_address);
                if token0 == "" {
                    token0 = get_mint(&td.vault_a, &pre_token_balances, &accounts, &td_dapp_address);
                }
                let mut token1 = get_mint(&td.vault_b, &pre_token_balances, &accounts, &"".to_string());
                if token1 == "" {
                    token1 = get_mint(&td.vault_b, &post_token_balances, &accounts, &"".to_string());
                }

                // exclude trading pairs that are not sol
                if is_not_soltoken(&token0, &token1) {
                    continue;
                }

                let (amount0, decimals0) = get_amt(
                    &td.amm,
                    &td.vault_a,
                    0 as u32,
                    &inner_instructions,
                    &accounts,
                    &post_token_balances,
                    &td_dapp_address,
                    &pre_balances,
                    &post_balances,
                );
                let (amount1, decimals1) = get_amt(
                    &td.amm,
                    &td.vault_b,
                    0 as u32,
                    &inner_instructions,
                    &accounts,
                    &post_token_balances,
                    &EMPTY_STRING,
                    &pre_balances,
                    &post_balances,
                );

                let (reserves0, reserves1) = get_reserves(
                    program,
                    &td.amm,
                    &td.vault_a,
                    &td.vault_b,
                    &inner_instructions,
                    log_message,
                    &accounts,
                    &token0,
                    &token1,
                    &amount0,
                    &amount1,
                    &post_token_balances,
                    &post_balances,
                );

                data.push(TradeData {
                    tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                    block_slot: slot,
                    block_time: timestamp,
                    signer: accounts.get(0).unwrap().to_string(),
                    pool_address: td.amm.clone(),
                    base_mint: token0,
                    quote_mint: token1,
                    base_amount: amount0,
                    quote_amount: amount1,
                    base_reserves: reserves0,
                    quote_reserves: reserves1,
                    base_decimals: decimals0,
                    quote_decimals: decimals1,
                    base_vault: td.vault_a,
                    quote_vault: td.vault_b,
                    is_inner_instruction: false,
                    instruction_index: idx as u32,
                    instruction_type: td_name.clone(),
                    inner_instruxtion_index: 0,
                    outer_program: td_dapp_address.clone(),
                    inner_program: "".to_string(),
                    txn_fee_lamports: meta.fee,
                });

                if td.second_swap_amm.clone().unwrap_or_default() != "" {
                    let mut token0 = get_mint(
                        td.second_swap_vault_a.as_ref().unwrap(),
                        &post_token_balances,
                        &accounts,
                        &td_dapp_address,
                    );
                    if token0 == "" {
                        token0 = get_mint(
                            td.second_swap_vault_a.as_ref().unwrap(),
                            &pre_token_balances,
                            &accounts,
                            &td_dapp_address,
                        );
                    }
                    let mut token1 = get_mint(
                        &td.second_swap_vault_b.clone().unwrap(),
                        &pre_token_balances,
                        &accounts,
                        &EMPTY_STRING,
                    );
                    if token1 == "" {
                        token1 = get_mint(
                            &td.second_swap_vault_b.clone().unwrap(),
                            &post_token_balances,
                            &accounts,
                            &EMPTY_STRING,
                        );
                    }

                    // exclude trading pairs that are not sol
                    if is_not_soltoken(&token0, &token1) {
                        continue;
                    }

                    let (amount0, decimals0) = get_amt(
                        &td.amm,
                        &td.second_swap_vault_a.clone().unwrap(),
                        0 as u32,
                        &inner_instructions,
                        &accounts,
                        &post_token_balances,
                        &td_dapp_address,
                        &pre_balances,
                        &post_balances,
                    );
                    let (amount1, decimals1) = get_amt(
                        &td.amm,
                        &td.second_swap_vault_b.clone().unwrap(),
                        0 as u32,
                        &inner_instructions,
                        &accounts,
                        &post_token_balances,
                        &EMPTY_STRING,
                        &pre_balances,
                        &post_balances,
                    );
                    let (reserves0, reserves1) = get_reserves(
                        program,
                        td.second_swap_amm.as_ref().unwrap(),
                        td.second_swap_vault_a.as_ref().unwrap(),
                        td.second_swap_vault_b.as_ref().unwrap(),
                        &inner_instructions,
                        log_message,
                        &accounts,
                        &token0,
                        &token1,
                        &amount0,
                        &amount1,
                        &post_token_balances,
                        &post_balances,
                    );

                    data.push(TradeData {
                        tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                        block_slot: slot,
                        block_time: timestamp,
                        signer: accounts.get(0).unwrap().to_string(),
                        pool_address: td.second_swap_amm.unwrap(),
                        base_mint: token0,
                        quote_mint: token1,
                        base_amount: amount0,
                        quote_amount: amount1,
                        base_reserves: reserves0,
                        quote_reserves: reserves1,
                        base_decimals: decimals0,
                        quote_decimals: decimals1,
                        base_vault: td.second_swap_vault_a.unwrap(),
                        quote_vault: td.second_swap_vault_b.unwrap(),
                        is_inner_instruction: false,
                        instruction_index: idx as u32,
                        instruction_type: td_name.clone(),
                        inner_instruxtion_index: 0,
                        outer_program: td_dapp_address,
                        inner_program: "".to_string(),
                        txn_fee_lamports: meta.fee,
                    });
                }
            }

            meta.inner_instructions
                .iter()
                .filter(|inner_instruction| inner_instruction.index == idx as u32)
                .for_each(|inner_instruction| {
                    inner_instruction
                        .instructions
                        .iter()
                        .enumerate()
                        .for_each(|(inner_idx, inner_inst)| {
                            let inner_program = &accounts[inner_inst.program_id_index as usize];
                            let inner_trade_data = get_trade_instruction(
                                inner_program,
                                &inner_inst.data.clone(),
                                &inner_inst.accounts,
                                &accounts,
                                &post_token_balances,
                            );
                            if inner_trade_data.is_some() {
                                let inner_td = inner_trade_data.unwrap();

                                let inner_td_name = inner_td.name;
                                let inner_td_dapp_address = inner_td.program;

                                let mut token0 = get_mint(
                                    &inner_td.vault_a,
                                    &pre_token_balances,
                                    &accounts,
                                    &inner_td_dapp_address,
                                );
                                if token0 == "" {
                                    token0 = get_mint(
                                        &inner_td.vault_a,
                                        &post_token_balances,
                                        &accounts,
                                        &inner_td_dapp_address,
                                    );
                                }
                                let mut token1 =
                                    get_mint(&inner_td.vault_b, &pre_token_balances, &accounts, &EMPTY_STRING);
                                if token1 == "" {
                                    token1 =
                                        get_mint(&inner_td.vault_b, &post_token_balances, &accounts, &EMPTY_STRING);
                                }

                                // exclude trading pairs that are not sol
                                if !is_not_soltoken(&token0, &token1) {
                                    let (amount0, decimals0) = get_amt(
                                        &inner_td.amm,
                                        &inner_td.vault_a,
                                        inner_idx as u32,
                                        &inner_instructions,
                                        &accounts,
                                        &post_token_balances,
                                        &inner_td_dapp_address,
                                        &pre_balances,
                                        &post_balances,
                                    );
                                    let (amount1, decimals1) = get_amt(
                                        &inner_td.amm,
                                        &inner_td.vault_b,
                                        inner_idx as u32,
                                        &inner_instructions,
                                        &accounts,
                                        &post_token_balances,
                                        &EMPTY_STRING,
                                        &pre_balances,
                                        &post_balances,
                                    );
                                    let (reserves0, reserves1) = get_reserves(
                                        inner_program,
                                        &inner_td.amm,
                                        &inner_td.vault_a,
                                        &inner_td.vault_b,
                                        &inner_instructions,
                                        log_message,
                                        &accounts,
                                        &token0,
                                        &token1,
                                        &amount0,
                                        &amount1,
                                        &post_token_balances,
                                        &post_balances,
                                    );

                                    data.push(TradeData {
                                        tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                                        block_slot: slot,
                                        block_time: timestamp,
                                        signer: accounts.get(0).unwrap().to_string(),
                                        pool_address: inner_td.amm,
                                        base_mint: token0,
                                        quote_mint: token1,
                                        base_amount: amount0,
                                        quote_amount: amount1,
                                        base_decimals: decimals0,
                                        quote_decimals: decimals1,
                                        base_reserves: reserves0,
                                        quote_reserves: reserves1,
                                        base_vault: inner_td.vault_a,
                                        quote_vault: inner_td.vault_b,
                                        is_inner_instruction: true,
                                        instruction_index: idx as u32,
                                        instruction_type: inner_td_name,
                                        inner_instruxtion_index: inner_idx as u32,
                                        outer_program: program.clone(),
                                        inner_program: inner_td_dapp_address,
                                        txn_fee_lamports: meta.fee,
                                    });
                                }
                            }
                        })
                });
        }
    }
    log::info!("{:#?}", slot);
    Ok(Swaps { data })
}

fn get_trade_instruction(
    program: &String,
    instruction_data: &Vec<u8>,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
) -> Option<TradeInstruction> {
    let input_accounts = prepare_input_accounts(account_indices, accounts);

    let mut result = None;
    match program.as_str() {
        // Raydium pool v4
        RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS => {
            result = dapps::dapp_675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8::parse_trade_instruction(
                instruction_data,
                &input_accounts,
                &post_token_balances,
                accounts,
            );
        }
        // Pump.fun
        PUMP_FUN_AMM_PROGRAM_ADDRESS => {
            result = dapps::dapp_6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P::parse_trade_instruction(
                instruction_data,
                &input_accounts,
            );
        }

        RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS => {
            result = dapps::dapp_CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK::parse_trade_instruction(
                instruction_data,
                &input_accounts,
            );
        }

        METEORA_PROGRAM_ADDRESS => {
            result = dapps::dapp_LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo::parse_trade_instruction(
                instruction_data,
                &input_accounts,
            );
        }
        METEORA_POOL_PROGRAM_ADDRESS => {
            result = dapps::dapp_Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB::parse_trade_instruction(
                instruction_data,
                &input_accounts,
            );
        }
        ORCA_PROGRAM_ADDRESS => {
            result = dapps::dapp_whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc::parse_trade_instruction(
                instruction_data,
                &input_accounts,
            );
        }
        RAYDIUM_CPMM_ADDRESS => {
            result = dapps::dapp_CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C::parse_trade_instruction(
                instruction_data,
                &input_accounts,
            );
        }
        MOONSHOT_ADDRESS => {
            result = dapps::dapp_MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG::parse_trade_instruction(
                instruction_data,
                &input_accounts,
            );
        }
        _ => {}
    }
    return result;
}

fn get_reserves(
    program: &String,
    amm: &String,
    vault_a: &String,
    vault_b: &String,
    inner_instructions: &Vec<InnerInstructions>,
    log_messages: &Vec<String>,
    accounts: &Vec<String>,
    token0: &String,
    token1: &String,
    amount0: &String,
    amount1: &String,
    post_token_balances: &Vec<TokenBalance>,
    post_balances: &Vec<u64>,
) -> (u64, u64) {
    let (mut reserves0, mut reserves1) = (0, 0);
    match program.as_str() {
        RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS => {
            (reserves0, reserves1) = parse_reserves_instruction(
                RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS.to_string(),
                &"5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1".to_string(),
                accounts,
                post_token_balances,
                post_balances,
                vault_a,
                vault_b,
                token0,
                token1,
            );
        }
        // Pump.fun
        PUMP_FUN_AMM_PROGRAM_ADDRESS => {
            (reserves0, reserves1) =
                dapps::dapp_6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P::parse_reserves_instruction(
                    inner_instructions,
                    accounts,
                    log_messages,
                    token0,
                    token1,
                );
        }
        RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS => {
            (reserves0, reserves1) = parse_reserves_instruction(
                RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS.to_string(),
                amm,
                accounts,
                post_token_balances,
                post_balances,
                vault_a,
                vault_b,
                token0,
                token1,
            );
        }
        METEORA_PROGRAM_ADDRESS => {
            (reserves0, reserves1) = parse_reserves_instruction(
                METEORA_PROGRAM_ADDRESS.to_string(),
                amm,
                accounts,
                post_token_balances,
                post_balances,
                vault_a,
                vault_b,
                token0,
                token1,
            );
        }

        ORCA_PROGRAM_ADDRESS => {
            (reserves0, reserves1) = parse_reserves_instruction(
                ORCA_PROGRAM_ADDRESS.to_string(),
                amm,
                accounts,
                post_token_balances,
                post_balances,
                vault_a,
                vault_b,
                token0,
                token1,
            );
        }

        RAYDIUM_CPMM_ADDRESS => {
            (reserves0, reserves1) = parse_reserves_instruction(
                RAYDIUM_CPMM_ADDRESS.to_string(),
                &"GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL".to_string(),
                accounts,
                post_token_balances,
                post_balances,
                vault_a,
                vault_b,
                token0,
                token1,
            );
        }

        MOONSHOT_ADDRESS => {
            (reserves0, reserves1) = parse_reserves_instruction(
                MOONSHOT_ADDRESS.to_string(),
                amm,
                accounts,
                post_token_balances,
                post_balances,
                vault_a,
                vault_b,
                token0,
                token1,
            );
        }

        METEORA_POOL_PROGRAM_ADDRESS => {
            (reserves0, reserves1) = parse_reserves_instruction(
                METEORA_POOL_PROGRAM_ADDRESS.to_string(),
                amm,
                accounts,
                post_token_balances,
                post_balances,
                vault_a,
                vault_b,
                token0,
                token1,
            );
        }

        _ => {}
    }
    return (reserves0, reserves1);
}

fn filter_inner_instructions(meta_inner_instructions: &Vec<InnerInstructions>, idx: u32) -> Vec<InnerInstructions> {
    let mut inner_instructions: Vec<InnerInstructions> = vec![];
    let mut iterator = meta_inner_instructions.iter();
    while let Some(inner_inst) = iterator.next() {
        if inner_inst.index == idx as u32 {
            inner_instructions.push(inner_inst.clone());
        }
    }
    return inner_instructions;
}
