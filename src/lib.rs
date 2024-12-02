mod pb;
mod utils;
mod db;
mod constants;
mod spl_token;
mod swap;
mod jupiter_aggregator;

use std::io::Error;
use substreams::prelude::*;
use substreams::store::StoreGetFloat64;
use substreams_database_change::pb::database::DatabaseChanges;
use crate::pb::sf::solana::dex::jupiter_aggregator::v1::JupiterSwaps;
use crate::pb::sf::solana::dex::meta::v1::TokenMetas;
use crate::pb::sf::solana::dex::spl::v1::SplTokens;
use crate::pb::sf::solana::dex::trades::v1::{Pools, Swaps};

#[substreams::handlers::map]
pub fn slink_database_out(pools: Pools, tokens: SplTokens, token_metas: TokenMetas, swaps: Swaps, jupiter_swaps: JupiterSwaps, store: StoreGetFloat64) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();

    db::create_token_database_changes(&mut tables, &tokens, &token_metas);
    db::create_pool_database_changes(&mut tables,&pools);
    db::create_jupiter_swap_database_changes(&mut tables, &jupiter_swaps,&store);
    db::created_trade_database_changes(&mut tables, &swaps, &store);

    return Ok(tables.to_database_changes())
}