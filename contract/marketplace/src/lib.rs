use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, is_promise_success, promise_result_as_success, near_bindgen, log, AccountId, Gas, Promise, PanicOnDefault};
use near_sdk::json_types::{U128};
use near_contract_standards::non_fungible_token::{Token};

pub mod external;
pub use crate::external::*;

pub const ACTIVITY_FT_EXCHANGE_RATE: u128 = 100;
pub const ACTIVITY_FT_REGISTRATION_FEE: u128 = 1_250_000_000_000_000_000_000;

pub const ARTFANS_NFT_PRICE: u128 = 3_500_000_000_000_000_000_000_000;
pub const ARTFANS_NFT_REGISTRATION_FEE: u128 = 100_000_000_000_000_000_000_000;


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
        if near_amount < ACTIVITY_FT_REGISTRATION_FEE {
            env::panic_str("Attached deposit must be greater than 0.00125 NEAR");
        };

        let buyer_id = env::predecessor_account_id();
        let ft_amount = near_amount.saturating_mul(ACTIVITY_FT_EXCHANGE_RATE);
        let ft_registration_fee = ACTIVITY_FT_REGISTRATION_FEE.saturating_mul(ACTIVITY_FT_EXCHANGE_RATE);
        self.purchase_activity_ft(buyer_id, ft_amount, ft_registration_fee)
    }

    fn purchase_activity_ft(&mut self, buyer_id: AccountId, ft_amount: u128, ft_registration_fee: u128) -> Promise {
        ext_ft::ext(self.activity_ft.clone())
            .with_static_gas(Gas(5*TGAS))
            .with_attached_deposit(ACTIVITY_FT_REGISTRATION_FEE)
            .mint(buyer_id.clone(), U128::from(ft_amount), Some(U128::from(ft_registration_fee)))
                .then(
                    ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5*TGAS))
                    .on_activity_ft_purchased(buyer_id, ft_amount)
                )
    }

    #[private]
    pub fn on_activity_ft_purchased(&mut self, buyer_id: AccountId, ft_amount: u128) -> U128 {
        let near_amount = ft_amount.saturating_div(ACTIVITY_FT_EXCHANGE_RATE);

        if is_promise_success() {
            let result = promise_result_as_success().expect("Unexpected promise result");
            let minted_ft_amount = u128::from(near_sdk::serde_json::from_slice::<U128>(&result).ok().expect("Unexpected value result from promise"));

            if minted_ft_amount == ft_amount {
                Promise::new(self.activity_ft_beneficiary.clone()).transfer(near_amount);
            } else {
                let ft_registration_fee = ACTIVITY_FT_REGISTRATION_FEE.saturating_mul(ACTIVITY_FT_EXCHANGE_RATE);
                assert_eq!(ft_amount.saturating_sub(minted_ft_amount), ft_registration_fee, "Unexpected amount of minted tokens");
                let near_registration_fee = ft_registration_fee.saturating_div(ACTIVITY_FT_EXCHANGE_RATE);
                let amount = near_amount - near_registration_fee;
                Promise::new(self.activity_ft_beneficiary.clone()).transfer(amount);
            };
            U128(minted_ft_amount)
        } else {
            Promise::new(buyer_id.clone()).transfer(near_amount);
            U128(0)
        }
    }

    #[payable]
    pub fn mint_artfans_nft(&mut self) -> Promise {
        let near_amount = env::attached_deposit();
        if near_amount != ARTFANS_NFT_PRICE {
            env::panic_str("Attached deposit must be equal to 3.5 NEAR");
        };

        let buyer_id = env::predecessor_account_id();
        self.purchase_artfans_nft(buyer_id)
    }
    
    fn purchase_artfans_nft(&mut self, buyer_id: AccountId) -> Promise {
        ext_nft::ext(self.artfans_nft.clone())
            .with_static_gas(Gas(5*TGAS))
            .with_attached_deposit(ARTFANS_NFT_REGISTRATION_FEE)
            .nft_mint(buyer_id.clone(), None)
                .then(
                    ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5*TGAS))
                    .on_artfans_nft_purchased(buyer_id)
                )
    }

    #[private]
    pub fn on_artfans_nft_purchased(&mut self, buyer_id: AccountId) -> Option<Token> {
        let near_amount = ARTFANS_NFT_PRICE - ARTFANS_NFT_REGISTRATION_FEE;
        
        if is_promise_success() {
            let result = promise_result_as_success().expect("Unexpected promise result");
            let token = near_sdk::serde_json::from_slice::<Token>(&result).ok().expect("Unexpected value result from promise");
            Promise::new(self.artfans_nft_beneficiary.clone()).transfer(near_amount);
            Some(token)
        } else {
            Promise::new(buyer_id.clone()).transfer(near_amount);
            None
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