use crate::pb::sf::solana::dex::meta::v1::InputAccounts;

pub fn prepare_input_accounts(
    instruction_type: String,
    account_indices: &Vec<u8>,
    accounts: &Vec<String>,
) -> Option<InputAccounts> {
    let input_accounts = populate_input_accounts(account_indices, accounts);

    let mut result = InputAccounts::default();
    match instruction_type.as_str() {
        "CreateMetadataAccount" => {
            result.metadata = get_account_with(&input_accounts, 0);
            result.mint = get_account_with(&input_accounts, 1);
            result.mint_authority = get_account_with(&input_accounts, 2);
            result.payer = get_account_with(&input_accounts, 3);
            result.update_authority = get_account_with(&input_accounts, 4);
            result.system_program = get_account_with(&input_accounts, 5);
            result.rent = get_account_with(&input_accounts, 6);
        }

        "CreateMetadataAccountV2" => {
            result.metadata = get_account_with(&input_accounts, 0);
            result.mint = get_account_with(&input_accounts, 1);
            result.mint_authority = get_account_with(&input_accounts, 2);
            result.payer = get_account_with(&input_accounts, 3);
            result.update_authority = get_account_with(&input_accounts, 4);
            result.system_program = get_account_with(&input_accounts, 5);
            result.rent = get_account_with(&input_accounts, 6);
        }
        "CreateMetadataAccountV3" => {
            result.metadata = get_account_with(&input_accounts, 0);
            result.mint = get_account_with(&input_accounts, 1);
            result.mint_authority = get_account_with(&input_accounts, 2);
            result.payer = get_account_with(&input_accounts, 3);
            result.update_authority = get_account_with(&input_accounts, 4);
            result.system_program = get_account_with(&input_accounts, 5);
            result.rent = get_account_with(&input_accounts, 6);
        }
        _ => {}
    }

    return Some(result);
}

fn get_account_with(accounts: &Vec<String>, index: usize) -> Option<String> {
    let mut result: Option<String> = None;
    let account = accounts.get(index);
    if account.is_some() {
        result = Some(account.unwrap().to_string());
    }
    return result;
}

fn populate_input_accounts(account_indices: &Vec<u8>, accounts: &Vec<String>) -> Vec<String> {
    let mut instruction_accounts: Vec<String> = vec![];
    for (index, &el) in account_indices.iter().enumerate() {
        instruction_accounts.push(accounts.as_slice()[el as usize].to_string());
    }
    return instruction_accounts;
}
