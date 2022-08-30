use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupSet};
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    default_token_metadata: LazyOption<TokenMetadata>,
    minters: LookupSet<AccountId>,
    token_metadata_admins: LookupSet<AccountId>
}


#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    DefaultTokenMetadata,
    Minters,
    TokenMetadataAdmins
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner: AccountId, contract_metadata: NFTContractMetadata, default_token_metadata: TokenMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        
        contract_metadata.assert_valid();
        default_token_metadata.assert_valid();

        let mut this = Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner.clone(),
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval)
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&contract_metadata)),
            default_token_metadata: LazyOption::new(StorageKey::DefaultTokenMetadata, Some(&default_token_metadata)),
            minters: LookupSet::new(StorageKey::Minters),
            token_metadata_admins: LookupSet::new(StorageKey::TokenMetadataAdmins)
        };
        this.minters.insert(&owner);
        this.token_metadata_admins.insert(&owner);
        this
    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId
    ) -> Token {
        self.assert_minter();
        if let Some(token_metadata) = self.default_token_metadata.get() {
            self.tokens.internal_mint(token_id, receiver_id, Some(token_metadata))
        } else {          
            env::panic_str("Default token metadata is missed");
        }
    }


    #[payable]
    pub fn set_token_metadata(
        &mut self,
        token_id: TokenId,
        token_metadata: TokenMetadata
    ) {
        self.assert_token_metadata_admin();
        if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
            token_metadata_by_id.insert(&token_id, &token_metadata);
        } else {
            env::panic_str("Token metadata is not set");
        };
    }


    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.tokens.owner_id,
            "This operation is restricted to token owner"
        );
    }

    fn assert_minter(&self) {
        assert!(self.minters.contains(&env::predecessor_account_id()),
            "This operation is restricted to token minters"
        );
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

    fn assert_token_metadata_admin(&self) {
        assert!(self.token_metadata_admins.contains(&env::predecessor_account_id()),
            "This operation is restricted to token token metadata admin"
        );
    }

    pub fn add_token_metadata_admin(&mut self, account_id: AccountId) {
        self.assert_owner();
        if !self.token_metadata_admins.insert(&account_id) {
            env::panic_str("The account is already registered as a token metadata admin");
        }
    }

    pub fn remove_token_metadata_admin(&mut self, account_id: AccountId) {
        self.assert_owner();
        if !self.token_metadata_admins.remove(&account_id) {
            env::panic_str("The account is not registered as a token metadata admin");
        }
    }

}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}