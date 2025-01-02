use substreams_solana::pb::sf::solana::r#type::v1::Block;
use crate::constants;
use crate::pb::sf::solana::dex::meta::v1::{Arg, TokenMetadataMeta, TokenMetas};
use crate::spl_token::spl_token_meta_instruction::{prepare_arg, prepare_input_accounts, INSTRUCTION_TYPE_CREATE, INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT, INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V2, INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V3};
use crate::utils::convert_to_date;

#[substreams::handlers::map]
fn map_token_metadata(block: Block) -> Result<TokenMetas, substreams::errors::Error> {
    let slot = block.slot;
    let timestamp = block.block_time.as_ref();
    let mut data: Vec<TokenMetadataMeta> = vec![];
    if timestamp.is_none() {
        log::info!("block at slot {} has no timestamp", slot);
        return Ok(TokenMetas { data });
    }
    let timestamp = timestamp.unwrap().timestamp;
    for trx in block.transactions_owned() {
        let accounts: Vec<String> = trx.resolved_accounts().iter()
            .map(|account| bs58::encode(account).into_string())
            .collect();

        if let Some(transaction) = trx.transaction {
            let msg = transaction.message.unwrap();
            let meta = trx.meta.unwrap();
            let tx_id = bs58::encode(&transaction.signatures[0]).into_string();

            for (idx, inst) in msg.instructions.into_iter().enumerate() {
                let program = &accounts[inst.program_id_index as usize];
                if let Some(parsed_arg_data) = get_arg(program, inst.data, tx_id.clone()) {
                    process_token_metadata(
                        &mut data,
                        parsed_arg_data,
                        &inst.accounts,
                        &accounts,
                        &tx_id,
                        slot,
                        timestamp,
                        idx as u32,
                        false,
                        0,
                    );
                }

                meta.inner_instructions
                    .iter()
                    .filter(|inner_instruction| inner_instruction.index == idx as u32)
                    .for_each(|inner_instruction| {
                        inner_instruction.instructions.iter().enumerate().for_each(|(inner_idx, inner_inst)| {
                            let program = &accounts[inner_inst.program_id_index as usize];
                            if let Some(parsed_arg_data) = get_arg(program, inner_inst.data.clone(), tx_id.clone()) {
                                process_token_metadata(
                                    &mut data,
                                    parsed_arg_data,
                                    &inner_inst.accounts,
                                    &accounts,
                                    &tx_id,
                                    slot,
                                    timestamp,
                                    idx as u32,
                                    true,
                                    inner_idx as u32,
                                );
                            }
                        });
                    });
            }
        }
    }

    Ok(TokenMetas { data })
}

fn process_token_metadata(
    data: &mut Vec<TokenMetadataMeta>,
    args: Arg,
    instruction_accounts: &Vec<u8>,
    accounts: &Vec<String>,
    tx_id: &str,
    slot: u64,
    timestamp: i64,
    instruction_index: u32,
    is_inner_instruction: bool,
    inner_instruction_index: u32,
) {
    let mut token_metadata_meta: TokenMetadataMeta = TokenMetadataMeta::default();
    if token_metadata_meta.args.is_none() {
        token_metadata_meta.args = Some(args);
    }

    if let Some(args) = &token_metadata_meta.args {
        token_metadata_meta.instruction_type = args.instruction_type.clone();
        token_metadata_meta.input_accounts = prepare_input_accounts(
            args.instruction_type.clone(),
            instruction_accounts,
            accounts,
        );
    }

    token_metadata_meta.block_date = convert_to_date(timestamp);
    token_metadata_meta.block_time = timestamp;
    token_metadata_meta.block_slot = slot;
    token_metadata_meta.tx_id = tx_id.to_string();
    token_metadata_meta.dapp = constants::TOKEN_METADATA_PROGRAM_ADDRESS.to_string();
    token_metadata_meta.instruction_index = instruction_index;
    token_metadata_meta.is_inner_instruction = is_inner_instruction;
    token_metadata_meta.inner_instruction_index = inner_instruction_index;

    if !filter_metadata_is_none(&token_metadata_meta) {
        data.push(token_metadata_meta);
    }
}

fn get_arg(program: &str, instruction_data: Vec<u8>, tx_id: String) -> Option<Arg> {
    if program != constants::TOKEN_METADATA_PROGRAM_ADDRESS {
        return None;
    }
    Some(prepare_arg(instruction_data, tx_id))
}

fn filter_metadata_is_none(mut obj: &TokenMetadataMeta) -> bool {
   return  obj.instruction_type != INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT && obj.instruction_type != INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V2 && obj.instruction_type != INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V3
    && obj.instruction_type != INSTRUCTION_TYPE_CREATE
}