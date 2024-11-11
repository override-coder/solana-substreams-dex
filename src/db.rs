use substreams::prelude::StoreGetFloat64;
use substreams::store::StoreGet;
use substreams_database_change::tables::Tables;
use crate::pb::sf::solana::dex::trades::v1::{Output, TradeData};
use crate::utils::{calculate_price_and_amount_usd, WSOL_ADDRESS};

pub fn created_trande_database_changes(tables: &mut Tables, trade: &Output, store: &StoreGetFloat64) {
    let wsol_price = store.get_last( WSOL_ADDRESS);
    for (index, t) in trade.data.iter().enumerate() {
        create_trade(tables, t,index as u32,wsol_price);
    }
}

fn create_trade(tables: &mut Tables, data: &TradeData, index: u32,wsol_price_option: Option<f64>) {
    let (token_price, amount_usdt,wsol_price) = match wsol_price_option {
        Some(wsol_price) => { calculate_price_and_amount_usd(
                &data.base_mint,
                &data.quote_mint,
                data.base_amount,
                data.quote_amount,
                data.base_decimals,
                data.quote_decimals,
                wsol_price.abs(),
            )
        }
        None => (0.0, 0.0,0.0),
    };
    tables.create_row("trade", format!("{}-{}", &data.tx_id,index))
        .set("blockSlot", data.block_slot)
        .set("blockTime", data.block_time)
        .set("txId",&data.tx_id)
        .set("signer",&data.signer)
        .set("poolAddress",&data.pool_address)
        .set("baseMint",&data.base_mint)
        .set("quoteMint",&data.quote_mint)
        .set("baseVault",&data.base_vault)
        .set("quoteVault",&data.quote_vault)
        .set("baseAmount",data.base_amount)
        .set("quoteAmount",data.quote_amount)
        .set("baseDecimals",data.base_decimals)
        .set("quoteDecimals",data.quote_decimals)
        .set("price",token_price.to_string())
        .set("wsolPrice",wsol_price.to_string())
        .set("amountUSD",amount_usdt.to_string())
        .set("isInnerInstruction",data.is_inner_instruction)
        .set("instructionIndex",data.instruction_index)
        .set("instructionType",&data.instruction_type)
        .set("innerInstruxtionIndex",data.inner_instruxtion_index)
        .set("outerProgram",&data.outer_program)
        .set("innerProgram",&data.inner_program)
        .set("txnFeeLamports",data.txn_fee_lamports);
}