use crate::constants::{
    METEORA_POOL_PROGRAM_ADDRESS, METEORA_PROGRAM_ADDRESS, MOONSHOT_ADDRESS, ORCA_PROGRAM_ADDRESS,
    PUMP_FUN_AMM_PROGRAM_ADDRESS, RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS, RAYDIUM_CPMM_ADDRESS,
    RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS,
};
use crate::pb::sf::solana::dex::trades::v1::{Pool, Pools};
use crate::pool_creations::dapps;
use crate::pool_creations::pool_instruction::CreatePoolInstruction;
use crate::utils::{is_not_soltoken, prepare_input_accounts, WSOL_ADDRESS};
use substreams_solana::pb::sf::solana::r#type::v1::{Block, InnerInstructions};

#[substreams::handlers::map]
fn map_pools_created(block: Block) -> Result<Pools, substreams::errors::Error> {
    process_block(block)
}

fn process_block(block: Block) -> Result<Pools, substreams::errors::Error> {
    let slot = block.slot;
    let timestamp = block.block_time.as_ref();
    let mut data: Vec<Pool> = vec![];

    if timestamp.is_none() {
        log::info!("block at slot {} has no timestamp", slot);
        return Ok(Pools { pools: data });
    }
    let timestamp = timestamp.unwrap().timestamp;
    for trx in block.transactions_owned() {
        let accounts: Vec<String> = trx
            .resolved_accounts()
            .iter()
            .map(|account| bs58::encode(account).into_string())
            .collect();

        if let Some(transaction) = trx.transaction {
            let meta = trx.meta.unwrap();

            let msg = transaction.message.unwrap();

            for (idx, inst) in msg.instructions.into_iter().enumerate() {
                let program = &accounts[inst.program_id_index as usize];
                let inner_instructions = filter_inner_instructions(&meta.inner_instructions, idx as u32);

                let pool_instruction = get_pool_instruction(program, &inst.data, &inst.accounts, &accounts);
                if pool_instruction.is_some() {
                    let p = pool_instruction.unwrap();
                    if is_not_soltoken(&p.pc_mint, &p.coin_mint) {
                        continue;
                    }
                    data.push(Pool {
                        program: p.program,
                        address: p.amm,
                        created_at_timestamp: timestamp as u64,
                        created_at_block_number: slot,
                        coin_mint: p.coin_mint,
                        pc_mint: p.pc_mint,
                        is_pump_fun: p.is_pump_fun,
                        is_moonshot: p.is_moonshot,
                        tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                    });
                }

                inner_instructions.iter().for_each(|inner_instruction| {
                    inner_instruction
                        .instructions
                        .iter()
                        .enumerate()
                        .for_each(|(inner_idx, inner_inst)| {
                            let inner_program = &accounts[inner_inst.program_id_index as usize];
                            let pool_inner_instruction =
                                get_pool_instruction(inner_program, &inner_inst.data, &inner_inst.accounts, &accounts);

                            if pool_inner_instruction.is_some() {
                                let inner_pool = pool_inner_instruction.unwrap();
                                if !is_not_soltoken(&inner_pool.pc_mint, &inner_pool.coin_mint) {
                                    data.push(Pool {
                                        program: inner_pool.program,
                                        address: inner_pool.amm,
                                        created_at_timestamp: timestamp as u64,
                                        created_at_block_number: slot,
                                        coin_mint: inner_pool.coin_mint,
                                        pc_mint: inner_pool.pc_mint,
                                        is_pump_fun: inner_pool.is_pump_fun,
                                        is_moonshot: inner_pool.is_moonshot,
                                        tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                                    });
                                }
                            }
                        })
                });
            }
        }
    }
    log::info!("{:#?}", slot);
    Ok(Pools { pools: data })
}

fn get_pool_instruction(
    program: &String,
    instruction_data: &Vec<u8>,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
) -> Option<CreatePoolInstruction> {
    let input_accounts = prepare_input_accounts(account_indices, accounts);
    let mut result = None;
    match program.as_str() {
        RAYDIUM_POOL_V4_AMM_PROGRAM_ADDRESS => {
            result = dapps::dapp_675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8::parse_trade_instruction(
                instruction_data,
                &input_accounts,
            );
        }
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
                &instruction_data,
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
    result
}

fn filter_inner_instructions(meta_inner_instructions: &Vec<InnerInstructions>, idx: u32) -> Vec<InnerInstructions> {
    let mut inner_instructions: Vec<InnerInstructions> = vec![];
    let mut iterator = meta_inner_instructions.iter();
    while let Some(inner_inst) = iterator.next() {
        if inner_inst.index == idx {
            inner_instructions.push(inner_inst.clone());
        }
    }
    return inner_instructions;
}
