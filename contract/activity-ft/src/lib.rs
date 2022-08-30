use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupSet};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, assert_one_yocto, AccountId, Balance, PanicOnDefault, PromiseOrValue, BorshStorageKey};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner: AccountId,
    fee_collectors: LookupSet<AccountId>,
    minters: LookupSet<AccountId>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Token,
    Metadata,
    FeeCollectors,
    Minters
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner: AccountId,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(StorageKeys::Token),
            metadata: LazyOption::new(StorageKeys::Metadata, Some(&metadata)),
            owner: owner.clone(),
            fee_collectors: LookupSet::new(StorageKeys::FeeCollectors),
            minters: LookupSet::new(StorageKeys::Minters),
        };
        this.token.internal_register_account(&owner);
        this.minters.insert(&owner);
        this
    }

    pub fn ft_collect_fee(&mut self, amount: U128) {
        assert!(self.fee_collectors.contains(&env::predecessor_account_id()), "Only registered fee collectors can collect fees in this token");
        if !self.token.accounts.contains_key(&env::predecessor_account_id()) {
            self.token.accounts.insert(&env::predecessor_account_id(), &0);
        }
        let amount: Balance = amount.into();
        self.token.internal_transfer(&env::signer_account_id(), &env::predecessor_account_id(), amount, None);
    }

    pub fn add_fee_collector(&mut self, account_id: AccountId) {
        self.assert_owner();
        if !self.fee_collectors.insert(&account_id) {
            env::panic_str("The account is already registered as a fee collector");
        }
    }
    
    pub fn remove_fee_collector(&mut self, account_id: AccountId) {
        self.assert_owner();
        if !self.fee_collectors.remove(&account_id) {
            env::panic_str("The account is not registered as a fee collector");
        }
    }

    pub fn add_minter(&mut self, account_id: AccountId) {
        self.assert_owner();
        if !self.minters.insert(&account_id) {
            env::panic_str("The account is already registered as a minter");
        }
    }

    pub fn remove_minter(&mut self, account_id: AccountId) {
        self.assert_owner();
        if !self.minters.remove(&account_id) {
            env::panic_str("The account is not registered as a minter");
        }
    }

    fn assert_minter(&self) {
        assert!(self.minters.contains(&env::predecessor_account_id()),
            "This operation is restricted to token minters"
        );
    }

    #[payable]
    pub fn mint(&mut self, account_id: AccountId, amount: U128, registration_fee: Option<U128>) -> U128 {
        self.assert_minter();
        let amount_to_mint: u128 = if self.token.accounts.contains_key(&account_id) {
            amount.into()
        } else {
            match registration_fee {
                Some(fee) => {
                    let total: u128 = amount.into();
                    let correction: u128 = fee.into();
                    if total < correction {
                        env::panic_str("Amount is not enough to cover the storage deposit fee");
                    };
                    total - correction
                },
                None => amount.into()
            }
        };
        self.storage_deposit(Some(account_id.clone()), None);
        self.token.internal_deposit(&account_id, amount_to_mint);
        U128(amount_to_mint)
    }

    #[payable]
    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        self.assert_owner();
        assert_one_yocto();
        self.token.internal_withdraw(&account_id, amount.into());
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

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}