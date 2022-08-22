use near_sdk::{ext_contract, AccountId, Promise};
use near_sdk::json_types::{U128};

pub const TGAS: u64 = 1_000_000_000_000;

#[ext_contract(ext_ft)]
trait FungibleToken {
    fn mint(&mut self, account_id: AccountId, amount: U128, registration_fee: Option<U128>) -> U128;
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn on_activity_ft_purchased(&mut self, buyer_id: AccountId, ft_amount: u128) -> Promise;
}