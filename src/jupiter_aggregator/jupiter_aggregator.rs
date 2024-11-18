use substreams_solana::pb::sf::solana::r#type::v1::{Block, TokenBalance};
use crate::constants::JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS;
use crate::jupiter_aggregator::jupiter_aggregator_instruction::{parse_instruction};
use crate::pb::sf::solana::dex::jupiter_aggregator::v1::{JupiterSwaps, JupiterTrade};
use crate::utils::{get_decimals, is_not_soltoken};

#[substreams::handlers::map]
fn map_jupiter_aggregator(block: Block) -> Result<JupiterSwaps, substreams::errors::Error> {
    let slot = block.slot;
    let timestamp = block.block_time.as_ref().unwrap().timestamp;
    let mut data: Vec<JupiterTrade> = vec![];
    for trx in block.transactions_owned() {
        let accounts: Vec<String> = trx.resolved_accounts().iter().map(|account| bs58::encode(account).into_string())
            .collect();
        if let Some(transaction) = trx.transaction {
            let meta = trx.meta.unwrap();
            let msg = transaction.message.unwrap();

            let post_token_balances = meta.post_token_balances;

            for (idx, inst) in msg.instructions.into_iter().enumerate() {
                let program = &accounts[inst.program_id_index as usize];
                // if program == JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS && bs58::encode(&transaction.signatures[0]).into_string() == "4fBdpn6b5DgwmjKHaQLqb3PWExnyturBUmQ7yuw1NUZepvcgHywqkYZTJjrj4q6majJScCzdoi6nxYs6yHJz5vk1"{
                //     panic!("{:?}",inst.data)
                // }
                if let Some(out) = parse_instruction(program, inst.data, &inst.accounts, &accounts) {
                    if is_not_soltoken(&out.source_mint,&out.destination_mint){
                            continue
                    }

                    let (in_decimals,quoted_decimals) = get_decimals(&out.source_mint,&out.destination_mint, &post_token_balances);

                    data.push(JupiterTrade {
                        dapp: JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS.to_string(),
                        block_time: timestamp,
                        block_slot: slot,
                        tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                        signer: out.signer,
                        source_token_account: out.source_token_account,
                        destination_token_account: out.destination_token_account,
                        source_mint: out.source_mint,
                        destination_mint: out.destination_mint,
                        in_amount: out.in_amount,
                        quoted_out_amount: out.quoted_out_amount,
                        in_decimals,
                        quoted_decimals,
                        instruction_type: out.instruction_types,
                    });
                }
                meta.inner_instructions
                    .iter()
                    .filter(|inner_instruction| inner_instruction.index == idx as u32)
                    .for_each(|inner_instruction| {
                        inner_instruction.instructions.iter().enumerate().for_each(
                            |(inner_idx, inner_inst)| {
                                let inner_program = &accounts[inner_inst.program_id_index as usize];
                                if let Some(out) = parse_instruction(inner_program, inner_inst.data.clone(), &inst.accounts, &accounts) {
                                    if !is_not_soltoken(&out.source_mint,&out.destination_mint){

                                        let (in_decimals,quoted_decimals) = get_decimals(&out.source_mint,&out.destination_mint, &post_token_balances);

                                        data.push(JupiterTrade {
                                            dapp: JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS.to_string(),
                                            block_time: timestamp,
                                            block_slot: slot,
                                            tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                                            signer: out.signer,
                                            source_token_account: out.source_token_account,
                                            destination_token_account: out.destination_token_account,
                                            source_mint: out.source_mint,
                                            destination_mint: out.destination_mint,
                                            in_amount: out.in_amount,
                                            quoted_out_amount: out.quoted_out_amount,
                                            in_decimals,
                                            quoted_decimals,
                                            instruction_type: out.instruction_types,
                                        });
                                    }
                                }
                            },
                        )
                    });
            }
        }
    }
    Ok(JupiterSwaps { data })
}