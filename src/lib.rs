mod pb;
mod trade_instruction;
mod dapps;
mod utils;
mod db;
mod constants;
mod spl_token;
mod spl_token_meta;
mod prepare_input_accounts;
mod prepare_arg;
mod instructions;
mod swap;


use std::io::Error;
use substreams::prelude::*;
use substreams::store::StoreGetFloat64;
use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::solana::dex::meta::v1::TokenMetas;
use crate::pb::sf::solana::dex::spl::v1::SplTokens;
use crate::pb::sf::solana::dex::trades::v1::{Pools, Swaps};


#[substreams::handlers::map]
pub fn slink_database_out(pools: Pools,tokens: SplTokens, token_metas: TokenMetas ,swaps: Swaps,store: StoreGetFloat64) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();

    db::created_trade_database_changes(&mut tables, &swaps, &store);
    db::create_token_database_changes(&mut tables, &tokens, &token_metas);

    return Ok(tables.to_database_changes())
}
