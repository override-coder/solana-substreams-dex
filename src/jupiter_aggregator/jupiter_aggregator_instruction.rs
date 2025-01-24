use crate::constants::{JUPITER_AGGREGATOR_V6_EVENT_AUTHORITY, JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS};
use crate::utils::prepare_input_accounts;
use borsh::{BorshDeserialize, BorshSerialize};
use bs58;

const ROUTE_DISCRIMINATOR: u64 = u64::from_le_bytes([229, 23, 203, 151, 122, 227, 173, 42]);
const SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR: u64 = u64::from_le_bytes([193, 32, 155, 51, 65, 214, 156, 129]);
const EXACT_OUT_ROUTE_DISCRIMINATOR: u64 = u64::from_le_bytes([208, 51, 239, 151, 123, 43, 237, 92]);

const SWAP_EVENT_DISCRIMINATOR: u64 = u64::from_le_bytes([64, 198, 205, 232, 38, 8, 113, 226]);

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct InstructionRoutePlan {
    pub in_amount: u64,
    pub quoted_out_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Clone)]
pub struct InstructionSwapEvent {
    pub amm: [u8; 32],
    pub input_mint: [u8; 32],
    pub input_amount: u64,
    pub output_mint: [u8; 32],
    pub output_amount: u64,
}

#[derive(Debug, Default)]
pub struct RouterInstruction {
    pub signer: String,
    pub instruction_types: String,
    pub source_token_account: String,
    pub destination_token_account: String,
    pub source_mint: String,
    pub destination_mint: String,
    pub in_amount: String,
    pub quoted_out_amount: String,
}

pub fn parse_instruction(
    program: &String,
    bytes_stream: Vec<u8>,
    account_indices: &Vec<u8>,
    input_accounts: &Vec<String>,
) -> Option<RouterInstruction> {
    if program != JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS {
        return None;
    }
    let accounts = prepare_input_accounts(account_indices, input_accounts);
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let discriminator: u64 = u64::from_le_bytes(disc_bytes.try_into().unwrap());
    match discriminator {
        ROUTE_DISCRIMINATOR => Some(RouterInstruction {
            signer: accounts.get(1).resolved_or_empty(),
            instruction_types: "Router".to_string(),
            source_token_account: accounts.get(2).resolved_or_empty(),
            destination_token_account: accounts.get(3).resolved_or_empty(),
            source_mint: "".to_string(),
            destination_mint: accounts.get(5).resolved_or_empty(),
            in_amount: "".to_string(),
            quoted_out_amount: "".to_string(),
        }),
        SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR => Some(RouterInstruction {
            signer: accounts.get(2).resolved_or_empty(),
            instruction_types: "SharedAccountsRoute".to_string(),
            source_token_account: accounts.get(3).resolved_or_empty(),
            destination_token_account: accounts.get(6).resolved_or_empty(),
            source_mint: accounts.get(7).resolved_or_empty(),
            destination_mint: accounts.get(8).resolved_or_empty(),
            in_amount: "".to_string(),
            quoted_out_amount: "".to_string(),
        }),
        EXACT_OUT_ROUTE_DISCRIMINATOR => Some(RouterInstruction {
            signer: accounts.get(1).resolved_or_empty(),
            instruction_types: "ExactOutRoute".to_string(),
            source_token_account: accounts.get(2).resolved_or_empty(),
            destination_token_account: accounts.get(3).resolved_or_empty(),
            source_mint: accounts.get(5).resolved_or_empty(),
            destination_mint: accounts.get(6).resolved_or_empty(),
            in_amount: "".to_string(),
            quoted_out_amount: "".to_string(),
        }),
        _ => None,
    }
}

pub fn parse_inner_instruction(
    program: &String,
    bytes_stream: &Vec<u8>,
    account_indices: &Vec<u8>,
    input_accounts: &Vec<String>,
) -> Option<InstructionSwapEvent> {
    if program != JUPITER_AGGREGATOR_V6_PROGRAM_ADDRESS {
        return None;
    }
    let accounts = prepare_input_accounts(account_indices, input_accounts);
    let account = accounts.get(0).resolved_or_empty();
    if account != JUPITER_AGGREGATOR_V6_EVENT_AUTHORITY {
        return None;
    }
    let (event, rest) = bytes_stream.split_at(16);
    let (_, disc_bytes) = event.split_at(8);
    let discriminator: u64 = u64::from_le_bytes(disc_bytes.try_into().unwrap());
    match discriminator {
        SWAP_EVENT_DISCRIMINATOR => {
            let mut rest_slice = &mut &rest[..];
            let mut event = InstructionSwapEvent::default();
            event = InstructionSwapEvent::deserialize(&mut rest_slice).unwrap();
            Some(event)
        }
        _ => None,
    }
}

pub trait AccountResolver {
    fn resolved_or_empty(&self) -> String;
}

impl AccountResolver for Option<&&String> {
    fn resolved_or_empty(&self) -> String {
        self.map(|x| *x).cloned().unwrap_or_default()
    }
}
