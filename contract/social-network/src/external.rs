use near_sdk::{ext_contract, AccountId};
use near_sdk::json_types::{U128};
use crate::Call;
use crate::CallResult;

pub const TGAS: u64 = 1_000_000_000_000;
pub const FIXED_FEE: u128 = 1_000_000_000_000_000_000;


#[ext_contract(ext_ft)]
trait FungibleToken {
    fn ft_collect_fee(&mut self, amount: U128);
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn on_fee_collected(&mut self, call: Call) -> CallResult;
}