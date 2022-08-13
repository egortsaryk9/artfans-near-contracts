use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PanicOnDefault, PromiseResult};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{LookupMap, LookupSet, Vector};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::BorshStorageKey;

pub mod external;
pub use crate::external::*;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    fee_ft: AccountId,
    posts: LookupMap<PostId, Post>,
    likes: LookupMap<AccountId, AccountLikesStats>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Posts,
    Messages { post_id: Vec<u8> },
    AccountsLikesStats,
    AccountLikedPosts { account_id: Vec<u8> },
    AccountLikedMessages { account_id: Vec<u8>, post_id: Vec<u8> }
}

type PostId = String;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Post {
    messages: Vector<Message>,
    // likes_count: u64
}

#[derive(BorshDeserialize, BorshSerialize, Copy, Clone)]
pub struct MessagePartialId {
    id: u64,
    parent_id: u64 // 0 means the 1st lvl
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum MessagePayload {
    Text { text: String }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message {
    partial_id: MessagePartialId,
    account: AccountId,
    payload: MessagePayload,
    // likes_count: u64
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountLikedPostWithMessages {
    is_post_liked: bool,
    liked_messages: LookupSet<u64>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountLikesStats {
    posts: LookupMap<PostId, AccountLikedPostWithMessages>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCall {
    AddMessage { post_id: PostId, text: String },
    TogglePostLike { post_id: PostId },
    ToggleMessageLike { message_id: InputMessageId },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCallResult {
    AddMessageResult { message_id: InputMessageId },
    TogglePostLikeResult { is_liked: bool },
    ToggleMessageLikeResult { is_liked: bool }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct InputMessageId {
    post_id: PostId,
    msg_idx: U64
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OutputMessage {
    msg_idx: U64,
    account: AccountId,
    text: Option<String>
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
            posts: LookupMap::new(StorageKeys::Posts),
            likes: LookupMap::new(StorageKeys::AccountsLikesStats),
        }
    }

    pub fn add_message(&mut self, post_id: PostId, text: String) -> Promise {
        self.assert_add_message_call(&post_id, &text);
        self.collect_fee_and_execute_call(ContractCall::AddMessage { post_id, text })
    }

    pub fn toggle_post_like(&mut self, post_id: PostId) -> Promise {
        self.assert_toggle_post_like_call(&post_id);
        self.collect_fee_and_execute_call(ContractCall::TogglePostLike { post_id })
    }

    pub fn toggle_message_like(&mut self, message_id: InputMessageId) -> Promise {
        self.assert_toggle_message_like_call(&message_id);
        self.collect_fee_and_execute_call(ContractCall::ToggleMessageLike { message_id })
    }

    pub fn get_post_messages(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<OutputMessage> {
        if let Some(post) = self.posts.get(&post_id) {
            let from = u64::from(from_index);
            let lim = u64::from(limit);
            (from..std::cmp::min(from + lim, post.messages.len()))
                .map(|idx| {
                    let message = post.messages.get(idx).unwrap();
                    match message.payload {
                        MessagePayload::Text { text } => {
                            OutputMessage {
                                msg_idx: U64(idx),
                                account: message.account,
                                text: Some(text)
                            }
                        }
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


    // Assert incoming action

    fn assert_add_message_call(&self, post_id: &PostId, text: &String) {
        // TODO: add check for string MAX_LENGTH
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        }
    }

    fn assert_toggle_post_like_call(&self, post_id: &PostId) {
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
    }

    fn assert_toggle_message_like_call(&self, message_id: &InputMessageId) {
        let post_id = &message_id.post_id;
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }

        let msg_idx = u64::from(message_id.msg_idx.clone());
        if let Some(post) = self.posts.get(post_id) {
            let max_idx = post.messages.len() - 1;
            if msg_idx > max_idx {
                env::panic_str("'msg_idx' is out of bounds");
            }
        }
    }


    // Add storage collections

    fn add_post_storage(&mut self, post_id: &PostId) -> Post {
        let post = Post {
            messages: Vector::new(
                StorageKeys::Messages { 
                    post_id: env::sha256(post_id.as_bytes()) 
                }
            ),
          // likes_count: 0
        };

        self.posts.insert(post_id, &post);

        post
    }

    fn add_account_likes_stats_storage(&mut self, account_id: &AccountId) -> AccountLikesStats {
        let likes_stats = AccountLikesStats {
            posts: LookupMap::new(
                StorageKeys::AccountLikedPosts { 
                    account_id: env::sha256(account_id.as_bytes()) 
                }
            )
        };
        self.likes.insert(account_id, &likes_stats);

        likes_stats
    }

    fn add_account_liked_post_storage(&mut self, account_id: &AccountId, post_id: &PostId) -> AccountLikedPostWithMessages {
        let mut likes_stats = self.likes.get(account_id).unwrap_or_else(|| {
            self.add_account_likes_stats_storage(account_id)
        });

        let liked_post_stat = AccountLikedPostWithMessages {
            is_post_liked: false,
            liked_messages: LookupSet::new(
                StorageKeys::AccountLikedMessages {
                    account_id: env::sha256(account_id.as_bytes()), 
                    post_id: env::sha256(post_id.as_bytes()) 
                }
            )
        };

        likes_stats.posts.insert(post_id, &liked_post_stat);
        self.likes.insert(account_id, &likes_stats);

        liked_post_stat
    }

    
    // Execute call logic

    fn execute_add_message_call(&mut self, post_id: &PostId, text: String) -> InputMessageId {
        let account = env::signer_account_id();
        let mut post = self.posts.get(post_id).unwrap_or_else(|| {
            self.add_post_storage(post_id)
        });

        let partial_id = MessagePartialId {
            id: post.messages.len() + 1,
            parent_id: 0
        };
        
        let message = Message {
            partial_id: partial_id,
            account,
            payload: MessagePayload::Text { text },
            // likes_count: 0
        };

        post.messages.push(&message);
        self.posts.insert(post_id, &post);

        InputMessageId {
            post_id: post_id.clone(), 
            msg_idx: U64(post.messages.len() - 1)
        }
    }

    fn execute_toggle_post_like_call(&mut self, post_id: &PostId) -> bool {
        let account_id = env::signer_account_id();

        let mut likes_stats = self.likes.get(&account_id).unwrap_or_else(|| {
            self.add_account_likes_stats_storage(&account_id)
        });

        let mut liked_post_stat = likes_stats.posts.get(post_id).unwrap_or_else(|| {
            self.add_account_liked_post_storage(&account_id, post_id)
        });

        let is_liked = !liked_post_stat.is_post_liked;
        liked_post_stat.is_post_liked = is_liked;
        likes_stats.posts.insert(post_id, &liked_post_stat);
        self.likes.insert(&account_id, &likes_stats);

        // TODO: Replace with Event ?
        // self.update_post_likes_count(post_id, is_liked);

        is_liked
    }

    fn execute_toggle_message_like_call(&mut self, message_id: &InputMessageId) -> bool {
        let account_id = env::signer_account_id();
        let post_id = &message_id.post_id;
        let msg_idx = u64::from(message_id.msg_idx.clone());

        let mut likes_stats = self.likes.get(&account_id).unwrap_or_else(|| {
            self.add_account_likes_stats_storage(&account_id)
        });

        let mut liked_post_stat = likes_stats.posts.get(post_id).unwrap_or_else(|| {
            self.add_account_liked_post_storage(&account_id, post_id)
        });

        let is_liked = !liked_post_stat.liked_messages.contains(&msg_idx);
        if is_liked {
            liked_post_stat.liked_messages.remove(&msg_idx);
        } else {
            liked_post_stat.liked_messages.insert(&msg_idx);
        }
        likes_stats.posts.insert(post_id, &liked_post_stat);
        self.likes.insert(&account_id, &likes_stats);

        // TODO: Replace with Event ?
        // self.update_message_likes_count(message_id, is_liked);

        is_liked
    }


    // TODO: Revise this
    // fn update_message_likes_count(&mut self, message_id: &InputMessageId, is_liked: bool) {
    //     let post_id = &message_id.post_id;
    //     let msg_idx = u64::from(message_id.msg_idx.clone());

    //     let mut post = self.posts.get(post_id).unwrap();
    //     let mut message = post.messages.get(msg_idx).unwrap();

    //     if is_liked {
    //         message.likes_count += 1;
    //     } else {
    //         message.likes_count -= 1;
    //     }
    //     post.messages.replace(msg_idx, &message);
    //     self.posts.insert(post_id, &post);
    // }


    // TODO: Revise this
    // fn update_post_likes_count(&mut self, post_id: &PostId, is_liked: bool) {
    //     let mut post = self.posts.get(post_id).unwrap();
    //     if is_liked {
    //         post.likes_count += 1;
    //     } else {
    //         post.likes_count -= 1;
    //     }
    //     self.posts.insert(post_id, &post);
    // }


    fn collect_fee_and_execute_call(&mut self, call: ContractCall) -> Promise {
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
                        let message_id = self.execute_add_message_call(&post_id, text);
                        return ContractCallResult::AddMessageResult { message_id }
                    },
                    ContractCall::TogglePostLike { post_id } => {
                        let is_liked = self.execute_toggle_post_like_call(&post_id);
                        return ContractCallResult::TogglePostLikeResult { is_liked }
                    },
                    ContractCall::ToggleMessageLike { message_id } => {
                        let is_liked = self.execute_toggle_message_like_call(&message_id);
                        return ContractCallResult::ToggleMessageLikeResult { is_liked }
                    },
                }
            },
            _ => env::panic_str("Fee was not charged"),
        };
    }
}