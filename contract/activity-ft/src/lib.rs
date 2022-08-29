use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupSet};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner: AccountId,
    fee_collectors: LookupSet<AccountId>
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            owner: owner.clone(),
            fee_collectors: LookupSet::new(b"f".to_vec())
        };
        this.token.internal_register_account(&owner);
        this.token.internal_deposit(&owner, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let metadata = FungibleTokenMetadata {
          spec: "ft-1.0.0".to_string(),
          name: "Test Token".to_string(),
          symbol: "TST".to_string(),
          icon: None,
          reference: None,
          reference_hash: None,
          decimals: 0,
        };
        let contract = Contract::new(accounts(1).into(), TOTAL_SUPPLY.into(), metadata);
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let metadata = FungibleTokenMetadata {
          spec: "ft-1.0.0".to_string(),
          name: "Test Token".to_string(),
          symbol: "TST".to_string(),
          icon: None,
          reference: None,
          reference_hash: None,
          decimals: 0,
        };
        let mut contract = Contract::new(accounts(2).into(), TOTAL_SUPPLY.into(), metadata);
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}