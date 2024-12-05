use crate::constants::JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS;
use crate::jupiter_aggregator::jupiter_aggregator_instruction::{
    parse_inner_instruction, parse_instruction, InstructionSwapEvent, RouterInstruction,
};
use crate::pb::sf::solana::dex::jupiter_aggregator::v1::{JupiterSwaps, JupiterTrade};
use crate::utils::{get_decimals, is_not_soltoken, USDC_ADDRESS, USDT_ADDRESS, WSOL_ADDRESS};
use substreams_solana::pb::sf::solana::r#type::v1::{
    Block, CompiledInstruction, TokenBalance, TransactionStatusMeta,
};

#[substreams::handlers::map]
fn map_jupiter_aggregator(block: Block) -> Result<JupiterSwaps, substreams::errors::Error> {
    let slot = block.slot;
    let timestamp = block.block_time.as_ref().unwrap().timestamp;
    let mut data: Vec<JupiterTrade> = vec![];
    for trx in block.transactions_owned() {
        let accounts: Vec<String> = trx
            .resolved_accounts()
            .iter()
            .map(|account| bs58::encode(account).into_string())
            .collect();
        if let Some(transaction) = trx.transaction {
            let meta = trx.meta.unwrap();
            let msg = transaction.message.unwrap();
            let pre_token_balances = &meta.pre_token_balances;
            let post_token_balances = &meta.post_token_balances;
            for (idx, inst) in msg.instructions.into_iter().enumerate() {
                if let Some(out) = extract_instruction_events(&accounts, &inst, idx, &meta) {
                    let source_mint = out.source_mint;
                    let destination_mint = out.destination_mint;
                    let (mut in_decimals, mut quoted_decimals) =
                        get_decimals(&source_mint, &destination_mint, &post_token_balances);
                    if in_decimals == 0 {
                        (in_decimals, _) =
                            get_decimals(&source_mint, &destination_mint, &pre_token_balances);
                    }
                    if filter_data(&source_mint,&destination_mint)
                    {
                        data.push(JupiterTrade {
                            dapp: JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS.to_string(),
                            block_time: timestamp,
                            block_slot: slot,
                            tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                            signer: out.signer,
                            source_token_account: out.source_token_account,
                            destination_token_account: out.destination_token_account,
                            source_mint,
                            destination_mint,
                            in_amount: out.in_amount,
                            quoted_out_amount: out.quoted_out_amount,
                            in_decimals,
                            quoted_decimals,
                            instruction_type: out.instruction_types,
                        });
                    }
                }
            }
        }
    }
    Ok(JupiterSwaps { data })
}

fn extract_instruction_events(
    accounts: &Vec<String>,
    inst: &CompiledInstruction,
    idx: usize,
    meta: &TransactionStatusMeta,
) -> Option<RouterInstruction> {
    let program = &accounts[inst.program_id_index as usize];
    if program != JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS {
        return None;
    }
    let mut instruction_data =
        parse_instruction(program, inst.data.clone(), &inst.accounts, accounts)
            .unwrap_or(RouterInstruction::default());
    let mut events: Vec<InstructionSwapEvent> = vec![];
    meta.inner_instructions
        .iter()
        .filter(|inner_instruction| inner_instruction.index == idx as u32)
        .for_each(|inner_instruction| {
            inner_instruction.instructions.iter().enumerate().for_each(
                |(inner_idx, inner_inst)| {
                    let inner_program = &accounts[inner_inst.program_id_index as usize];
                    if let Some(event) = parse_inner_instruction(
                        inner_program,
                        inner_inst.data.clone(),
                        &inner_inst.accounts,
                        accounts,
                    ) {
                        events.push(event);
                    }
                },
            );
        });
    if let Some(result) = select_swap_events(&events) {
        if let Some((input_mint, input_amount, output_mint, output_amount)) =
            extract_swap_event_data(result)
        {
            instruction_data.source_mint = input_mint;
            instruction_data.destination_mint = output_mint;
            instruction_data.in_amount = input_amount.to_string();
            instruction_data.quoted_out_amount = output_amount.to_string();
            return Some(instruction_data);
        }
    }
    return None;
}

// Helper function to select the first and last events, or a single event
fn select_swap_events(events: &[InstructionSwapEvent]) -> Option<Vec<InstructionSwapEvent>> {
    if events.is_empty() {
        return None;
    }
    if events.len() == 1 {
        Some(vec![events[0].clone()])
    } else {
        Some(vec![events[0].clone(), events[events.len() - 1].clone()])
    }
}

fn extract_swap_event_data(
    events: Vec<InstructionSwapEvent>,
) -> Option<(String, u64, String, u64)> {
    if events.is_empty() {
        return None;
    }
    if events.len() == 1 {
        let event = &events[0];
        Some((
            bs58::encode(&event.input_mint).into_string(),
            event.input_amount,
            bs58::encode(&event.output_mint).into_string(),
            event.output_amount,
        ))
    } else {
        let first_event = &events[0];
        let last_event = events.last().unwrap();

        Some((
            bs58::encode(&first_event.input_mint).into_string(),
            first_event.input_amount,
            bs58::encode(&last_event.output_mint).into_string(),
            last_event.output_amount,
        ))
    }
}

fn filter_data(source_mint: &String, destination_mint: &String) -> bool {
    is_target_pair(source_mint, destination_mint, WSOL_ADDRESS)
        || is_target_pair(source_mint, destination_mint, USDT_ADDRESS)
        || is_target_pair(source_mint, destination_mint, USDC_ADDRESS)
}

fn is_target_pair(source_mint: &str, destination_mint: &str, target_address: &str) -> bool {
    (source_mint == target_address && destination_mint != target_address)
        || (source_mint != target_address && destination_mint == target_address)
}