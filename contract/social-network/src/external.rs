use near_sdk::{ext_contract};
use near_sdk::json_types::{U128};
use near_sdk::serde::{Deserialize, Serialize};

pub const TGAS: u64 = 1_000_000_000_000;
pub const FIXED_FEE: u128 = 10_000_000_000_000_000_000;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub 
enum ContractAction {
    AddMessage { post_id: String, text: String },
    AddFriend,
}

#[ext_contract(ext_ft)]
trait FungibleToken {
    fn ft_collect_fee(&mut self, amount: U128);
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn on_fee_collected(&mut self, action: ContractAction) -> String;
}