use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, is_promise_success, near_bindgen, log, AccountId, Gas, Promise, PanicOnDefault
};
use near_sdk::json_types::{U128};

pub mod external;
pub use crate::external::*;

pub const EXCHANGE_RATE: u128 = 100;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    activity_ft: AccountId,
    activity_ft_beneficiary: AccountId,
    artfans_nft: AccountId,
    artfans_nft_beneficiary: AccountId
}


#[near_bindgen]
impl Contract {

    #[init]
    pub fn new(
        owner: AccountId, 
        activity_ft: AccountId, 
        activity_ft_beneficiary: AccountId, 
        artfans_nft: AccountId, 
        artfans_nft_beneficiary: AccountId
    ) -> Self {

        if env::state_exists() == true {
            env::panic_str("Already initialized");
        }

        Self {
            owner,
            activity_ft,
            activity_ft_beneficiary,
            artfans_nft,
            artfans_nft_beneficiary
        }
    }
    
    #[payable]
    pub fn buy_activity_ft(&mut self) -> Promise {
        let buyer_id = env::predecessor_account_id();
        let near_amount = env::attached_deposit();
        let activity_ft_amount = near_amount.saturating_mul(EXCHANGE_RATE);
        self.purchase_activity_ft(buyer_id,near_amount, activity_ft_amount)
    }

    fn purchase_activity_ft(&mut self, buyer_id: AccountId, near_amount: u128, activity_ft_amount: u128) -> Promise {
        ext_ft::ext(self.activity_ft.clone())
            .with_static_gas(Gas(5*TGAS))
            .ft_transfer(buyer_id.clone(), U128::from(activity_ft_amount), None)
                .then(
                    ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5*TGAS))
                    .on_activity_ft_purchased(buyer_id, near_amount, activity_ft_amount)
                )
    }

    #[private]
    pub fn on_activity_ft_purchased(&mut self, buyer_id: AccountId, near_amount: u128, activity_ft_amount: u128) -> u128 {
        if is_promise_success() {
            Promise::new(self.activity_ft_beneficiary.clone()).transfer(u128::from(near_amount));
            activity_ft_amount
        } else {
            // Refund
            Promise::new(buyer_id.clone()).transfer(u128::from(near_amount));
            0
        }
    }
}


pub trait Ownable {
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.get_owner(),
            "This operation is restricted to the contract owner."
        );
    }
    fn get_owner(&self) -> AccountId;
    fn set_owner(&mut self, owner: AccountId);
}

#[near_bindgen]
impl Ownable for Contract {
    fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    fn set_owner(&mut self, owner: AccountId) {
        self.assert_owner();
        self.owner = owner;
    }
}