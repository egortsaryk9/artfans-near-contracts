use near_sdk::{ext_contract};
use near_sdk::json_types::{U128};
use crate::Call;


pub const TGAS: u64 = 1_000_000_000_000;
pub const FIXED_FEE: u128 = 1_000_000_000_000_000_000;


#[ext_contract(ext_ft)]
trait FungibleToken {
    fn ft_collect_fee(&mut self, amount: U128);
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn on_fee_collected(&mut self, call: Call) -> String;
}