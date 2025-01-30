use crate::pb::sf::solana::dex::meta::v1::{
    Arg, InputAccounts, PbAssetDataLayout, PbCollectionDetailsLayout, PbCollectionLayout, PbCreateArgsLayout,
    PbCreateMetadataAccountArgsLayout, PbCreateMetadataAccountArgsV2Layout, PbCreateMetadataAccountArgsV3Layout,
    PbCreatorLayout, PbDataLayout, PbDataV2Layout, PbPrintSupplyLayout, PbUsesLayout,
};
use crate::utils::get_b58_string;
use borsh::{BorshDeserialize, BorshSerialize};

pub const INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT: &str = "CreateMetadataAccount";
pub const INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V2: &str = "CreateMetadataAccountV2";
pub const INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V3: &str = "CreateMetadataAccountV3";
pub const INSTRUCTION_TYPE_CREATE: &str = "Create";

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default, Copy)]
pub struct PubKeyLayout {
    pub value: [u8; 32],
}

impl PubKeyLayout {
    pub fn to_proto_struct(&self) -> String {
        get_b58_string(self.value).unwrap_or_default()
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CreatorLayout {
    pub address: PubKeyLayout,
    pub verified: bool,
    pub share: u8,
}

impl CreatorLayout {
    pub fn to_proto_struct(&self) -> PbCreatorLayout {
        PbCreatorLayout {
            address: self.address.to_proto_struct(),
            verified: self.verified,
            share: self.share as u32,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CollectionLayout {
    pub verified: bool,
    pub key: PubKeyLayout,
}

impl CollectionLayout {
    pub fn to_proto_struct(&self) -> PbCollectionLayout {
        PbCollectionLayout {
            verified: self.verified,
            key: self.key.to_proto_struct(),
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
#[repr(u8)]
pub enum UseMethodLayout {
    #[default]
    Burn,
    Multiple,
    Single,
}

impl UseMethodLayout {
    pub fn to_proto_struct(&self) -> String {
        let mut result = "".to_string();
        match self {
            UseMethodLayout::Burn => result = "Burn".to_string(),
            UseMethodLayout::Multiple => result = "Multiple".to_string(),
            UseMethodLayout::Single => result = "Single".to_string(),
        }
        result
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct UsesLayout {
    pub useMethod: UseMethodLayout,
    pub remaining: u64,
    pub total: u64,
}

impl UsesLayout {
    pub fn to_proto_struct(&self) -> PbUsesLayout {
        PbUsesLayout {
            use_method: self.useMethod.to_proto_struct(),
            remaining: self.remaining,
            total: self.total,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub enum CollectionDetailsLayoutName {
    #[default]
    V1,
}

impl CollectionDetailsLayoutName {
    pub fn to_proto_struct(&self) -> String {
        let mut result = "".to_string();

        match self {
            CollectionDetailsLayoutName::V1 => {
                result = "V1".to_string();
            }
        }

        result
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CollectionDetailsLayout {
    pub name: CollectionDetailsLayoutName,
    pub size: u64,
}

impl CollectionDetailsLayout {
    pub fn to_proto_struct(&self) -> PbCollectionDetailsLayout {
        PbCollectionDetailsLayout {
            name: self.name.to_proto_struct(),
            size: self.size,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct DataLayout {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<CreatorLayout>>,
}

impl DataLayout {
    pub fn to_proto_struct(&self) -> PbDataLayout {
        let mut creators: Vec<PbCreatorLayout> = vec![];
        if self.creators.is_some() {
            for x in self.creators.as_ref().unwrap().iter() {
                creators.push(x.to_proto_struct());
            }
        }

        PbDataLayout {
            name: self.name.to_string(),
            symbol: self.symbol.to_string(),
            uri: self.uri.to_string(),
            seller_fee_basis_points: self.seller_fee_basis_points as u32,
            creators: creators,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
#[repr(u8)]
pub enum CreateArgsLayoutName {
    #[default]
    V1,
}

impl CreateArgsLayoutName {
    pub fn to_proto_struct(&self) -> String {
        let mut result = "".to_string();

        match self {
            CreateArgsLayoutName::V1 => {
                result = "V1".to_string();
            }
        }

        result
    }
}

#[derive(BorshDeserialize, Debug, Default)]
#[repr(u8)]
pub enum TokenStandardLayout {
    #[default]
    NonFungible,
    FungibleAsset,
    Fungible,
    NonFungibleEdition,
    ProgrammableNonFungible,
}

impl TokenStandardLayout {
    pub fn to_proto_struct(&self) -> String {
        let mut result = "NonFungible".to_string();

        match self {
            TokenStandardLayout::NonFungible => {
                result = "NonFungible".to_string();
            }
            TokenStandardLayout::FungibleAsset => {
                result = "FungibleAsset".to_string();
            }
            TokenStandardLayout::Fungible => {
                result = "Fungible".to_string();
            }
            TokenStandardLayout::NonFungibleEdition => {
                result = "NonFungibleEdition".to_string();
            }
            TokenStandardLayout::ProgrammableNonFungible => {
                result = "ProgrammableNonFungible".to_string();
            }
        }

        return result;
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct AssetDataLayout {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub sellerFeeBasisPoints: u16,
    pub creators: Option<Vec<CreatorLayout>>,
    pub primarySaleHappened: bool,
    pub isMutable: bool,
    pub tokenStandard: TokenStandardLayout,
    pub collection: Option<CollectionLayout>,
    pub uses: Option<UsesLayout>,
    pub collectionDetails: Option<CollectionDetailsLayout>,
    pub ruleSet: Option<PubKeyLayout>,
}

impl AssetDataLayout {
    pub fn to_proto_struct(&self) -> PbAssetDataLayout {
        let mut creators: Vec<PbCreatorLayout> = vec![];
        if self.creators.is_some() {
            for x in self.creators.as_ref().unwrap().iter() {
                creators.push(x.to_proto_struct());
            }
        }

        let mut collection: Option<PbCollectionLayout> = None;
        if self.collection.is_some() {
            collection = Some(self.collection.as_ref().unwrap().to_proto_struct());
        }

        let mut uses: Option<PbUsesLayout> = None;
        if self.uses.is_some() {
            uses = Some(self.uses.as_ref().unwrap().to_proto_struct());
        }

        let mut collection_details: Option<PbCollectionDetailsLayout> = None;
        if self.collectionDetails.is_some() {
            collection_details = Some(self.collectionDetails.as_ref().unwrap().to_proto_struct());
        }

        let mut rule_set: Option<String> = None;
        if self.ruleSet.is_some() {
            rule_set = Some(self.ruleSet.unwrap().to_proto_struct());
        }

        PbAssetDataLayout {
            name: self.name.to_string(),
            symbol: self.symbol.to_string(),
            uri: self.uri.to_string(),
            seller_fee_basis_points: self.sellerFeeBasisPoints as u32,
            creators: creators,
            primary_sale_happened: self.primarySaleHappened,
            is_mutable: self.isMutable,
            token_standard: self.tokenStandard.to_proto_struct(),
            collection: collection,
            uses: uses,
            collection_details: collection_details,
            rule_set: rule_set,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct PrintSupplyLayout {
    pub name: PrintSupplyLayoutName,
}

impl PrintSupplyLayout {
    pub fn to_proto_struct(&self) -> PbPrintSupplyLayout {
        let mut name: String = "Zero".to_string();
        let mut val = None;

        match &self.name {
            PrintSupplyLayoutName::Zero => {
                name = "Zero".to_string();
            }
            PrintSupplyLayoutName::Limited { val: value } => {
                name = "Limited".to_string();
                val = Some(*value);
            }
            PrintSupplyLayoutName::Unlimited => {
                name = "Unlimited".to_string();
            }
        }

        PbPrintSupplyLayout { name: name, val: val }
    }
}

#[derive(BorshDeserialize, Debug, Default, Clone)]
pub enum PrintSupplyLayoutName {
    #[default]
    Zero,
    Limited {
        val: u64,
    },
    Unlimited,
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CreateArgsLayout {
    pub name: CreateArgsLayoutName,
    pub asset_data: AssetDataLayout,
    pub decimals: Option<u8>,
    pub print_supply: Option<PrintSupplyLayout>,
}

impl CreateArgsLayout {
    pub fn to_proto_struct(&mut self) -> PbCreateArgsLayout {
        let mut decimals: Option<u32> = None;
        if self.decimals.is_some() {
            decimals = Some(self.decimals.unwrap() as u32);
        }

        let mut print_supply: Option<PbPrintSupplyLayout> = None;
        if self.print_supply.is_some() {
            print_supply = Some(self.print_supply.as_ref().unwrap().to_proto_struct());
        }

        PbCreateArgsLayout {
            name: self.name.to_proto_struct(),
            asset_data: Some(self.asset_data.to_proto_struct()),
            decimals: decimals,
            print_supply: print_supply,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CreateMetadataAccountArgsLayout {
    pub data: DataLayout,
    pub is_mutable: bool,
}

impl CreateMetadataAccountArgsLayout {
    pub fn to_proto_struct(&self) -> PbCreateMetadataAccountArgsLayout {
        PbCreateMetadataAccountArgsLayout {
            data: Some(self.data.to_proto_struct()),
            is_mutable: self.is_mutable,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct DataV2Layout {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<CreatorLayout>>,
    pub collection: Option<CollectionLayout>,
    pub uses: Option<UsesLayout>,
}

impl DataV2Layout {
    pub fn to_proto_struct(&self) -> PbDataV2Layout {
        let mut creators: Vec<PbCreatorLayout> = vec![];
        if self.creators.is_some() {
            for x in self.creators.as_ref().unwrap().iter() {
                creators.push(x.to_proto_struct());
            }
        }

        let mut collection: Option<PbCollectionLayout> = None;
        if self.collection.is_some() {
            collection = Some(self.collection.as_ref().unwrap().to_proto_struct());
        }

        let mut uses: Option<PbUsesLayout> = None;
        if self.uses.is_some() {
            uses = Some(self.uses.as_ref().unwrap().to_proto_struct());
        }

        PbDataV2Layout {
            name: self.name.to_string(),
            symbol: self.symbol.to_string(),
            uri: self.uri.to_string(),
            seller_fee_basis_points: self.seller_fee_basis_points as u32,
            creators,
            collection,
            uses,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CreateMetadataAccountArgsV2Layout {
    pub data: DataV2Layout,
    pub is_mutable: bool,
}

impl CreateMetadataAccountArgsV2Layout {
    pub fn to_proto_struct(&self) -> PbCreateMetadataAccountArgsV2Layout {
        PbCreateMetadataAccountArgsV2Layout {
            data: Some(self.data.to_proto_struct()),
            is_mutable: self.is_mutable,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CreateMetadataAccountArgsV3Layout {
    pub data: DataV2Layout,
    pub is_mutable: bool,
    pub collection_details: Option<CollectionDetailsLayout>,
}

impl CreateMetadataAccountArgsV3Layout {
    pub fn to_proto_struct(&self) -> PbCreateMetadataAccountArgsV3Layout {
        let mut collection_details: Option<PbCollectionDetailsLayout> = None;
        if self.collection_details.is_some() {
            collection_details = Some(self.collection_details.as_ref().unwrap().to_proto_struct());
        }

        PbCreateMetadataAccountArgsV3Layout {
            data: Some(self.data.to_proto_struct()),
            is_mutable: self.is_mutable,
            collection_details: collection_details,
        }
    }
}

#[derive(Debug, Default)]
pub struct Instruction {
    pub instruction_type: String,
    pub create_args: CreateArgsLayout,
    pub create_metadata_account_args: CreateMetadataAccountArgsLayout,
    pub create_metadata_account_args_v2: CreateMetadataAccountArgsV2Layout,
    pub create_metadata_account_args_v3: CreateMetadataAccountArgsV3Layout,
}

pub fn parse_instruction(bytes_stream: &Vec<u8>) -> Instruction {
    let mut result: Instruction = Instruction::default();

    let (disc_bytes, rest) = bytes_stream.split_at(1);
    let discriminator: u8 = u8::from(disc_bytes[0]);
    let rest_bytes = &mut rest.clone();

    match discriminator {
        0 => {
            result.instruction_type = INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT.to_string();
            if rest_bytes.len() > 0 {
                result.create_metadata_account_args = CreateMetadataAccountArgsLayout::deserialize(rest_bytes).unwrap();
            }
        }
        16 => {
            result.instruction_type = INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V2.to_string();
            if rest_bytes.len() > 0 {
                result.create_metadata_account_args_v2 =
                    CreateMetadataAccountArgsV2Layout::deserialize(rest_bytes).unwrap();
            }
        }
        33 => {
            result.instruction_type = INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V3.to_string();
            if rest_bytes.len() > 0 {
                result.create_metadata_account_args_v3 =
                    CreateMetadataAccountArgsV3Layout::deserialize(rest_bytes).unwrap();
            }
        }
        42 => {
            result.instruction_type = INSTRUCTION_TYPE_CREATE.to_string();
            if rest_bytes.len() > 0 {
                result.create_args = CreateArgsLayout::deserialize(rest_bytes).unwrap_or(CreateArgsLayout::default());
            }
        }
        _ => {}
    }
    return result;
}

pub fn prepare_arg(instruction_data: &Vec<u8>) -> Arg {
    let mut arg: Arg = Arg::default();
    let mut instruction: Instruction = parse_instruction(&instruction_data);

    arg.instruction_type = instruction.instruction_type;

    match arg.instruction_type.as_str() {
        INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT => {
            arg.create_metadata_account_args = Some(instruction.create_metadata_account_args.to_proto_struct());
        }
        INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V2 => {
            arg.create_metadata_account_args_v2 = Some(instruction.create_metadata_account_args_v2.to_proto_struct());
        }
        INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V3 => {
            arg.create_metadata_account_args_v3 = Some(instruction.create_metadata_account_args_v3.to_proto_struct());
        }
        INSTRUCTION_TYPE_CREATE => {
            arg.create_args = Some(instruction.create_args.to_proto_struct());
        }
        _ => {}
    }
    return arg;
}

pub fn prepare_input_accounts(
    instruction_type: String,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
) -> Option<InputAccounts> {
    let input_accounts = populate_input_accounts(account_indices, accounts);

    let mut result = InputAccounts::default();
    match instruction_type.as_str() {
        INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT => {
            result.metadata = get_account_with(&input_accounts, 0);
            result.mint = get_account_with(&input_accounts, 1);
            result.mint_authority = get_account_with(&input_accounts, 2);
            result.payer = get_account_with(&input_accounts, 3);
            result.update_authority = get_account_with(&input_accounts, 4);
            result.system_program = get_account_with(&input_accounts, 5);
            result.rent = get_account_with(&input_accounts, 6);
        }

        INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V2 => {
            result.metadata = get_account_with(&input_accounts, 0);
            result.mint = get_account_with(&input_accounts, 1);
            result.mint_authority = get_account_with(&input_accounts, 2);
            result.payer = get_account_with(&input_accounts, 3);
            result.update_authority = get_account_with(&input_accounts, 4);
            result.system_program = get_account_with(&input_accounts, 5);
            result.rent = get_account_with(&input_accounts, 6);
        }
        INSTRUCTION_TYPE_CREATE_METADATA_ACCOUNT_V3 => {
            result.metadata = get_account_with(&input_accounts, 0);
            result.mint = get_account_with(&input_accounts, 1);
            result.mint_authority = get_account_with(&input_accounts, 2);
            result.payer = get_account_with(&input_accounts, 3);
            result.update_authority = get_account_with(&input_accounts, 4);
            result.system_program = get_account_with(&input_accounts, 5);
            result.rent = get_account_with(&input_accounts, 6);
        }
        INSTRUCTION_TYPE_CREATE => {
            result.metadata = get_account_with(&input_accounts, 0);
            result.mint = get_account_with(&input_accounts, 2);
            result.payer = get_account_with(&input_accounts, 4);
            result.update_authority = get_account_with(&input_accounts, 5);
            result.system_program = get_account_with(&input_accounts, 6);
        }
        _ => {}
    }

    return Some(result);
}

fn get_account_with(accounts: &Vec<String>, index: usize) -> Option<String> {
    let mut result: Option<String> = None;
    let account = accounts.get(index);
    if account.is_some() {
        result = Some(account.unwrap().to_string());
    }
    return result;
}

fn populate_input_accounts(account_indices: &Vec<u8>, accounts: &Vec<String>) -> Vec<String> {
    let mut instruction_accounts: Vec<String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        instruction_accounts.push(accounts.as_slice()[el as usize].to_string());
    }
    return instruction_accounts;
}
