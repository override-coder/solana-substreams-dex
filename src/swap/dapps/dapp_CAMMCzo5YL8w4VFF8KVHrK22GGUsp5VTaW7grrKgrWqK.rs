use substreams_solana::pb::sf::solana::r#type::v1::{ TokenBalance};
use crate::constants::{ RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS};
use crate::swap::trade_instruction::TradeInstruction;

const SWAP_DISCRIMINATOR: u64 = u64::from_le_bytes([248, 198, 158, 145, 225, 117, 135, 200]);
const SWAP_V2_DISCRIMINATOR: u64 = u64::from_le_bytes([43, 4, 237, 11, 26, 201, 30, 98]);

pub fn parse_trade_instruction(
    bytes_stream: &Vec<u8>,
    accounts: Vec<String>,
) -> Option<TradeInstruction> {
    let (disc_bytes, rest) = bytes_stream.split_at(8);
    let disc_bytes_arr: [u8; 8] = disc_bytes.to_vec().try_into().unwrap();
    let discriminator: u64 = u64::from_le_bytes(disc_bytes_arr);

    let mut result = None;

    match discriminator {
        SWAP_DISCRIMINATOR => {
            result = Some(TradeInstruction {
                program: String::from(RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS),
                name: String::from("Swap"),
                amm: accounts.get(2).unwrap().to_string(),
                vault_a: accounts.get(5).unwrap().to_string(),
                vault_b: accounts.get(6).unwrap().to_string(),
                ..Default::default()
            });
        }
        SWAP_V2_DISCRIMINATOR => {
            result = Some(TradeInstruction {
                program: String::from(RAYDIUM_CONCENTRATED_CAMM_PROGRAM_ADDRESS),
                name: String::from("SwapV2"),
                amm: accounts.get(2).unwrap().to_string(),
                vault_a: accounts.get(5).unwrap().to_string(),
                vault_b: accounts.get(6).unwrap().to_string(),
                ..Default::default()
            });
        }
        _ => {}
    }

    return result;
}


pub fn parse_reserves_instruction(
    amm: &String,
    accounts: &Vec<String>,
    token_balances: &Vec<TokenBalance>,
    vault_a: &String,
    vault_b: &String,
    token0: &String,
    token1: &String,
) -> (u64, u64) {
    let index_a = accounts.iter().position(|r| r == vault_a).unwrap_or_else(|| {
        panic!("Vault A not found in accounts");
    });
    let index_b = accounts.iter().position(|r| r == vault_b).unwrap_or_else(|| {
        panic!("Vault B not found in accounts");
    });

    let mut reserves0: u64 = 0;
    let mut reserves1: u64 = 0;

    token_balances
        .iter()
        .filter(|token_balance| {
            token_balance.account_index == index_a as u32
                || token_balance.account_index == index_b as u32
        })
        .filter(|token_balance| token_balance.owner == amm.clone()) // 仅处理匹配 amm 的记录
        .for_each(|token_balance| {
            if let Some(ref ui_token_amount) = token_balance.ui_token_amount {
                if token_balance.mint == token0.clone() {
                    reserves0 = ui_token_amount.amount.parse::<u64>().unwrap_or(0);
                } else if token_balance.mint == token1.clone() {
                    reserves1 = ui_token_amount.amount.parse::<u64>().unwrap_or(0);
                }
            }
        });
    (reserves0, reserves1)
}
