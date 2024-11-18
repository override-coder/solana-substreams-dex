use substreams_solana::pb::sf::solana::r#type::v1::{Block, TokenBalance};
use crate::constants;
use crate::constants::RAYDIUM_AUTHORITY_V4;
use crate::pb::sf::solana::dex::spl::v1::{Accounts, Arg, SplTokenMeta, SplTokens};
use crate::spl_token::spl_token_instruction::{
    parse_instruction,
    Instruction,
    INSTRUCTION_TYPE_TRANSFER,
    INSTRUCTION_TYPE_APPROVE,
    INSTRUCTION_TYPE_TRANSFER_CHECKED,
    INSTRUCTION_TYPE_APPROVE_CHECKED,
    INSTRUCTION_TYPE_INITIALIZE_MINT,
    INSTRUCTION_TYPE_INITIALIZE_MINT2,
    INSTRUCTION_TYPE_MINT_TO,
    INSTRUCTION_TYPE_MINT_TO_CHECKED,
    INSTRUCTION_TYPE_UNKNOWN,

};
use crate::utils::{convert_to_date, get_b58_string, prepare_input_accounts};

#[substreams::handlers::map]
fn map_spl_token(block: Block) -> Result<SplTokens, substreams::errors::Error> {
    let slot = block.slot;
    let timestamp = block.block_time.as_ref().unwrap().timestamp;
    let mut data: Vec<SplTokenMeta> = vec![];
    for trx in block.transactions_owned() {
        let accounts:Vec<String> = trx.resolved_accounts().iter().map(|account| bs58::encode(account).into_string())
            .collect();
        if let Some(transaction) = trx.transaction {
            let meta = trx.meta.unwrap();
            let msg = transaction.message.unwrap();
            let pre_token_balances = meta.pre_token_balances;
            for (idx, inst) in msg.instructions.into_iter().enumerate() {
                let program = &accounts[inst.program_id_index as usize];
                if program == constants::TOKEN_PROGRAM_ADDRESS {
                    let outer_arg = get_outer_arg(inst.data, &inst.accounts, &accounts);
                    let obj: SplTokenMeta = SplTokenMeta {
                        block_date: convert_to_date(timestamp),
                        block_time: timestamp,
                        tx_id: bs58::encode(&transaction.signatures[0]).into_string(),
                        dapp: constants::TOKEN_PROGRAM_ADDRESS.to_string(),
                        block_slot: slot,
                        instruction_index: idx as u32,
                        is_inner_instruction: false,
                        inner_instruction_index: 0,
                        instruction_type: outer_arg.instruction_type,
                        input_accounts: outer_arg.input_accounts,
                        outer_program: program.to_string(),
                        args: outer_arg.arg,
                    };
                    if filter_token(&obj) {
                        continue
                    }
                    if bs58::encode(&transaction.signatures[0]).into_string() =="5n7SCH9uM4Ftr63xYnHqSj33H5aZTvQZxYnyvPPXiQ4vRcYLhAHs4X4cLHJzRjCg5PAuUPykgMEJGuozwtVh4JG1"{
                        data.push(handle_mints(obj, &pre_token_balances, &accounts));
                    }

                }

                meta.inner_instructions
                    .iter()
                    .filter(|inner_instruction| inner_instruction.index == idx as u32)
                    .for_each(|inner_instruction| {
                        inner_instruction.instructions.iter().enumerate().for_each(
                            |(inner_idx, inner_inst)| {
                                let inner_program = &accounts[inner_inst.program_id_index as usize];
                                if inner_program == constants::TOKEN_PROGRAM_ADDRESS {
                                    let outer_arg = get_outer_arg(
                                        inner_inst.data.clone(),
                                        &inner_inst.accounts,
                                        &accounts,
                                    );
                                    let obj: SplTokenMeta = SplTokenMeta {
                                        block_date: convert_to_date(timestamp),
                                        block_time: timestamp,
                                        tx_id: bs58::encode(&transaction.signatures[0])
                                            .into_string(),
                                        dapp: constants::TOKEN_PROGRAM_ADDRESS.to_string(),
                                        block_slot: slot,
                                        instruction_index: idx as u32,
                                        is_inner_instruction: true,
                                        inner_instruction_index: inner_idx as u32,
                                        instruction_type: outer_arg.instruction_type,
                                        input_accounts: outer_arg.input_accounts,
                                        outer_program: program.to_string(),
                                        args: outer_arg.arg,
                                    };
                                    if !filter_token(&obj) {
                                        if bs58::encode(&transaction.signatures[0]).into_string() =="5n7SCH9uM4Ftr63xYnHqSj33H5aZTvQZxYnyvPPXiQ4vRcYLhAHs4X4cLHJzRjCg5PAuUPykgMEJGuozwtVh4JG1"{
                                            data.push(handle_mints(obj, &pre_token_balances, &accounts));
                                        }

                                    }
                                }
                            },
                        )
                    });
            }
        }
    }
    Ok(SplTokens { data })
}

