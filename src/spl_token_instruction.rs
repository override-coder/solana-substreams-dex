extern crate bs58;
use borsh::{BorshDeserialize, BorshSerialize};

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
    //pub freeze_authority: PubkeyLayout,
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
    pub initializeMintArgs: InitializeMintLayout,
    pub transferArgs: TransferLayout,
    pub approveArgs: ApproveLayout,
    pub mintToArgs: MintToLayout,
    pub transferCheckedArgs: TransferCheckedLayout,
    pub approveCheckedArgs: ApproveCheckedLayout,
    pub mintToCheckedArgs: MintToCheckedLayout,
    pub initializeMint2Args: InitializeMint2Layout,
}

pub fn parse_instruction(bytes_stream: Vec<u8>, accounts: Vec<String>) -> Instruction {
    // let mut bytes_stream = bs58::decode(base58_string).into_vec().unwrap();
    let mut instruction_name = String::default();
    let mut instruction_accounts = InstructionAccounts::default();

    let mut initializeMintArgs: InitializeMintLayout = InitializeMintLayout::default();
    let mut transferArgs: TransferLayout = TransferLayout::default();
    let mut approveArgs: ApproveLayout = ApproveLayout::default();
    let mut mintToArgs: MintToLayout = MintToLayout::default();
    let mut transferCheckedArgs: TransferCheckedLayout = TransferCheckedLayout::default();
    let mut approveCheckedArgs: ApproveCheckedLayout = ApproveCheckedLayout::default();
    let mut mintToCheckedArgs: MintToCheckedLayout = MintToCheckedLayout::default();
    let mut initializeMint2Args: InitializeMint2Layout = InitializeMint2Layout::default();


    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let discriminator: u8 = u8::from(disc_bytes[0]);

    match discriminator {
        0 => {
            instruction_name = String::from("InitializeMint");

            instruction_accounts.mint = accounts.get(0).unwrap().to_string();
            instruction_accounts.rent_sysvar = accounts.get(1).unwrap().to_string();

       //   initializeMintArgs = InitializeMintLayout::try_from_slice(rest).unwrap_or_default();
            initializeMintArgs = InitializeMintLayout::deserialize(&mut rest.clone()).unwrap();
        }
        3 => {
            instruction_name = String::from("Transfer");

            instruction_accounts.source = accounts.get(0).unwrap().to_string();
            instruction_accounts.destination = accounts.get(1).unwrap().to_string();
            instruction_accounts.owner = accounts.get(2).unwrap().to_string();
            if accounts.len() > 3 {
                instruction_accounts.signer_accounts = accounts.split_at(3).1.to_vec();
            }

            if rest.len() > 8 {
                let (rest_split, _) = rest.split_at(8);
                transferArgs = TransferLayout::try_from_slice(rest_split).unwrap();
            } else {
                transferArgs = TransferLayout::try_from_slice(rest).unwrap();
            }
        }
        4 => {
            instruction_name = String::from("Approve");

            instruction_accounts.source = accounts.get(0).unwrap().to_string();
            instruction_accounts.delegate = accounts.get(1).unwrap().to_string();
            instruction_accounts.owner = accounts.get(2).unwrap().to_string();
            if accounts.len() > 3 {
                instruction_accounts.signer_accounts = accounts.split_at(3).1.to_vec();
            }

            if rest.len() > 8 {
                let (rest_split, _) = rest.split_at(8);
                approveArgs = ApproveLayout::try_from_slice(rest_split).unwrap();
            } else {
                approveArgs = ApproveLayout::try_from_slice(rest).unwrap();
            }
        }

        7 => {
            instruction_name = String::from("MintTo");

            instruction_accounts.mint = accounts.get(0).unwrap().to_string();
            instruction_accounts.account = accounts.get(1).unwrap().to_string();
            instruction_accounts.authority = accounts.get(2).unwrap().to_string();
            if accounts.len() > 3 {
                instruction_accounts.signer_accounts = accounts.split_at(3).1.to_vec();
            }

            if rest.len() > 8 {
                let (rest_split, _) = rest.split_at(8);
                mintToArgs = MintToLayout::try_from_slice(rest_split).unwrap();
            } else {
                mintToArgs = MintToLayout::try_from_slice(rest).unwrap();
            }
        }
        12 => {
            instruction_name = String::from("TransferChecked");

            instruction_accounts.source = accounts.get(0).unwrap().to_string();
            instruction_accounts.mint = accounts.get(1).unwrap().to_string();
            instruction_accounts.destination = accounts.get(2).unwrap().to_string();
            instruction_accounts.owner = accounts.get(3).unwrap().to_string();
            if accounts.len() > 4 {
                instruction_accounts.signer_accounts = accounts.split_at(4).1.to_vec();
            }

            transferCheckedArgs = TransferCheckedLayout::deserialize(&mut rest.clone()).unwrap();
        }
        13 => {
            instruction_name = String::from("ApproveChecked");

            instruction_accounts.source = accounts.get(0).unwrap().to_string();
            instruction_accounts.mint = accounts.get(1).unwrap().to_string();
            instruction_accounts.delegate = accounts.get(2).unwrap().to_string();
            instruction_accounts.owner = accounts.get(3).unwrap().to_string();
            if accounts.len() > 4 {
                instruction_accounts.signer_accounts = accounts.split_at(4).1.to_vec();
            }

            approveCheckedArgs = ApproveCheckedLayout::try_from_slice(rest).unwrap();
        }
        14 => {
            instruction_name = String::from("MintToChecked");

            instruction_accounts.mint = accounts.get(0).unwrap().to_string();
            instruction_accounts.account = accounts.get(1).unwrap().to_string();
            instruction_accounts.authority = accounts.get(2).unwrap().to_string();
            if accounts.len() > 3 {
                instruction_accounts.signer_accounts = accounts.split_at(3).1.to_vec();
            }

            mintToCheckedArgs = MintToCheckedLayout::try_from_slice(rest).unwrap();
        }
        20 => {
            instruction_name = String::from("InitializeMint2");
            instruction_accounts.mint = accounts.get(0).unwrap().to_string();
            //initializeMint2Args = InitializeMint2Layout::try_from_slice(rest).unwrap_or_default();
            initializeMint2Args = InitializeMint2Layout::deserialize(&mut rest.clone()).unwrap();
        }
        _ => {}
    }

    let result: Instruction = Instruction {
        name: instruction_name,
        instruction_accounts: instruction_accounts,
        initializeMintArgs: initializeMintArgs,
        transferArgs: transferArgs,
        approveArgs: approveArgs,
        mintToArgs: mintToArgs,
        transferCheckedArgs: transferCheckedArgs,
        approveCheckedArgs: approveCheckedArgs,
        mintToCheckedArgs: mintToCheckedArgs,
        initializeMint2Args: initializeMint2Args,
    };
    return result;
}
