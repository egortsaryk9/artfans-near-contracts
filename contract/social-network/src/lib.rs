use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PanicOnDefault, PromiseResult};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{LookupMap, LookupSet, Vector};
use near_sdk::serde::{Deserialize, Serialize};

pub mod external;
pub use crate::external::*;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    fee_ft: AccountId,
    post_messages: LookupMap<String, MessageList>,
    account_likes: LookupMap<AccountId, PostsLikesStatsSet>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct MessageList {
    list: Vector<Message>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message {
    account: AccountId,
    text: String
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PostLikesStat {
    post_id: String,
    is_post_liked: bool,
    liked_messages_idx: LookupSet<u64>
}

impl PartialEq for PostLikesStat {
    fn eq(&self, other: &Self) -> bool {
        self.post_id == other.post_id
    }
}

impl Eq for PostLikesStat {}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PostsLikesStatsSet {
    set: LookupSet<PostLikesStat>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCall {
    AddMessage { post_id: String, text: String },
    ToggleLike { post_id: String, message_idx: Option<U64> }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCallResult {
    AddMessageResult { message_id: MessageID },
    ToggleLikeResult { like_id: LikeID, is_enabled: bool }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MessageID {
    post_id: String,
    message_idx: U64
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MessageDTO {
    message_id: MessageID,
    account: AccountId,
    text: String
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LikeID {
    account: AccountId,
    post_id: String,
    message_idx: Option<U64>
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
            post_messages: LookupMap::new(b'm'),
            account_likes: LookupMap::new(b'l'),
        }
    }

    pub fn add_message(&mut self, post_id: String, text: String) -> Promise {
        self.validate_add_message_call(&post_id, &text);
        self.collect_fee_and_execute(ContractCall::AddMessage { post_id, text })
    }

    pub fn toggle_like(&mut self, post_id: String, message_idx: Option<U64>) -> Promise {
        self.validate_toggle_like_call(&post_id, &message_idx);
        self.collect_fee_and_execute(ContractCall::ToggleLike { post_id, message_idx })
    }




    pub fn get_post_messages(&self, post_id: String, from_index: u64, limit: u64) -> Vec<MessageDTO> {
        if let Some(messages) = self.post_messages.get(&post_id) {
            (from_index..std::cmp::min(from_index + limit, messages.list.len()))
                .map(|index| {
                    let message = messages.list.get(index).unwrap();
                    MessageDTO {
                        message_id: MessageID {
                            post_id: post_id.clone(),
                            message_idx: U64(index)
                        },
                        account: message.account,
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

    fn validate_add_message_call(&self, post_id: &String, text: &String) {
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        }
    }

    fn execute_add_message_call(&mut self, post_id: String, text: String) -> MessageID {
        let mut messages = self.post_messages.get(&post_id).unwrap_or_else(|| {
            let mut prefix = Vec::with_capacity(33);
            prefix.push(b'm');
            prefix.extend(env::sha256(post_id.as_bytes()));
            MessageList {
                list: Vector::new(prefix)
            }
        });

        let message = Message {
            account: env::signer_account_id(),
            text
        };

        messages.list.push(&message);
        self.post_messages.insert(&post_id, &messages);

        MessageID {
            post_id, 
            message_idx: U64(messages.list.len() - 1)
        }
    }


    fn validate_toggle_like_call(&self, post_id: &String, message_idx: &Option<U64>) {
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
        if let Some(idx) = message_idx {
            // Message like
            if let Some(messages) = self.post_messages.get(&post_id) {
                let max_idx = U64::from(messages.list.len() - 1);
                if idx >= &max_idx {
                    env::panic_str("'message_idx' is out of bounds");
                }
            }
        } else {
            // Post like
        }
    }


    fn execute_toggle_like_call(&mut self, post_id: String, message_idx: Option<U64>) -> LikeID {
        let account = env::signer_account_id();

        let likes_stats = self.account_likes.get(&account).unwrap_or_else(|| {
            // First initialization for post likes
            let mut stats_prefix = Vec::with_capacity(33);
            stats_prefix.push(b'l');
            stats_prefix.extend(env::sha256(account.as_bytes()));

            let mut likes_stats = PostsLikesStatsSet {
                set: LookupSet::new(stats_prefix)
            };

            let mut liked_messages_prefix = Vec::with_capacity(65);
            liked_messages_prefix.push(b'l');
            liked_messages_prefix.extend(env::sha256(account.as_bytes()));
            liked_messages_prefix.extend(env::sha256(post_id.as_bytes()));

            let likes_stat = PostLikesStat {
                post_id: post_id.clone(),
                is_post_liked: false,
                liked_messages_idx: LookupSet::new(liked_messages_prefix)
            };

            likes_stats.set.insert(&likes_stat);
            likes_stats
        });

        LikeID {
            account: env::signer_account_id(),
            post_id,
            message_idx
        }
    }



    fn collect_fee_and_execute(&mut self, call: ContractCall) -> Promise {
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
    pub fn on_fee_collected(&mut self, call: ContractCall) -> ContractCallResult {
        if env::promise_results_count() != 1 {
            env::panic_str("Unexpected promise results count");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                match call {
                    ContractCall::AddMessage { post_id, text } => {
                        let message_id = self.execute_add_message_call(post_id, text);
                        return ContractCallResult::AddMessageResult { message_id }
                    },
                    ContractCall::ToggleLike { post_id, message_idx } => {
                        let like_id = self.execute_toggle_like_call(post_id, message_idx);
                        return ContractCallResult::ToggleLikeResult { like_id, is_enabled: true }
                    },
                }
            },
            _ => env::panic_str("Fee was not charged"),
        };
    }
}