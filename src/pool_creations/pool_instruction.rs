
#[derive(Debug)]
pub struct CreatePoolInstruction {
    pub program: String,
    pub name: String,
    pub amm: String,
    pub coin_mint: String,
    pub pc_mint: String,
    pub is_pump_fun: bool,
    pub is_moonshot: bool,
}

impl Default for CreatePoolInstruction {
    fn default() -> Self {
        CreatePoolInstruction {
            program: "".to_string(),
            name: "".to_string(),
            amm: "".to_string(),
            coin_mint: "".to_string(),
            pc_mint: "".to_string(),
            is_pump_fun: false,
            is_moonshot: false,
        }
    }
}

