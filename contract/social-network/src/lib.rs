use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PanicOnDefault, PromiseResult};
use near_sdk::json_types::{U128};
use near_sdk::collections::{LookupMap, Vector};
use near_sdk::serde::{Deserialize, Serialize};

pub mod external;
pub use crate::external::*;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    fee_ft: AccountId,
    post_messages: LookupMap<String, MessageList>
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Message {
    sender: AccountId,
    text: String,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct MessageList {
    list: Vector<Message>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCall {
    AddMessage { post_id: String, text: String },
    AddFriend,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner: AccountId, fee_ft: AccountId) -> Self {
        if env::state_exists() == true {
            env::panic_str("Already initialized");
        }
        Self {
            owner,
            fee_ft,
            post_messages: LookupMap::new(b"p".to_vec())
        }
    }

    pub fn add_message(&mut self, post_id: String, text: String) {
        self.collect_fee_and_call(ContractCall::AddMessage { post_id, text });
    }

    pub fn get_post_messages(&self, post_id: String, from_index: u64, limit: u64) -> Vec<Message> {
        if let Some(messages) = self.post_messages.get(&post_id) {
            (from_index..std::cmp::min(from_index + limit, messages.list.len()))
                .map(|index| {
                    let message = messages.list.get(index).unwrap();
                    Message {
                        sender: message.sender,
                        text: message.text
                    }
                })
                .collect()
        } else {
          Vec::new()
        }
    }
}


// Private functions
#[near_bindgen]
impl Contract {

    fn add_message_call(&mut self, post_id: String, text: String) {
        let message = Message {
            sender: env::signer_account_id().clone(),
            text
        };

        let mut messages = self.post_messages.get(&post_id).unwrap_or_else(|| {
            let mut prefix = Vec::with_capacity(33);
            prefix.push(b'm');
            prefix.extend(env::sha256(post_id.as_bytes()));
            MessageList {
                list: Vector::new(prefix),
            }
        });

        messages.list.push(&message);
        self.post_messages.insert(&post_id, &messages);
    }

    fn collect_fee_and_call(&mut self, call: ContractCall) -> Promise {
        ext_ft::ext(self.fee_ft.clone())
            .with_static_gas(Gas(5*TGAS))
            .ft_collect_fee(U128::from(FIXED_FEE))
                .then(
                    ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5*TGAS))
                    .on_fee_collected(call)
                )
    }

    #[private]
    pub fn on_fee_collected(&mut self, call: ContractCall) -> String {
        if env::promise_results_count() != 1 {
            env::panic_str("Unexpected promise results count");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                match call {
                    ContractCall::AddMessage { post_id, text } => {
                        self.add_message_call(post_id, text);
                    },
                    _ => {
                        env::panic_str("Unknown contract call");
                        // TODO: add refund
                    }
                }
                return "Success".to_string();
            },
            _ => env::panic_str("Fee was not charged"),
        };
    }
}