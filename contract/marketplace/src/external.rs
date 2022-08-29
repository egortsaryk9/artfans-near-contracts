use near_sdk::{ext_contract, AccountId, Promise};
use near_sdk::json_types::{U128};

pub const TGAS: u64 = 1_000_000_000_000;

#[ext_contract(ext_ft)]
trait FungibleToken {
    // fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn mint(&mut self, account_id: AccountId, ft_amount: U128) -> bool;
    fn burn(&mut self, account_id: AccountId, ft_amount: U128);
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn on_activity_ft_purchased(&mut self, buyer_id: AccountId, near_amount: u128, ft_amount: u128) -> Promise;
}