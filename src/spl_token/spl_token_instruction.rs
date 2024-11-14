extern crate bs58;
use borsh::{BorshDeserialize, BorshSerialize};

pub const INSTRUCTION_TYPE_TRANSFER: &str = "Transfer";
pub const INSTRUCTION_TYPE_APPROVE: &str = "Approve";
pub const INSTRUCTION_TYPE_TRANSFER_CHECKED: &str = "TransferChecked";
pub const INSTRUCTION_TYPE_APPROVE_CHECKED: &str = "ApproveChecked";
pub const INSTRUCTION_TYPE_INITIALIZE_MINT: &str = "InitializeMint";
pub const INSTRUCTION_TYPE_INITIALIZE_MINT2: &str = "InitializeMint2";
pub const INSTRUCTION_TYPE_MINT_TO: &str = "MintTo";
pub const INSTRUCTION_TYPE_MINT_TO_CHECKED: &str = "MintToChecked";
pub const INSTRUCTION_TYPE_UNKNOWN: &str = "Unknown Instruction";


#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default, Copy)]
pub struct PubkeyLayout {
    pub value: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct InitializeMintLayout {
    pub decimals: u8,
    pub mint_authority: PubkeyLayout,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct TransferLayout {
    pub amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct ApproveLayout {
    pub amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct MintToLayout {
    pub amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct TransferCheckedLayout {
    pub amount: u64,
    pub decimals: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct ApproveCheckedLayout {
    pub amount: u64,
    pub decimals: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct MintToCheckedLayout {
    pub amount: u64,
    pub decimals: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct InitializeAccount3Layout {
    pub owner: PubkeyLayout,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct InitializeMint2Layout {
    pub decimals: u8,
    pub mint_authority: PubkeyLayout,
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct ReallocateLayout {}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct InstructionAccounts {
    pub mint: String,
    pub rent_sysvar: String,
    pub account: String,
    pub owner: String,
    pub signer_accounts: Vec<String>,
    pub source: String,
    pub destination: String,
    pub delegate: String,
    pub authority: String,
    pub payer: String,
    pub fund_relocation_sys_program: String,
    pub funding_account: String,
    pub mint_funding_sys_program: String,
}

#[derive(Debug)]
pub struct Instruction {
    pub name: String,
    pub instruction_accounts: InstructionAccounts,
    pub initialize_mint_args: InitializeMintLayout,
    pub transfer_args: TransferLayout,
    pub approve_args: ApproveLayout,
    pub mint_to_args: MintToLayout,
    pub transfer_checked_args: TransferCheckedLayout,
    pub approve_checked_args: ApproveCheckedLayout,
    pub mint_to_checked_args: MintToCheckedLayout,
    pub initialize_mint2args: InitializeMint2Layout,
}

pub fn parse_instruction(bytes_stream: Vec<u8>, accounts: Vec<String>) -> Instruction {
    let mut instruction_name = String::default();
    let mut instruction_accounts = InstructionAccounts::default();

    let mut initialize_mint_args: InitializeMintLayout = InitializeMintLayout::default();
    let mut transfer_args: TransferLayout = TransferLayout::default();
    let mut approve_args: ApproveLayout = ApproveLayout::default();
    let mut mint_to_args: MintToLayout = MintToLayout::default();
    let mut transfer_checked_args: TransferCheckedLayout = TransferCheckedLayout::default();
    let mut approve_checked_args: ApproveCheckedLayout = ApproveCheckedLayout::default();
    let mut mint_to_checked_args: MintToCheckedLayout = MintToCheckedLayout::default();
    let mut initialize_mint2args: InitializeMint2Layout = InitializeMint2Layout::default();


    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let discriminator: u8 = u8::from(disc_bytes[0]);

    match discriminator {
        0 => {
            instruction_name = String::from(INSTRUCTION_TYPE_INITIALIZE_MINT);
            instruction_accounts.mint = accounts.get(0).unwrap().to_string();
            instruction_accounts.rent_sysvar = accounts.get(1).unwrap().to_string();
            initialize_mint_args = InitializeMintLayout::deserialize(&mut rest.clone()).unwrap();
        }
        3 => {
            instruction_name = String::from(INSTRUCTION_TYPE_TRANSFER);

            instruction_accounts.source = accounts.get(0).unwrap().to_string();
            instruction_accounts.destination = accounts.get(1).unwrap().to_string();
            instruction_accounts.owner = accounts.get(2).unwrap().to_string();
            if accounts.len() > 3 {
                instruction_accounts.signer_accounts = accounts.split_at(3).1.to_vec();
            }

            if rest.len() > 8 {
                let (rest_split, _) = rest.split_at(8);
                transfer_args = TransferLayout::try_from_slice(rest_split).unwrap();
            } else {
                transfer_args = TransferLayout::try_from_slice(rest).unwrap();
            }
        }
        4 => {
            instruction_name = String::from(INSTRUCTION_TYPE_APPROVE);

            instruction_accounts.source = accounts.get(0).unwrap().to_string();
            instruction_accounts.delegate = accounts.get(1).unwrap().to_string();
            instruction_accounts.owner = accounts.get(2).unwrap().to_string();
            if accounts.len() > 3 {
                instruction_accounts.signer_accounts = accounts.split_at(3).1.to_vec();
            }
            if rest.len() > 8 {
                let (rest_split, _) = rest.split_at(8);
                approve_args = ApproveLayout::try_from_slice(rest_split).unwrap();
            } else {
                approve_args = ApproveLayout::try_from_slice(rest).unwrap();
            }
        }

        7 => {
            instruction_name = String::from(INSTRUCTION_TYPE_MINT_TO);

            instruction_accounts.mint = accounts.get(0).unwrap().to_string();
            instruction_accounts.account = accounts.get(1).unwrap().to_string();
            instruction_accounts.authority = accounts.get(2).unwrap().to_string();
            if accounts.len() > 3 {
                instruction_accounts.signer_accounts = accounts.split_at(3).1.to_vec();
            }

            if rest.len() > 8 {
                let (rest_split, _) = rest.split_at(8);
                mint_to_args = MintToLayout::try_from_slice(rest_split).unwrap();
            } else {
                mint_to_args = MintToLayout::try_from_slice(rest).unwrap();
            }
        }
        12 => {
            instruction_name = String::from(INSTRUCTION_TYPE_TRANSFER_CHECKED);

            instruction_accounts.source = accounts.get(0).unwrap().to_string();
            instruction_accounts.mint = accounts.get(1).unwrap().to_string();
            instruction_accounts.destination = accounts.get(2).unwrap().to_string();
            instruction_accounts.owner = accounts.get(3).unwrap().to_string();
            if accounts.len() > 4 {
                instruction_accounts.signer_accounts = accounts.split_at(4).1.to_vec();
            }

            transfer_checked_args = TransferCheckedLayout::deserialize(&mut rest.clone()).unwrap();
        }
        13 => {
            instruction_name = String::from(INSTRUCTION_TYPE_APPROVE_CHECKED);

            instruction_accounts.source = accounts.get(0).unwrap().to_string();
            instruction_accounts.mint = accounts.get(1).unwrap().to_string();
            instruction_accounts.delegate = accounts.get(2).unwrap().to_string();
            instruction_accounts.owner = accounts.get(3).unwrap().to_string();
            if accounts.len() > 4 {
                instruction_accounts.signer_accounts = accounts.split_at(4).1.to_vec();
            }

            approve_checked_args = ApproveCheckedLayout::try_from_slice(rest).unwrap();
        }
        14 => {
            instruction_name = String::from(INSTRUCTION_TYPE_MINT_TO_CHECKED);

            instruction_accounts.mint = accounts.get(0).unwrap().to_string();
            instruction_accounts.account = accounts.get(1).unwrap().to_string();
            instruction_accounts.authority = accounts.get(2).unwrap().to_string();
            if accounts.len() > 3 {
                instruction_accounts.signer_accounts = accounts.split_at(3).1.to_vec();
            }

            mint_to_checked_args = MintToCheckedLayout::try_from_slice(rest).unwrap();
        }
        20 => {
            instruction_name = String::from(INSTRUCTION_TYPE_INITIALIZE_MINT2);
            instruction_accounts.mint = accounts.get(0).unwrap().to_string();
            initialize_mint2args = InitializeMint2Layout::deserialize(&mut rest.clone()).unwrap();
        }
        _ => {}
    }

    let result: Instruction = Instruction {
        name: instruction_name,
        instruction_accounts,
        initialize_mint_args,
        transfer_args,
        approve_args,
        mint_to_args,
        transfer_checked_args,
        approve_checked_args,
        mint_to_checked_args,
        initialize_mint2args,
    };
    return result;
}
