use std::collections::HashMap;
use substreams::prelude::StoreGetFloat64;
use substreams::store::StoreGet;
use substreams_database_change::tables::Tables;
use crate::constants::{PUMP_FUN_TOKEN_MINT_AUTHORITY_ADDRESS, TOKEN_PROGRAM_ADDRESS};
use crate::pb::sf::solana::dex::jupiter_aggregator::v1::{JupiterSwaps, JupiterTrade};
use crate::pb::sf::solana::dex::meta::v1::{TokenMetadataMeta, TokenMetas};
use crate::pb::sf::solana::dex::spl::v1::{Accounts, Arg, SplTokenMeta, SplTokens};
use crate::pb::sf::solana::dex::trades::v1::{Pool, Pools, Swaps, TradeData};
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
        if t.base_amount == 0 && t.quote_amount == 0 {
            continue
        }
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
        .set("baseReserves", data.base_reserves)
        .set("quoteReserves", data.quote_reserves)
        .set("price", token_price.to_string())
        .set("wsolPrice", wsol_price.to_string())
        .set("amountUSD", amount_usdt.to_string())
        .set("isInnerInstruction", data.is_inner_instruction)
        .set("instructionIndex", data.instruction_index)
        .set("instruction_type", &data.instruction_type)
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
    for (index, t) in tokens.data.iter().enumerate() {
        if t.instruction_type == "InitializeMint2" || t.instruction_type == "InitializeMint" {
            let meta = meta_map.get(&t.tx_id);
            parse_token_meta(t.clone(), meta, &mut token_map);
        }
        if t.instruction_type == "MintTo" || t.instruction_type == "MintToChecked" {
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
        if t.instruction_type == "Transfer" || t.instruction_type == "TransferChecked" {
        //   if t.outer_program == TOKEN_PROGRAM_ADDRESS.to_string() {
                create_transfer(tables, t,index)
          //  }

        }
    }
    for (_, value) in &token_map {
        create_token(tables,value);
    }
}

fn create_transfer(tables: &mut Tables, token: &SplTokenMeta, index: usize) {
    let (mint,signer,source,destination) = match &token.input_accounts  {
        Some(account) => {
            (account.mint.clone().unwrap_or("".to_string()),
             account.owner.clone().unwrap_or("".to_string()),
             account.source.clone().unwrap_or("".to_string()),
             account.destination.clone().unwrap_or("".to_string()),
            )
        }
        None => ("".to_string(),"".to_string(),"".to_string(),"".to_string()),
    };
    let (amount , decimals) = match &token.args {
        Some(arg) => {
            (arg.amount.unwrap_or(0),arg.decimals.unwrap_or(9))
        }
        None => {
            (0,0)
        }
    };
    tables.create_row("transfer", format!("{}-{}", &token.tx_id, index))
        .set("blockSlot", token.block_slot)
        .set("blockTime", token.block_time)
        .set("txId", &token.tx_id)
        .set("token",mint)
        .set("signer",signer)
        .set("source",source)
        .set("destination",destination)
        .set("amount",amount.to_string())
        .set("decimals",decimals.to_string());
}

fn create_token(tables: &mut Tables, token: &Token) {
    tables.create_row("token", &token.address)
        .set("txId", &token.tx_id)
        .set("address", &token.address)
        .set("name", &token.name)
        .set("symbol", &token.symbol)
        .set("decimals", token.decimals)
        .set("totalSupply", &token.total_supply)
        .set("isPumpFun", token.is_pump_fun);
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

pub(crate) fn create_pool_database_changes(tables: &mut Tables, pools: &Pools) {
    for (_, t) in pools.pools.iter().enumerate() {
        create_pool(tables, t);
    }
}

fn create_pool(tables: &mut Tables, pool: &Pool) {
    tables.create_row("pool",  &pool.address)
        .set("createBlockSlot", pool.created_at_block_number)
        .set("createBlockTime", pool.created_at_timestamp)
        .set("txId", &pool.tx_id)
        .set("poolAddress", &pool.address)
        .set("program", &pool.program)
        .set("coinMint", &pool.coin_mint)
        .set("pcMint", &pool.pc_mint)
        .set("isPumpFun", pool.is_pump_fun);
}

pub(crate) fn create_jupiter_swap_database_changes(tables: &mut Tables, swaps: &JupiterSwaps, store: &StoreGetFloat64) {
    let wsol_price = store.get_last(WSOL_ADDRESS);
    for (index, t) in swaps.data.iter().enumerate() {
        create_jupiter_trade(tables, t,index as u32,wsol_price);
    }
}

fn create_jupiter_trade(tables: &mut Tables,j: &JupiterTrade,index:u32, wsol_price_option: Option<f64>) {
    let (token_price, amount_usdt, wsol_price) = match wsol_price_option {
        Some(wsol_price) => {
            calculate_price_and_amount_usd(
                &j.source_mint,
                &j.destination_mint,
                j.in_amount.parse().unwrap_or(0),
                j.quoted_out_amount.parse().unwrap_or(0),
                j.in_decimals,
                j.quoted_decimals,
                wsol_price.abs(),
            )
        }
        None => (0.0, 0.0, 0.0),
    };
    tables.create_row("jupiter", format!("{}-{}", &j.tx_id, index))
        .set("blockSlot", j.block_slot)
        .set("blockTime", j.block_time)
        .set("txId", &j.tx_id)
        .set("signer", &j.signer)
        .set("sourceTokenAccount", &j.source_token_account)
        .set("destinationTokenAccount", &j.destination_token_account)
        .set("sourceMint", &j.source_mint)
        .set("destinationMint", &j.destination_mint)
        .set("inAmount", &j.in_amount)
        .set("quotedOutAmount",&j.quoted_out_amount)
        .set("baseDecimals", j.in_decimals)
        .set("quoteDecimals", j.quoted_decimals)
        .set("price", token_price.to_string())
        .set("wsolPrice", wsol_price.to_string())
        .set("amountUSD", amount_usdt.to_string())
        .set("instructionType", &j.instruction_type);
}