fn handle_mints(
    mut obj: SplTokenMeta,
    pre_token_balances: &Vec<TokenBalance>,
    accounts: &Vec<String>,
) -> SplTokenMeta {
    if obj.instruction_type == INSTRUCTION_TYPE_TRANSFER {
        if let Some(input_accounts) = &mut obj.input_accounts {
            let index = accounts
                .iter()
                .position(|r| r == &input_accounts.source.as_ref().unwrap().as_str())
                .unwrap();
            pre_token_balances
                .iter()
                .filter(|token_balance| token_balance.account_index == index as u32)
                .for_each(|token_balance| {
                    input_accounts.mint = Some(token_balance.mint.clone());
                });
        }
    }
    return obj;
}

fn filter_token(mut obj: &SplTokenMeta) -> bool {
    if obj.instruction_type == INSTRUCTION_TYPE_UNKNOWN  {
        return true
    }
    if obj.instruction_type == INSTRUCTION_TYPE_INITIALIZE_MINT || obj.instruction_type == INSTRUCTION_TYPE_INITIALIZE_MINT2 {
        if let Some(args) = &obj.args {
            // todo most filter ing
            if args.decimals == Some(0) || args.mint_authority == Some(RAYDIUM_AUTHORITY_V4.to_string()) {
               return  true;
            }
        }
    }
    return false;
}

#[derive(Default)]
pub struct OuterArg {
    pub instruction_type: String,
    pub input_accounts: Option<Accounts>,
    pub arg: Option<Arg>,
}

fn get_outer_arg(
    instruction_data: Vec<u8>,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
) -> OuterArg {
    let account_args = prepare_input_accounts(account_indices, accounts);
    let mut outer_arg: OuterArg = OuterArg::default();
    let mut arg: Arg = Arg::default();
    let instruction: Instruction = parse_instruction(instruction_data, account_args);

    outer_arg.input_accounts = Some(Accounts {
        mint: Some(instruction.instruction_accounts.mint),
        rent_sysvar: Some(instruction.instruction_accounts.rent_sysvar),
        account: Some(instruction.instruction_accounts.account),
        owner: Some(instruction.instruction_accounts.owner),
        signer_accounts: instruction.instruction_accounts.signer_accounts,
        source: Some(instruction.instruction_accounts.source),
        destination: Some(instruction.instruction_accounts.destination),
        delegate: Some(instruction.instruction_accounts.delegate),
        authority: Some(instruction.instruction_accounts.authority),
        payer: Some(instruction.instruction_accounts.payer),
        fund_relocation_sys_program: Some(
            instruction.instruction_accounts.fund_relocation_sys_program,
        ),
        funding_account: Some(instruction.instruction_accounts.funding_account),
        mint_funding_sys_program: Some(instruction.instruction_accounts.mint_funding_sys_program),
    });

    match instruction.name.as_str() {
        INSTRUCTION_TYPE_TRANSFER => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_TRANSFER);
            arg.amount = Some(instruction.transfer_args.amount);
        }
        INSTRUCTION_TYPE_APPROVE => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_APPROVE);
            arg.amount = Some(instruction.approve_args.amount);
        }

        INSTRUCTION_TYPE_TRANSFER_CHECKED => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_TRANSFER_CHECKED);
            arg.amount = Some(instruction.transfer_checked_args.amount);
            arg.decimals = Some(i32::from(instruction.transfer_checked_args.decimals));
        }
        INSTRUCTION_TYPE_APPROVE_CHECKED => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_APPROVE_CHECKED);
            arg.amount = Some(instruction.approve_checked_args.amount);
            arg.decimals = Some(i32::from(instruction.approve_checked_args.decimals));
        }

        INSTRUCTION_TYPE_INITIALIZE_MINT => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_INITIALIZE_MINT);
            arg.decimals = Some(i32::from(instruction.initialize_mint_args.decimals));
            arg.mint_authority =
                get_b58_string(instruction.initialize_mint_args.mint_authority.value);
        }
        INSTRUCTION_TYPE_INITIALIZE_MINT2 => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_INITIALIZE_MINT2);
            arg.decimals = Some(i32::from(instruction.initialize_mint2args.decimals));
            arg.mint_authority =
                get_b58_string(instruction.initialize_mint2args.mint_authority.value);
        }
        INSTRUCTION_TYPE_MINT_TO => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_MINT_TO);
            arg.amount = Some(instruction.mint_to_args.amount);
        }
        INSTRUCTION_TYPE_MINT_TO_CHECKED => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_MINT_TO_CHECKED);
            arg.amount = Some(instruction.mint_to_checked_args.amount);
            arg.decimals = Some(i32::from(instruction.mint_to_checked_args.decimals));
        }
        _ => {
            outer_arg.instruction_type = String::from(INSTRUCTION_TYPE_UNKNOWN);
        }
    }
    outer_arg.arg = Some(arg);
    return outer_arg;
}
