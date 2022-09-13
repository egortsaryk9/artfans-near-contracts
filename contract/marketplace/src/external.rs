use near_sdk::{ext_contract, AccountId, Promise};
use near_sdk::json_types::{U128};
use near_contract_standards::non_fungible_token::{Token};
// use near_contract_standards::non_fungible_token::metadata::{TokenMetadata};

pub const TGAS: u64 = 1_000_000_000_000;

#[ext_contract(ext_ft)]
trait FungibleToken {
    fn ft_mint(&mut self, account_id: AccountId, amount: U128, registration_fee: Option<U128>) -> U128;
}

// #[ext_contract(ext_nft)]
// trait NonFungibleToken {
//     fn nft_mint(&mut self, receiver_id: AccountId, metadata: Option<TokenMetadata>) -> Token;
// }

#[ext_contract(ext_self)]
trait ExtSelf {
    fn on_activity_ft_purchased(&mut self, buyer_id: AccountId, ft_amount: u128) -> Promise;
    fn on_artfans_nft_purchased(&mut self, buyer_id: AccountId) -> Option<Token>;
}