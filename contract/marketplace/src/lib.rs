use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, is_promise_success, promise_result_as_success, near_bindgen, log, AccountId, Gas, Promise, PanicOnDefault
};
use near_sdk::json_types::{U128};

pub mod external;
pub use crate::external::*;

pub const ACTIVITY_FT_EXCHANGE_RATE: u128 = 100;
pub const ACTIVITY_FT_STORAGE_DEPOSIT_YOCTO: u128 = 1_250_000_000_000_000_000_000;


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
        let near_amount = env::attached_deposit();
        if near_amount < ACTIVITY_FT_STORAGE_DEPOSIT_YOCTO {
            env::panic_str("Attached deposit must be greater than 0.00125 NEAR");
        };

        let buyer_id = env::predecessor_account_id();
        let ft_amount = near_amount.saturating_mul(ACTIVITY_FT_EXCHANGE_RATE);
        self.purchase_activity_ft(buyer_id, near_amount, ft_amount)
    }

    fn purchase_activity_ft(&mut self, buyer_id: AccountId, near_amount: u128, ft_amount: u128) -> Promise {
        ext_ft::ext(self.activity_ft.clone())
            .with_static_gas(Gas(5*TGAS))
            .with_attached_deposit(ACTIVITY_FT_STORAGE_DEPOSIT_YOCTO)
            .mint(buyer_id.clone(), U128::from(ft_amount))
                .then(
                    ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5*TGAS))
                    .on_activity_ft_purchased(buyer_id, near_amount, ft_amount)
                )
    }

    #[private]
    pub fn on_activity_ft_purchased(&mut self, buyer_id: AccountId, near_amount: u128, ft_amount: u128) -> u128 {
        if is_promise_success() {
            let result = promise_result_as_success().expect("Unexpected promise result");
            let charge_storage_fee : bool = near_sdk::serde_json::from_slice::<bool>(&result).ok().expect("Unexpected value result from promise");

            if charge_storage_fee {
                let decreased_near_amount = near_amount.saturating_sub(ACTIVITY_FT_STORAGE_DEPOSIT_YOCTO);
                Promise::new(self.activity_ft_beneficiary.clone()).transfer(u128::from(decreased_near_amount));

                let ft_amount_to_burn = ACTIVITY_FT_STORAGE_DEPOSIT_YOCTO.saturating_mul(ACTIVITY_FT_EXCHANGE_RATE);
                ext_ft::ext(self.activity_ft.clone())
                    .with_static_gas(Gas(5*TGAS))
                    .with_attached_deposit(1)
                    .burn(buyer_id, U128(ft_amount_to_burn));

                ft_amount - ft_amount_to_burn

            } else {
                Promise::new(self.activity_ft_beneficiary.clone()).transfer(u128::from(near_amount));
                ft_amount
            }
        } else {
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