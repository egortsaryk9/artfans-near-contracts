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
    account_likes: LookupMap<AccountId, AccountLikesStats>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Posts,
    Messages { post_id: Vec<u8> },
    PostLikes { post_id: Vec<u8> },
    MessageLikes { post_id: Vec<u8>, msg_idx: u64 },
    AccountsLikesStats,
    AccountLikedPosts { account_id: Vec<u8> },
    AccountLikedMessages { account_id: Vec<u8>, post_id: Vec<u8> },
}

type PostId = String;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Post {
    messages: Vector<Message>,
    likes: LookupSet<AccountId>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum MessagePayload {
    Text { text: String }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message {
    account: AccountId,
    payload: MessagePayload,
    likes: LookupSet<AccountId>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountLikedPostWithMessages {
    is_post_liked: bool,
    liked_messages: LookupSet<u64>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountLikesStats {
    posts: LookupMap<PostId, AccountLikedPostWithMessages>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCall {
    AddMessage { post_id: PostId, text: String },
    LikePost { post_id: PostId },
    UnlikePost { post_id: PostId },
    ToggleMessageLike { message_id: PostMessageId },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCallResult {
    AddMessageResult { id: PostMessageId },
    LikePostResult,
    UnlikePostResult,
    ToggleMessageLikeResult { is_liked: bool }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PostMessageId {
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
            account_likes: LookupMap::new(StorageKeys::AccountsLikesStats),
        }
    }

    pub fn add_message(&mut self, post_id: PostId, text: String) -> Promise {
        self.assert_add_message_call(&post_id, &text);
        self.collect_fee_and_execute_call(ContractCall::AddMessage { post_id, text })
    }

    pub fn like_post(&mut self, post_id: PostId) -> Promise {
        self.assert_like_post_call(&post_id);
        self.collect_fee_and_execute_call(ContractCall::LikePost { post_id })
    }

    pub fn unlike_post(&mut self, post_id: PostId) -> Promise {
        self.assert_unlike_post_call(&post_id);
        self.collect_fee_and_execute_call(ContractCall::LikePost { post_id })
    }

    // pub fn toggle_post_like(&mut self, post_id: PostId) -> Promise {
    //     self.assert_toggle_post_like_call(&post_id);
    //     self.collect_fee_and_execute_call(ContractCall::LikePost { post_id })
    // }

    pub fn toggle_message_like(&mut self, message_id: PostMessageId) -> Promise {
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
        self.assert_post_id(post_id);

        // TODO: validate 'text' format and length
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        }
    }

    fn assert_like_post_call(&self, post_id: &PostId) {
        let account_id = env::signer_account_id();

        self.assert_post_id(post_id);
        self.assert_post_exists(post_id);

        if let Some(stats) = self.account_likes.get(&account_id) {
            if let Some(post_stat) = stats.posts.get(post_id) {
                if post_stat.is_post_liked {
                    env::panic_str("'post_id' is already liked");
                }
            }
        }
    }

    fn assert_unlike_post_call(&self, post_id: &PostId) {
        let account_id = env::signer_account_id();

        self.assert_post_id(post_id);
        self.assert_post_exists(post_id);

        if let Some(stats) = self.account_likes.get(&account_id) {
            if let Some(post_stat) = stats.posts.get(post_id) {
                if !post_stat.is_post_liked {
                    env::panic_str("'post_id' is not liked");
                }
            }
        }
    }


    // fn assert_toggle_post_like_call(&self, post_id: &PostId) {
    //     if post_id.trim().is_empty() {
    //         env::panic_str("'post_id' is empty or whitespace");
    //     }
    // }

    fn assert_toggle_message_like_call(&self, message_id: &PostMessageId) {
        let post_id = &message_id.post_id;
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }

        let msg_idx = u64::from(message_id.msg_idx.clone());
        if let Some(post) = self.posts.get(post_id) {
            if !post.messages.get(msg_idx).is_some() {
                env::panic_str("'msg_idx' does not exist");
            }
        }
    }

    fn assert_post_id(&self, post_id: &PostId) {
        // TODO: validate 'post_id' format and length
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
    }

    fn assert_post_exists(&self, post_id: &PostId) {
        if !self.posts.get(post_id).is_some() {
            env::panic_str("'post_id' does not exist");
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
            likes: LookupSet::new(
                StorageKeys::PostLikes { 
                    post_id: env::sha256(post_id.as_bytes())
                }
            )
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
        self.account_likes.insert(account_id, &likes_stats);

        likes_stats
    }

    fn add_account_liked_post_storage(&mut self, account_id: &AccountId, post_id: &PostId) -> AccountLikedPostWithMessages {
        let mut likes_stats = self.account_likes.get(account_id).unwrap_or_else(|| {
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
        self.account_likes.insert(account_id, &likes_stats);

        liked_post_stat
    }

    
    // Execute call logic

    fn execute_add_message_call(&mut self, post_id: &PostId, text: String) -> u64 {
        let account = env::signer_account_id();
        
        let mut post = self.posts.get(post_id).unwrap_or_else(|| {
            self.add_post_storage(post_id)
        });
        
        let msg_idx = post.messages.len();
        let msg = Message {
            account,
            payload: MessagePayload::Text { text },
            likes: LookupSet::new(
                StorageKeys::MessageLikes { 
                    post_id: env::sha256(post_id.as_bytes()),
                    msg_idx: msg_idx 
                }
            )
            // likes_count: 0
        };
        post.messages.push(&msg);
        self.posts.insert(post_id, &post);

        msg_idx
    }

    fn execute_like_post_call(&mut self, post_id: &PostId) {
        let account_id = env::signer_account_id();

        // Update post stats
        let mut post = self.posts.get(post_id).unwrap_or_else(|| {
            self.add_post_storage(post_id)
        });

        post.likes.insert(&account_id);
        self.posts.insert(post_id, &post);

        // Update account stats
        let mut likes_stats = self.account_likes.get(&account_id).unwrap_or_else(|| {
            self.add_account_likes_stats_storage(&account_id)
        });

        let mut liked_post_stat = likes_stats.posts.get(post_id).unwrap_or_else(|| {
            self.add_account_liked_post_storage(&account_id, post_id)
        });

        liked_post_stat.is_post_liked = true;
        likes_stats.posts.insert(post_id, &liked_post_stat);
        self.account_likes.insert(&account_id, &likes_stats);
    }


    fn execute_unlike_post_call(&mut self, post_id: &PostId) {
        let account_id = env::signer_account_id();

        // Update post stats
        let mut post = self.posts.get(post_id).unwrap();
        if !post.likes.remove(&account_id) {
            env::panic_str("'post_id' is not liked");
        }
        self.posts.insert(post_id, &post);


        // Update account stats
        let mut likes_stats = self.account_likes.get(&account_id).unwrap_or_else(|| {
            self.add_account_likes_stats_storage(&account_id)
        });

        let mut liked_post_stat = likes_stats.posts.get(post_id).unwrap_or_else(|| {
            self.add_account_liked_post_storage(&account_id, post_id)
        });

        liked_post_stat.is_post_liked = false;
        likes_stats.posts.insert(post_id, &liked_post_stat);
        self.account_likes.insert(&account_id, &likes_stats);
    }


    // fn execute_toggle_post_like_call(&mut self, post_id: &PostId) -> bool {
    //     let account_id = env::signer_account_id();

    //     let mut likes_stats = self.account_likes.get(&account_id).unwrap_or_else(|| {
    //         self.add_account_likes_stats_storage(&account_id)
    //     });

    //     let mut liked_post_stat = likes_stats.posts.get(post_id).unwrap_or_else(|| {
    //         self.add_account_liked_post_storage(&account_id, post_id)
    //     });

    //     let is_liked = !liked_post_stat.is_post_liked;
    //     liked_post_stat.is_post_liked = is_liked;
    //     likes_stats.posts.insert(post_id, &liked_post_stat);
    //     self.account_likes.insert(&account_id, &likes_stats);

    //     // TODO: Replace with Event ?
    //     // self.update_post_likes_count(post_id, is_liked);

    //     is_liked
    // }

    fn execute_toggle_message_like_call(&mut self, message_id: &PostMessageId) -> bool {
        let account_id = env::signer_account_id();
        let post_id = &message_id.post_id;
        let msg_idx = u64::from(message_id.msg_idx.clone());

        let mut likes_stats = self.account_likes.get(&account_id).unwrap_or_else(|| {
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
        self.account_likes.insert(&account_id, &likes_stats);

        // TODO: Replace with Event ?
        // self.update_message_likes_count(message_id, is_liked);

        is_liked
    }


    // TODO: Revise this
    // fn update_message_likes_count(&mut self, message_id: &PostMessageId, is_liked: bool) {
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
                        let msg_idx = self.execute_add_message_call(&post_id, text);
                        return ContractCallResult::AddMessageResult { id: PostMessageId { post_id, msg_idx: U64(msg_idx) } }
                    },
                    ContractCall::LikePost { post_id } => {
                        self.execute_like_post_call(&post_id);
                        return ContractCallResult::LikePostResult
                    },
                    ContractCall::UnlikePost { post_id } => {
                        self.execute_unlike_post_call(&post_id);
                        return ContractCallResult::UnlikePostResult
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