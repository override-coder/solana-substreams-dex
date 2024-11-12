mod pb;
mod trade_instruction;
mod dapps;
mod utils;
mod db;

use substreams::errors::Error;
use substreams::log;
use substreams::prelude::*;
use substreams::store::{ StoreGetFloat64, StoreSetFloat64};
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_solana::pb::sf::solana::r#type::v1::{Block, InnerInstructions, TokenBalance};
use pb::sf::solana::dex::trades::v1::{Output,Pools};
use crate::pb::sf::solana::dex::trades::v1::{Pool, TradeData};
use crate::trade_instruction::{CreatePoolInstruction, TradeInstruction};
use crate::utils::{WSOL_ADDRESS, get_amt, get_mint, get_wsol_price, is_not_soltoken, find_sol_stable_coin_trade};

#[substreams::handlers::map]
pub fn map_block(block: Block) -> Result<Output,Error> {
    process_block(block)
}

fn process_block(block: Block) -> Result<Output,Error> {
    let slot = block.slot;
    let timestamp = block.block_time.as_ref();
    let mut data: Vec<TradeData> = vec![];
    if timestamp.is_none() {
        log::info!("block at slot {} has no timestamp", slot);
        return Ok(Output { data });
    }
    let timestamp = timestamp.unwrap().timestamp;
    // iter txs
    for trx in block.transactions_owned() {
        // get txs amounts
        let accounts:Vec<String> = trx.resolved_accounts().iter().map(|account| bs58::encode(account).into_string())
            .collect();
        if trx.transaction.is_none() {
            continue
        }
        let transaction = trx.transaction.unwrap();
        let meta = trx.meta.unwrap();
        let pre_token_balances = meta.pre_token_balances;
        let post_token_balances = meta.post_token_balances;

        let message = transaction.message.unwrap();

        for (idx,inst) in message.instructions.into_iter().enumerate() {
            let inner_instructions: Vec<InnerInstructions> = filter_inner_instructions(&meta.inner_instructions, idx as u32);
            let program = &accounts[inst.program_id_index as usize];
            let trade_data = get_trade_instruction(
                program,
                inst.data,
                &inst.accounts,
                &accounts,
                &post_token_balances,
            );
            if  trade_data.is_some(){
                let td = trade_data.unwrap();
                let td_name = td.name;
                let td_dapp_address = td.program;

                let token0 = get_mint(&td.vault_a, &post_token_balances, &accounts);
                let mut token1 = get_mint(&td.vault_b, &pre_token_balances, &accounts);

                if (token1.is_empty() || token1 == "") && td_dapp_address.clone() == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" {
                    token1 = WSOL_ADDRESS.to_string()
                }

                // exclude trading pairs that are not sol
                if is_not_soltoken(&token0,&token1) {
                    continue
                }

                let (amount0,decimals0) = get_amt(&td.vault_a, 0 as u32, &inner_instructions, &accounts, &post_token_balances);
                let (amount1,decimals1) = get_amt(&td.vault_b, 0 as u32, &inner_instructions, &accounts, &post_token_balances);

                data.push(TradeData {
                    tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                    block_slot: slot,
                    block_time: timestamp,
                    signer: accounts.get(0).unwrap().to_string(),
                    pool_address: td.amm,
                    base_mint: token0,
                    quote_mint: token1,
                    base_amount: amount0,
                    quote_amount: amount1,
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
            }

            meta.inner_instructions
                .iter()
                .filter(|inner_instruction| inner_instruction.index == idx as u32)
                .for_each(|inner_instruction| {
                    inner_instruction.instructions.iter().enumerate().for_each(
                        |(inner_idx, inner_inst)| {
                            let inner_program =
                                &accounts[inner_inst.program_id_index as usize];
                            let inner_trade_data = get_trade_instruction(
                                inner_program,
                                inner_inst.data.clone(),
                                &inner_inst.accounts,
                                &accounts,
                                &post_token_balances,
                            );

                            if inner_trade_data.is_some() {
                                let inner_td = inner_trade_data.unwrap();

                                let inner_td_name = inner_td.name;
                                let inner_td_dapp_address = inner_td.program;

                                let token0 = get_mint(&inner_td.vault_a, &pre_token_balances, &accounts);
                                let mut token1 = get_mint(&inner_td.vault_b, &pre_token_balances, &accounts);

                                if (token1.is_empty() || token1 == "") && inner_td_dapp_address.to_string() == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" {
                                    token1 = WSOL_ADDRESS.to_string();
                                }

                                // exclude trading pairs that are not sol
                                if !is_not_soltoken(&token0, &token1) {
                                    let (amount0,decimals0) = get_amt(&inner_td.vault_a, 0 as u32, &inner_instructions, &accounts, &post_token_balances);
                                    let (amount1,decimals1) = get_amt(&inner_td.vault_b, 0 as u32, &inner_instructions, &accounts, &post_token_balances);

                                    data.push(TradeData {
                                        tx_id: bs58::encode(&transaction.signatures[0])
                                            .into_string(),
                                        block_slot: slot,
                                        block_time: timestamp,
                                        signer: accounts.get(0).unwrap().to_string(),
                                        pool_address: inner_td.amm,
                                        base_mint: token0,
                                        quote_mint: token1,
                                        base_amount:amount0,
                                        quote_amount: amount1,
                                        base_decimals:decimals0,
                                        quote_decimals: decimals1,
                                        base_vault: inner_td.vault_a,
                                        quote_vault: inner_td.vault_b,
                                        is_inner_instruction: true,
                                        instruction_index: idx as u32,
                                        instruction_type: inner_td_name.clone(),
                                        inner_instruxtion_index: inner_idx as u32,
                                        outer_program: program.to_string(),
                                        inner_program: inner_td_dapp_address.clone(),
                                        txn_fee_lamports: meta.fee,
                                    });
                                }
                            }
                        },
                    )
                });
        }
    }
    log::info!("{:#?}", slot);
    Ok(Output { data })
}

