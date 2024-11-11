use substreams_database_change::tables::Tables;
use crate::pb::sf::solana::dex::trades::v1::{Output, TradeData};

pub fn created_trande_database_changes(tables: &mut Tables, trade: &Output) {
    for (index, t) in trade.data.iter().enumerate() {
        create_trade(tables, t,index as u32);
    }
}

fn create_trade(tables: &mut Tables, data: &TradeData, index: u32) {
    tables
        .create_row("trade", format!("{}-{}", &data.tx_id,index))
        .set("blockSlot", data.block_slot)
        .set("blockSlot", data.block_slot)
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
        .set("price",&data.price)
        .set("wsolPrice",&data.wsol_price)
        .set("amountUsd",&data.amount_usd)
        .set("isInnerInstruction",data.is_inner_instruction)
        .set("instructionIndex",data.instruction_index)
        .set("instructionType",&data.instruction_type)
        .set("innerInstruxtionIndex",data.inner_instruxtion_index)
        .set("outerProgram",&data.outer_program)
        .set("innerProgram",&data.inner_program)
        .set("txnFeeLamports",data.txn_fee_lamports);
}