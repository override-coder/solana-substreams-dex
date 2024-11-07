#[derive(Debug)]
pub struct TradeInstruction {
    pub program: String,
    pub name: String,
    pub amm: String,
    pub vault_a: String,
    pub vault_b: String,
}

impl Default for TradeInstruction {
    fn default() -> Self {
        TradeInstruction {
            program: "".to_string(),
            name: "".to_string(),
            amm: "".to_string(),
            vault_a: "".to_string(),
            vault_b: "".to_string(),
        }
    }
}