fn get_trade_instruction(
    program: &String,
    instruction_data: Vec<u8>,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
    post_token_balances: &Vec<TokenBalance>,
) -> Option<TradeInstruction> {
    let input_accounts = prepare_input_accounts(account_indices, accounts);

    let mut result = None;
    match program.as_str() {
        // Raydium pool v4
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => {
            result =
                dapps::dapp_675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8::parse_trade_instruction(
                    instruction_data,
                    input_accounts,
                    &post_token_balances,
                    accounts,
                );
        }
        // Pump.fun
        "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" => {
            result =
                dapps::dapp_6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P::parse_trade_instruction(
                    instruction_data,
                    input_accounts,
                );
        }
        _ => {}
    }
    return result;
}

fn get_pool_instruction(
    program: &String,
    instruction_data: Vec<u8>,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
) -> Option<CreatePoolInstruction> {
    let input_accounts = prepare_input_accounts(account_indices, accounts);
    let mut result = None;
    match program.as_str() {
        // Raydium pool v4
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8" => {
            result =
                dapps::dapp_675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8::parse_pool_instruction(
                    instruction_data,
                    input_accounts,
                );
        }
        // Pump.fun
        "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" => {
            result =
                dapps::dapp_6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P::parse_pool_instruction(
                    instruction_data,
                    input_accounts,
                );
        }
        _ => {}
    }
    return result;
}

fn prepare_input_accounts(account_indices: &Vec<u8>, accounts: &Vec<String>) -> Vec<String> {
    let mut instruction_accounts: Vec<String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        instruction_accounts.push(accounts.as_slice()[el as usize].to_string());
    }
    return instruction_accounts;
}

fn filter_inner_instructions(
    meta_inner_instructions: &Vec<InnerInstructions>,
    idx: u32,
) -> Vec<InnerInstructions> {
    let mut inner_instructions: Vec<InnerInstructions> = vec![];
    let mut iterator = meta_inner_instructions.iter();
    while let Some(inner_inst) = iterator.next() {
        if inner_inst.index == idx as u32 {
            inner_instructions.push(inner_inst.clone());
        }
    }
    return inner_instructions;
}

#[substreams::handlers::map]
pub fn map_pools_created(block: Block) -> Result<Pools,Error> {
    let slot = block.slot;
    let timestamp = block.block_time.as_ref();
    let mut data: Vec<Pool> = vec![];
    if timestamp.is_none() {
        log::info!("block at slot {} has no timestamp", slot);
        return Ok(Pools { pools: data });
    }
    let timestamp = timestamp.unwrap().timestamp;
    // iter txs
    for trx in block.transactions_owned() {
        // get txs amounts
        let accounts:Vec<String> = trx.resolved_accounts().iter().map(|account| bs58::encode(account).into_string())
            .collect();
        if trx.transaction.is_none() {
            continue
        }
        let transaction = trx.transaction.unwrap();
        let message = transaction.message.unwrap();
        for (_,inst) in message.instructions.into_iter().enumerate() {
            let program = &accounts[inst.program_id_index as usize];
            let pool_instruction = get_pool_instruction(
                program,
                inst.data,
                &inst.accounts,
                &accounts,
            );
            if  pool_instruction.is_some(){
                let p = pool_instruction.unwrap();
                let mut coin_mint = p.coin_mint;
                if  p.program == "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P" && coin_mint == ""  {
                   coin_mint = WSOL_ADDRESS.to_string()
               }
                data.push(Pool{
                    program: p.program,
                    address:  p.amm,
                    created_at_timestamp: timestamp as u64,
                    created_at_block_number: slot,
                    coin_mint,
                    pc_mint: p.pc_mint,
                    coin_token_account: "".to_string(),
                    pc_amount: "".to_string(),
                    is_pump_fun: p.is_pump_fun,
                    tx_id: bs58::encode(&transaction.signatures[0])
                        .into_string(),
                });
            }

        }
    }
    Ok(Pools { pools: data })
}

#[substreams::handlers::store]
pub fn store_sol_prices(out: Output,store: StoreSetFloat64)  {
    if let Some(trade) = find_sol_stable_coin_trade(&out.data) {
        if let Some(price) = get_wsol_price(&trade.base_mint,&trade.quote_mint,trade.base_amount,trade.quote_amount) {
            store.set(0, WSOL_ADDRESS, &price);
        }
    }
}

#[substreams::handlers::map]
pub fn slink_database_out(out: Output,store: StoreGetFloat64) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();
    db::created_trande_database_changes(&mut tables, &out, &store);
    return Ok(tables.to_database_changes())
}
