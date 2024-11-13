use std::collections::HashMap;
use substreams::prelude::StoreGetFloat64;
use substreams::store::StoreGet;
use substreams_database_change::tables::Tables;
use crate::constants::PUMP_FUN_TOKEN_MINT_AUTHORITY_ADDRESS;
use crate::pb::sf::solana::dex::meta::v1::{TokenMetadataMeta, TokenMetas};
use crate::pb::sf::solana::dex::spl::v1::{SplTokenMeta, SplTokens};
use crate::pb::sf::solana::dex::trades::v1::{Swaps, TradeData};
use crate::utils::{calculate_price_and_amount_usd, WSOL_ADDRESS};

#[derive(Debug)]
pub struct Token {
    tx_id: String,
    address: String,
    name: String,
    symbol: String,
    decimals: i32,
    total_supply: String,
    is_pump_fun: bool,
}

pub fn created_trade_database_changes(tables: &mut Tables, trade: &Swaps, store: &StoreGetFloat64) {
    let wsol_price = store.get_last(WSOL_ADDRESS);
    for (index, t) in trade.data.iter().enumerate() {
        create_trade(tables, t, index as u32, wsol_price);
    }
}

fn create_trade(tables: &mut Tables, data: &TradeData, index: u32, wsol_price_option: Option<f64>) {
    let (token_price, amount_usdt, wsol_price) = match wsol_price_option {
        Some(wsol_price) => {
            calculate_price_and_amount_usd(
                &data.base_mint,
                &data.quote_mint,
                data.base_amount,
                data.quote_amount,
                data.base_decimals,
                data.quote_decimals,
                wsol_price.abs(),
            )
        }
        None => (0.0, 0.0, 0.0),
    };
    tables.create_row("trade", format!("{}-{}", &data.tx_id, index))
        .set("blockSlot", data.block_slot)
        .set("blockTime", data.block_time)
        .set("txId", &data.tx_id)
        .set("signer", &data.signer)
        .set("poolAddress", &data.pool_address)
        .set("baseMint", &data.base_mint)
        .set("quoteMint", &data.quote_mint)
        .set("baseVault", &data.base_vault)
        .set("quoteVault", &data.quote_vault)
        .set("baseAmount", data.base_amount)
        .set("quoteAmount", data.quote_amount)
        .set("baseDecimals", data.base_decimals)
        .set("quoteDecimals", data.quote_decimals)
        .set("price", token_price.to_string())
        .set("wsolPrice", wsol_price.to_string())
        .set("amountUSD", amount_usdt.to_string())
        .set("isInnerInstruction", data.is_inner_instruction)
        .set("instructionIndex", data.instruction_index)
        .set("instructionType", &data.instruction_type)
        .set("innerInstruxtionIndex", data.inner_instruxtion_index)
        .set("outerProgram", &data.outer_program)
        .set("innerProgram", &data.inner_program)
        .set("txnFeeLamports", data.txn_fee_lamports);
}

pub(crate) fn create_token_database_changes(tables: &mut Tables, tokens: &SplTokens, metas: &TokenMetas) {
    let mut meta_map: HashMap<String, TokenMetadataMeta> = HashMap::new();
    for token_meta in &metas.data {
        meta_map.insert(token_meta.tx_id.clone(), token_meta.clone());
    }
    let mut token_map: HashMap<String, Token> = HashMap::new();
    for (_, t) in tokens.data.iter().enumerate() {
        if t.instruction_type == "InitializeMint2" || t.instruction_type == "InitializeMint" {
            let meta = meta_map.get(&t.tx_id);
            parse_token_meta(t.clone(), meta, &mut token_map);
        }
        if t.instruction_type == "MintTo" {
            if let Some(account) = &t.input_accounts {
                if let Some(m) = &account.mint{
                    if let Some(token) = token_map.get_mut(m) {
                        if let Some(arg) = &t.args {
                            if let Some(a) = arg.amount {
                                token.total_supply = a.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    for (_, value) in &token_map {
        create_token(tables,value);
    }
}

fn create_token(tables: &mut Tables, token: &Token) {
    tables.create_row("token", &token.address)
        .set("txId", &token.tx_id)
        .set("address", &token.address)
        .set("name", &token.name)
        .set("symbol", &token.symbol)
        .set("decimals", token.decimals)
        .set("total_supply", &token.total_supply)
        .set("is_pump_fun", token.is_pump_fun);
}

fn parse_token_meta(token: SplTokenMeta, meta_option: Option<&TokenMetadataMeta>, token_map: &mut HashMap<String, Token>)  {
    if token.input_accounts.is_none(){
        return;
    }
    let account = token.input_accounts.unwrap();
    if account.mint.is_none() {
        return;
    }
    if token.args.is_none() {
        return;
    }
    let arg = token.args.unwrap();
    let mut t = Token{
        tx_id: token.tx_id.clone(),
        address: account.mint.unwrap().to_string(),
        name: "".to_string(),
        symbol: "".to_string(),
        decimals: arg.decimals().clone(),
        total_supply: "".to_string(),
        is_pump_fun: arg.mint_authority.as_ref().unwrap().to_string() == PUMP_FUN_TOKEN_MINT_AUTHORITY_ADDRESS.to_string(),
    };
    if let Some(meta) = meta_option{
        if let Some(arg) = &meta.args {
            if meta.instruction_type == "CreateMetadataAccount" {
                if let Some(m) = &arg.create_metadata_account_args {
                    if let Some(d) = &m.data{
                        t.name = d.name.clone();
                        t.symbol = d.symbol.clone();
                    }
                }
            }
            if meta.instruction_type == "CreateMetadataAccountV2" {
                if let Some(m) = &arg.create_metadata_account_args_v2 {
                    if let Some(d) = &m.data{
                        t.name = d.name.clone();
                        t.symbol = d.symbol.clone();
                    }
                }
            }
            if meta.instruction_type == "CreateMetadataAccountV3" {
                if let Some(m) = &arg.create_metadata_account_args_v3 {
                    if let Some(d) = &m.data{
                        t.name = d.name.clone();
                        t.symbol = d.symbol.clone();
                    }
                }
            }
        }
    }
    token_map.insert(t.address.clone(), t);
}
