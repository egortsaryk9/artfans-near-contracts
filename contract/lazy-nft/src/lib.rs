use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::{Token, TokenId, NonFungibleToken};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{
    env, log, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, Balance
};

use bigint::U256;
use near_sdk::serde_json;
use eip_712::{EIP712, hash_structured_data};
use rustc_hex::ToHex;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    minter_pk: String,
    metadata: LazyOption<NFTContractMetadata>,
    pending_withdrawals: LookupMap<AccountId, Balance>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    ContractMetadata,
    TokenMetadata,
    Enumeration,
    Approval,
    PendingWithdrawals
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTVoucher {
    token_id: String, // U256
    min_price: String, // U256
    uri: String
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        minter_pk: String,
        metadata: NFTContractMetadata
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        
        metadata.assert_valid();

        let this = Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval)
            ),
            minter_pk,
            metadata: LazyOption::new(StorageKey::ContractMetadata, Some(&metadata)),
            pending_withdrawals: LookupMap::new(StorageKey::PendingWithdrawals),
        };
        this
    }

    
    #[payable]
    pub fn redeem(&mut self, redeemer_id: AccountId, voucher: NFTVoucher, signature: Vec<u8>) -> TokenId {
        let near_amount = env::attached_deposit();

        let minter_pk = self.verify(voucher, signature);
        assert!(minter_pk == self.minter_pk,
            "Voucher signature is invalid or unauthorized"
        );

        // assert!(near_amount >= voucher.min_price, "Insufficient funds to redeem");


        let total_supply: u128 = self.tokens.owner_by_id.len() as u128;
        let token_id: TokenId = format!("{}", total_supply + 1);
        // let token = self.tokens.internal_mint_with_refund(
        //     token_id.clone(), 
        //     redeemer_id, 
        //     None, // Some(token_metadata), 
        //     None
        // );

        // let mut pending_amount = self.pending_withdrawals.get(&minter_pk).unwrap_or_else(|| {
        //     Balance::from(0u128)
        // });
        // pending_amount += near_amount;
        // self.pending_withdrawals.insert(&minter_pk, &pending_amount);
    
        token_id
    }


    pub fn withdraw(&mut self) {
        let signer = env::signer_account_id();
        assert_eq!(signer, self.tokens.owner_id,
            "This operation is restricted to token owner/minter"
        );
        // signer_account_pk

        // minter_pk

        let minter = signer;
        let amount = self.pending_withdrawals.get(&minter).expect("There is no pending withdrawals for the sender");

        let zero_amount = Balance::from(0u128);
        assert!(amount > zero_amount,
            "There is no pending amount to withdraw"
        );

        // zero account before transfer to prevent re-entrancy attack
        self.pending_withdrawals.insert(&minter, &zero_amount);

        Promise::new(minter).transfer(amount);
    }



    // Verify a signature on a message. Returns true if the signature is good.
	  // Parses Signature using parse_overflowing_slice.
    // fn extract_pubkey(signature: [u8; 65], message: [u8; 32]) -> Option<libsecp256k1::PublicKey> {
    //     let message = libsecp256k1::Message::parse(&message);
    //     let sig = libsecp256k1::Signature::parse_overflowing_slice(&signature[..64]).ok()?;
    //     let rid = libsecp256k1::RecoveryId::parse(signature[64]).ok()?;
    //     match libsecp256k1::recover(&message, &sig, &rid) {
    //       Ok(pubkey) => Some(pubkey),
    //       _ => None,
    //     }
    // }

    fn get_digest(&mut self, voucher: NFTVoucher) -> [u8; 32] {
        let token_id = format!("{:#066x}", U256::from_dec_str(&voucher.token_id).expect("Invalid U256 type"));
        let min_price = format!("{:#066x}", U256::from_dec_str(&voucher.min_price).expect("Invalid U256 type"));

        // let minPrice = U256::from_dec_str("1000000000000000000").unwrap();
        // ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi

        let json = format!(r#"{{
            "primaryType": "NFTVoucher",
            "domain": {{
              "name": "LazyNFT-Voucher",
              "version": "1",
              "chainId": "0x5",
              "verifyingContract": "0x7f0e636d67f6ec8d538484ae5d1a1fed8d7a1ab7"
            }},
            "message": {{
              "tokenId": "{}",
              "minPrice": "{}",
              "uri": "{}"
            }},
            "types": {{
              "EIP712Domain": [
                {{"name": "name", "type": "string"}},
                {{"name": "version", "type": "string"}},
                {{"name": "chainId", "type": "uint256"}},
                {{"name": "verifyingContract", "type": "address"}}
              ],
              "NFTVoucher": [
                {{"name": "tokenId", "type": "uint256"}},
                {{"name": "minPrice", "type": "uint256"}},
                {{"name": "uri", "type": "string"}}
              ]
            }}
          }}"#, token_id, min_price, voucher.uri);

        // log!("json {}", json.clone());

        let typed_data = serde_json::from_str::<EIP712>(&json).unwrap();
        let digets: [u8; 32] = hash_structured_data(typed_data).unwrap().into(); // to_fixed_bytes
        digets

        // let extracted_hash = hash_structured_data(typed_data).unwrap().to_hex::<String>();
        // log!("extracted_hash {}", extracted_hash);
        // assert_eq!(
        //     extracted_hash,
        //     "7670bde17285be885b5e7849491232bbdef3ccca053ea2d4c1e73ea7820626cc"
        // );
    }

    fn verify(&mut self, voucher: NFTVoucher, signature: Vec<u8>) -> String /* [u8; 65] */ /* libsecp256k1::PublicKey */ {
        assert_eq!(signature.len(), 65, "Signature must be 65 bytes long");

        let digest = self.get_digest(voucher);
        let message = libsecp256k1::Message::parse(&digest);
        let sig = libsecp256k1::Signature::parse_overflowing_slice(&signature[..64]).expect("Could not parse libsecp256k1::Signature");
        let rid = libsecp256k1::RecoveryId::parse(signature[64]).expect("Could not parse libsecp256k1::RecoveryId");
        let pub_key = libsecp256k1::recover(&message, &sig, &rid).expect("Could not recover libsecp256k1::PublicKey");
        pub_key.serialize().to_hex()
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