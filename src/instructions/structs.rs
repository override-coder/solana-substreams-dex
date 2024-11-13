use std::vec;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::pb::sf::solana::dex::meta::v1::{
    PbCollectionDetailsLayout,
    PbCollectionLayout,
    PbCreateMetadataAccountArgsLayout,
    PbCreateMetadataAccountArgsV2Layout, PbCreateMetadataAccountArgsV3Layout, PbCreatorLayout,
    PbDataLayout, PbDataV2Layout,
    PbUsesLayout,
};

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Default, Copy)]
pub struct PubKeyLayout {
    pub value: [u8; 32],
}

impl PubKeyLayout {
    pub fn to_proto_struct(&self) -> String {
        let result = get_b58_string(self.value);
        result
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
    pub sellerFeeBasisPoints: u16,
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
            seller_fee_basis_points: self.sellerFeeBasisPoints as u32,
            creators: creators,
        }
    }
}

//
// Instruction Layouts
//

#[derive(BorshDeserialize, Debug, Default)]
pub struct CreateMetadataAccountArgsLayout {
    pub data: DataLayout,
    pub isMutable: bool,
}

impl CreateMetadataAccountArgsLayout {
    pub fn to_proto_struct(&self) -> PbCreateMetadataAccountArgsLayout {
        PbCreateMetadataAccountArgsLayout {
            data: Some(self.data.to_proto_struct()),
            is_mutable: self.isMutable,
        }
    }
}


#[derive(BorshDeserialize, Debug, Default)]
pub struct DataV2Layout {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub sellerFeeBasisPoints: u16,
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
            seller_fee_basis_points: self.sellerFeeBasisPoints as u32,
            creators: creators,
            collection: collection,
            uses: uses,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CreateMetadataAccountArgsV2Layout {
    pub data: DataV2Layout,
    pub isMutable: bool,
}

impl CreateMetadataAccountArgsV2Layout {
    pub fn to_proto_struct(&self) -> PbCreateMetadataAccountArgsV2Layout {
        PbCreateMetadataAccountArgsV2Layout {
            data: Some(self.data.to_proto_struct()),
            is_mutable: self.isMutable,
        }
    }
}

#[derive(BorshDeserialize, Debug, Default)]
pub struct CreateMetadataAccountArgsV3Layout {
    pub data: DataV2Layout,
    pub isMutable: bool,
    pub collectionDetails: Option<CollectionDetailsLayout>,
}

impl CreateMetadataAccountArgsV3Layout {
    pub fn to_proto_struct(&self) -> PbCreateMetadataAccountArgsV3Layout {
        let mut collection_details: Option<PbCollectionDetailsLayout> = None;
        if self.collectionDetails.is_some() {
            collection_details = Some(self.collectionDetails.as_ref().unwrap().to_proto_struct());
        }

        PbCreateMetadataAccountArgsV3Layout {
            data: Some(self.data.to_proto_struct()),
            is_mutable: self.isMutable,
            collection_details: collection_details,
        }
    }
}

fn get_b58_string(data: [u8; 32]) -> String {
    return bs58::encode(data).into_string();
}
