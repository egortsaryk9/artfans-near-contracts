use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PanicOnDefault, PromiseResult};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{LookupMap, LookupSet, Vector};
use near_sdk::serde::{Deserialize, Serialize};

pub mod external;
pub use crate::external::*;

type PostId = String;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    fee_ft: AccountId,
    posts: LookupMap<PostId, Post>,
    likes: LookupMap<AccountId, AccountLikesStat>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Post {
    messages: Vector<Message>,
    likes_count: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message {
    account: AccountId,
    text: String,
    likes_count: u64
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountLikedPostState {
    is_post_liked: bool,
    liked_messages_idx: LookupSet<u64>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountLikesStat {
    posts: LookupMap<PostId, AccountLikedPostState>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCall {
    AddMessage { post_id: PostId, text: String },
    ToggleLike { post_id: PostId, message_idx: Option<U64> }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCallResult {
    AddMessageResult { message_id: MessageId },
    ToggleLikeResult { is_liked: bool }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MessageId {
    post_id: PostId,
    message_idx: U64
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MessageDTO {
    message_id: MessageId,
    account: AccountId,
    text: String
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
            posts: LookupMap::new(b'm'),
            likes: LookupMap::new(b'l'),
        }
    }

    pub fn add_message(&mut self, post_id: PostId, text: String) -> Promise {
        self.validate_add_message_call(&post_id, &text);
        self.collect_fee_and_execute(ContractCall::AddMessage { post_id, text })
    }

    pub fn toggle_like(&mut self, post_id: PostId, message_idx: Option<U64>) -> Promise {
        self.validate_toggle_like_call(&post_id, &message_idx);
        self.collect_fee_and_execute(ContractCall::ToggleLike { post_id, message_idx })
    }

    pub fn get_post_messages(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<MessageDTO> {
        if let Some(post) = self.posts.get(&post_id) {
            let from = u64::from(from_index);
            let lim = u64::from(limit);
            (from..std::cmp::min(from + lim, post.messages.len()))
                .map(|idx| {
                    let message = post.messages.get(idx).unwrap();
                    MessageDTO {
                        message_id: MessageId {
                            post_id: post_id.clone(),
                            message_idx: U64(idx)
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

    fn validate_add_message_call(&self, post_id: &PostId, text: &String) {
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        }
    }

    fn execute_add_message_call(&mut self, post_id: PostId, text: String) -> MessageId {
        let account = env::signer_account_id();

        let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
            let mut pref = Vec::with_capacity(33);
            pref.push(b'm');
            pref.extend(env::sha256(post_id.as_bytes()));
            Post {
                messages: Vector::new(pref),
                likes_count: 0
            }
        });

        let message = Message {
            account,
            text,
            likes_count: 0
        };

        post.messages.push(&message);
        self.posts.insert(&post_id, &post);

        MessageId {
            post_id, 
            message_idx: U64(post.messages.len() - 1)
        }
    }


    fn validate_toggle_like_call(&self, post_id: &PostId, message_idx: &Option<U64>) {
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
        if let Some(idx) = message_idx {
            // Message like
            if let Some(post) = self.posts.get(&post_id) {
                let max_idx = U64(post.messages.len() - 1);
                if idx > &max_idx {
                    env::panic_str("'message_idx' is out of bounds");
                }
            }
        } else {
            // Post like
        }
    }


    fn execute_toggle_like_call(&mut self, post_id: PostId, message_idx: Option<U64>) -> bool {
        let account = env::signer_account_id();

        let mut account_likes = self.likes.get(&account).unwrap_or_else(|| {
            // Initialize account likes statistic for this post
            let mut stats_pref = Vec::with_capacity(33);
            stats_pref.push(b'l');
            stats_pref.extend(env::sha256(account.as_bytes()));

            let mut account_likes = AccountLikesStat {
                posts: LookupMap::new(stats_pref)
            };

            let mut liked_messages_pref = Vec::with_capacity(65);
            liked_messages_pref.push(b'l');
            liked_messages_pref.extend(env::sha256(account.as_bytes()));
            liked_messages_pref.extend(env::sha256(post_id.as_bytes()));

            let liked_post = AccountLikedPostState {
                is_post_liked: false,
                liked_messages_idx: LookupSet::new(liked_messages_pref)
            };

            account_likes.posts.insert(&post_id, &liked_post);
            account_likes
        });


        let mut liked_post = account_likes.posts.get(&post_id).unwrap();

        let is_liked : bool;
        if let Some(idx) = message_idx {
            // Message like
            let msg_idx = u64::from(idx);
            is_liked = !liked_post.liked_messages_idx.contains(&msg_idx);
            if is_liked {
                liked_post.liked_messages_idx.remove(&msg_idx);
            } else {
                liked_post.liked_messages_idx.insert(&msg_idx);
            }

            let mut post = self.posts.get(&post_id).unwrap();
            let mut message = post.messages.get(msg_idx).unwrap();

            if is_liked {
                message.likes_count += 1;
            } else {
                message.likes_count -= 1;
            }

            post.messages.replace(msg_idx, &message);
            self.posts.insert(&post_id, &post);

        } else {
            // Post like
            is_liked = !liked_post.is_post_liked;
            liked_post.is_post_liked = is_liked;
            
            let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
                let mut pref = Vec::with_capacity(33);
                pref.push(b'm');
                pref.extend(env::sha256(post_id.as_bytes()));
                Post {
                    messages: Vector::new(pref),
                    likes_count: 0
                }
            });

            if is_liked {
                post.likes_count += 1;
            } else {
                post.likes_count -= 1;
            }

            self.posts.insert(&post_id, &post);
        }

        account_likes.posts.insert(&post_id, &liked_post);
        self.likes.insert(&account, &account_likes);

        is_liked
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
                        let is_liked = self.execute_toggle_like_call(post_id, message_idx);
                        return ContractCallResult::ToggleLikeResult { is_liked }
                    },
                }
            },
            _ => env::panic_str("Fee was not charged"),
        };
    }
}