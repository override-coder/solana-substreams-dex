use bs58;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::constants::JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS;
use crate::utils::prepare_input_accounts;

const ROUTE_DISCRIMINATOR: u64 = u64::from_le_bytes([229, 23, 203, 151, 122, 227, 173, 42]);
const SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR: u64 = u64::from_le_bytes([193, 32, 155, 51, 65, 214, 156, 129]);
const EXACT_OUT_ROUTE_DISCRIMINATOR: u64 = u64::from_le_bytes([208, 51, 239, 151, 123, 43, 237, 92,]);

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct InstructionRoutePlan {
    pub in_amount: u64,
    pub quoted_out_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u8,
}

#[derive(Debug, Default)]
pub struct Instruction {
    pub signer: String,
    pub instruction_types: String,
    pub source_token_account: String,
    pub destination_token_account: String,
    pub source_mint: String,
    pub destination_mint: String,
    pub in_amount: String,
    pub quoted_out_amount: String,
}
pub fn parse_instruction(program: &String,bytes_stream: Vec<u8>, account_indices: &Vec<u8>, input_accounts: &Vec<String>) -> Option<Instruction> {
    if program != JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS {
        return None
    }
    let mut plan = InstructionRoutePlan::default();
    let accounts = prepare_input_accounts(account_indices, input_accounts);

    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let discriminator: u64 = u64::from_le_bytes(disc_bytes.try_into().unwrap());

    if let Some(parsed_plan) = parse_route_plan(rest) {
        plan = parsed_plan;
    } else {
        return None;
    }
    match discriminator {
        ROUTE_DISCRIMINATOR => {
            Some(Instruction {
                signer: accounts.get(1).unwrap_or(&"".to_string()).to_string(),
                instruction_types: "Router".to_string(),
                source_token_account: accounts.get(2).unwrap_or(&"".to_string()).to_string(),
                destination_token_account: accounts.get(3).unwrap_or(&"".to_string()).to_string(),
                source_mint: "".to_string(),
                destination_mint: accounts.get(5).unwrap_or(&"".to_string()).to_string(),
                in_amount: plan.in_amount.to_string(),
                quoted_out_amount: plan.quoted_out_amount.to_string(),
            })
        }
        SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR => {
            Some(Instruction {
                signer: accounts.get(2).unwrap_or(&"".to_string()).to_string(),
                instruction_types:"SharedAccountsRoute".to_string(),
                source_token_account: accounts.get(3).unwrap_or(&"".to_string()).to_string(),
                destination_token_account: accounts.get(6).unwrap_or(&"".to_string()).to_string(),
                source_mint: accounts.get(7).unwrap_or(&"".to_string()).to_string(),
                destination_mint: accounts.get(8).unwrap_or(&"".to_string()).to_string(),
                in_amount: plan.in_amount.to_string(),
                quoted_out_amount: plan.quoted_out_amount.to_string(),
            })
        }
        EXACT_OUT_ROUTE_DISCRIMINATOR => {
            Some(Instruction {
                signer: accounts.get(1).unwrap_or(&"".to_string()).to_string(),
                instruction_types:"ExactOutRoute".to_string(),
                source_token_account: accounts.get(2).unwrap_or(&"".to_string()).to_string(),
                destination_token_account: accounts.get(3).unwrap_or(&"".to_string()).to_string(),
                source_mint: accounts.get(5).unwrap_or(&"".to_string()).to_string(),
                destination_mint: accounts.get(6).unwrap_or(&"".to_string()).to_string(),
                in_amount: plan.in_amount.to_string(),
                quoted_out_amount: plan.quoted_out_amount.to_string(),
            })
        }
        _ => None,
    }
}

fn parse_route_plan(rest: &[u8]) -> Option<InstructionRoutePlan> {
    let last_19_bytes = if rest.len() >= 19 {
        &rest[rest.len() - 19..]
    } else {
        rest
    };
    InstructionRoutePlan::try_from_slice(last_19_bytes).ok()
}