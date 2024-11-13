use substreams::log;
use substreams_solana::pb::sf::solana::r#type::v1::Block;
use crate::constants;
use crate::pb::sf::solana::dex::meta::v1::{Arg, TokenMetadataMeta, TokenMetas};
use crate::prepare_arg::prepare_arg;
use crate::prepare_input_accounts::prepare_input_accounts;
use crate::utils::convert_to_date;

#[substreams::handlers::map]
fn map_token_metadata(block: Block) -> Result<TokenMetas, substreams::errors::Error> {
    let slot = block.slot;
    let parent_slot = block.parent_slot;
    let timestamp = block.block_time.as_ref().unwrap().timestamp;

    let mut data: Vec<TokenMetadataMeta> = vec![];

    for trx in block.transactions_owned() {
        let accounts:Vec<String> = trx.resolved_accounts().iter().map(|account| bs58::encode(account).into_string())
            .collect();
        if let Some(transaction) = trx.transaction {
            let msg = transaction.message.unwrap();
            let meta = trx.meta.unwrap();

            for (idx, inst) in msg.instructions.into_iter().enumerate() {
                let program = &accounts[inst.program_id_index as usize];
                let tx_id = bs58::encode(&transaction.signatures[0]).into_string();
                let parsed_arg_data = get_arg(program, inst.data, tx_id.clone());
                if parsed_arg_data.is_some() {
                    let mut tokenMetadataMeta: TokenMetadataMeta = TokenMetadataMeta::default();
                    if tokenMetadataMeta.args.is_none() {
                        tokenMetadataMeta.args = parsed_arg_data
                    }
                    if let Some(args) = &tokenMetadataMeta.args{
                        tokenMetadataMeta.instruction_type =
                            args.instruction_type.clone();
                        tokenMetadataMeta.input_accounts = prepare_input_accounts(
                            args.instruction_type.clone(),
                            &inst.accounts,
                            &accounts,
                        );

                    }
                    tokenMetadataMeta.block_date = convert_to_date(timestamp);
                    tokenMetadataMeta.block_time = timestamp;
                    tokenMetadataMeta.block_slot = slot;
                    tokenMetadataMeta.tx_id = tx_id.clone();
                    tokenMetadataMeta.dapp = constants::TOKEN_METADATA_PROGRAM_ADDRESS.to_string();
                    tokenMetadataMeta.instruction_index = idx as u32;
                    tokenMetadataMeta.is_inner_instruction = false;
                    tokenMetadataMeta.inner_instruction_index = 0;
                    data.push(tokenMetadataMeta);
                }

                meta.inner_instructions
                    .iter()
                    .filter(|inner_instruction| inner_instruction.index == idx as u32)
                    .for_each(|inner_instruction| {
                        inner_instruction.instructions.iter().enumerate().for_each(
                            |(inner_idx, inner_inst)| {
                                let program = &accounts[inner_inst.program_id_index as usize];
                                let parsed_arg_data =
                                    get_arg(program, inner_inst.data.clone(), tx_id.clone());
                                if parsed_arg_data.is_some() {
                                    let mut tokenMetadataMeta: TokenMetadataMeta =
                                        TokenMetadataMeta::default();
                                    if tokenMetadataMeta.args.is_none() {
                                        tokenMetadataMeta.args = parsed_arg_data
                                    }
                                    if let Some(args) = &tokenMetadataMeta.args{
                                        tokenMetadataMeta.instruction_type =
                                            args.instruction_type.clone();
                                        tokenMetadataMeta.input_accounts = prepare_input_accounts(
                                            args.instruction_type.clone(),
                                            &inst.accounts,
                                            &accounts,
                                        );

                                    }
                                    tokenMetadataMeta.block_date = convert_to_date(timestamp);
                                    tokenMetadataMeta.block_time = timestamp;
                                    tokenMetadataMeta.block_slot = slot;
                                    tokenMetadataMeta.tx_id = tx_id.clone();
                                    tokenMetadataMeta.dapp =
                                        constants::TOKEN_METADATA_PROGRAM_ADDRESS.to_string();
                                    tokenMetadataMeta.instruction_index = idx as u32;
                                    tokenMetadataMeta.is_inner_instruction = true;
                                    tokenMetadataMeta.inner_instruction_index = inner_idx as u32;
                                    data.push(tokenMetadataMeta);
                                }
                            },
                        );
                    });
            }
        }
    }

    log::info!("{:#?}", slot);
    Ok(TokenMetas { data })
}

fn get_arg(program: &String, instruction_data: Vec<u8>, tx_id: String) -> Option<Arg> {
    let mut result = None;

    if program
        .to_string()
        .ne(constants::TOKEN_METADATA_PROGRAM_ADDRESS)
    {
        return result;
    } else {
        result = Some(prepare_arg(instruction_data, tx_id));
        return result;
    }
}
