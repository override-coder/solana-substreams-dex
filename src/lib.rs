mod pb;
mod trade_instruction;
mod dapps;
mod utils;

use substreams::log;
use substreams_solana::pb::sf::solana::r#type::v1::{Block, InnerInstructions, TokenBalance};
use pb::sf::solana::dex::trades::v1::{Output};
use crate::pb::sf::solana::dex::trades::v1::TradeData;
use crate::trade_instruction::TradeInstruction;
use crate::utils::{get_amt, get_mint};

#[substreams::handlers::map]
fn map_block(block: Block) -> Result<Output,substreams::errors::Error> {
    process_block(block)
}

fn process_block(block: Block) -> Result<Output,substreams::errors::Error> {
    let slot = block.slot;
    let parent_slot = block.parent_slot;
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
                data.push(TradeData {
                    tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                    block_slot: slot,
                    block_time: timestamp,
                    signer: accounts.get(0).unwrap().to_string(),
                    pool_address: td.amm,
                    base_mint: get_mint(&td.vault_a, &post_token_balances, &accounts),
                    quote_mint: get_mint(&td.vault_b, &pre_token_balances, &accounts),
                    base_amount: get_amt(
                        &td.vault_a,
                        0 as u32,
                        &inner_instructions,
                        &accounts,
                        &post_token_balances,
                    ),
                    quote_amount: get_amt(
                        &td.vault_b,
                        0 as u32,
                        &inner_instructions,
                        &accounts,
                        &post_token_balances,
                    ),
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

                                data.push(TradeData {
                                    tx_id: bs58::encode(&transaction.signatures[0])
                                        .into_string(),
                                    block_slot: slot,
                                    block_time: timestamp,
                                    signer: accounts.get(0).unwrap().to_string(),
                                    pool_address: inner_td.amm,
                                    base_mint: get_mint(
                                        &inner_td.vault_a,
                                        &pre_token_balances,
                                        &accounts,
                                    ),
                                    quote_mint: get_mint(
                                        &inner_td.vault_b,
                                        &pre_token_balances,
                                        &accounts,
                                    ),
                                    base_amount: get_amt(
                                        &inner_td.vault_a,
                                        inner_idx as u32,
                                        &inner_instructions,
                                        &accounts,
                                        &post_token_balances,
                                    ),
                                    quote_amount: get_amt(
                                        &inner_td.vault_b,
                                        inner_idx as u32,
                                        &inner_instructions,
                                        &accounts,
                                        &post_token_balances,
                                    ),
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