use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{Token, TokenId, NonFungibleToken};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupSet};
use near_sdk::{
    assert_one_yocto,
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use near_sdk::json_types::U128;
use std::collections::HashMap;


pub const NFT_MAX_SUPPLY: u128 = 26_000;
pub const NFT_PRICE: u128 = 3_500_000_000_000_000_000_000_000;
pub const NFT_REGISTRATION_FEE: u128 = 100_000_000_000_000_000_000_000;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    default_token_metadata: LazyOption<TokenMetadata>,
    token_metadata_admins: LookupSet<AccountId>,
    beneficiary: AccountId
}


#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
    DefaultTokenMetadata,
    TokenMetadataAdmins
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner: AccountId, 
        contract_metadata: NFTContractMetadata, 
        default_token_metadata: TokenMetadata,
        beneficiary: AccountId
    ) -> Self {
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
            token_metadata_admins: LookupSet::new(StorageKey::TokenMetadataAdmins),
            beneficiary
        };
        this.token_metadata_admins.insert(&owner);
        this
    }


    #[payable]
    pub fn nft_buy_mint_approve(&mut self, approve_receiver_id: Option<AccountId>, approve_msg: Option<String>) -> Token {
        
        if env::attached_deposit() != NFT_PRICE {
            env::panic_str("Attached deposit must be equal to 3.5 NEAR");
        };

        if approve_receiver_id.is_none() && approve_msg.is_some() {
            env::panic_str("'approve_receiver_id' must be specified for provided 'approve_msg'");
        };

        let buyer_id = env::predecessor_account_id();
        let total_supply: u128 = self.tokens.owner_by_id.len() as u128;
        if total_supply < NFT_MAX_SUPPLY {
            let token_id: TokenId = format!("{}", total_supply + 1);
            let token_metadata = self.default_token_metadata.get().expect("Default Token Metadata is not set");
            let token = self.tokens.internal_mint_with_refund(
                token_id.clone(), 
                buyer_id, 
                Some(token_metadata), 
                None
            );
            
            if let Some(account_id) = approve_receiver_id {
                self.tokens.nft_approve(token_id, account_id, approve_msg);
            };

            let near_amount = NFT_PRICE - NFT_REGISTRATION_FEE;
            Promise::new(self.beneficiary.clone()).transfer(near_amount); // send funds to beneficiary
            token
        } else {
            env::panic_str("Max Supply is reached");
        }
    }


    #[payable]
    pub fn nft_set_metadata(
        &mut self,
        token_id: TokenId,
        token_metadata: TokenMetadata
    ) {
        self.assert_token_metadata_admin();
        if self.tokens.owner_by_id.get(&token_id).is_none() {
            env::panic_str("Token id does not exist");
        };
        if let Some(token_metadata_by_id) = &mut self.tokens.token_metadata_by_id {
            token_metadata_by_id.insert(&token_id, &token_metadata);
        } else {
            env::panic_str("Token Metadata extension is not set");
        };
    }


    #[payable]
    pub fn set_default_token_metadata(
        &mut self,
        default_token_metadata: TokenMetadata
    ) {
        self.assert_token_metadata_admin();
        default_token_metadata.assert_valid();
        self.default_token_metadata.set(&default_token_metadata);
    }


    pub fn nft_payout(
        &self, 
        token_id: String,
        balance: U128, 
        max_len_payout: u32
    ) -> HashMap<AccountId, U128> {
        let owner_id = self.tokens.owner_by_id.get(&token_id).expect("Token id does not exist");
        let mut result: HashMap<AccountId, U128> = HashMap::new();
        result.insert(owner_id, balance);
        result
    }


    #[payable]
    pub fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: u64,
        balance: U128,
        max_len_payout: u32,
    ) -> HashMap<AccountId, U128> {
        assert_one_yocto();
        let owner_id = self.tokens.owner_by_id.get(&token_id).expect("Token id does not exist");
        self.tokens.nft_transfer(receiver_id, token_id, Some(approval_id), None);
        let mut result: HashMap<AccountId, U128> = HashMap::new();
        result.insert(owner_id, balance);
        result
    }
    
    fn assert_owner(&self) {
        assert_eq!(env::predecessor_account_id(), self.tokens.owner_id,
            "This operation is restricted to token owner"
        );
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