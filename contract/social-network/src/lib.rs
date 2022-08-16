use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PanicOnDefault, PromiseResult};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{LookupMap, LookupSet, Vector, UnorderedSet

};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::BorshStorageKey;
use std::convert::From;

pub mod external;
pub use crate::external::*;

type PostId = String;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    fee_ft: AccountId,
    posts: LookupMap<PostId, Post>,
    account_stats: LookupMap<AccountId, AccountStats>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Posts,
    Messages { post_id: Vec<u8> },
    PostLikes { post_id: Vec<u8> },
    MessageLikes { post_id: Vec<u8>, msg_idx: u64 },
    AccountsStats,
    AccountRecentLikes { account_id: Vec<u8> }
}


#[derive(BorshSerialize, BorshDeserialize)]
pub struct MessageId {
    post_id: PostId,
    msg_idx: u64
}

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
pub enum AccountLike {
    PostLike { post_id: PostId },
    MessageLike { msg_id: MessageId }
}

impl PartialEq for AccountLike {
    fn eq(&self, other: &Self) -> bool {
        use AccountLike::*;
        match (self, other) {
            (PostLike { post_id: id1 }, PostLike { post_id: id2 } ) => id1 == id2,
            (MessageLike { msg_id: id1 }, MessageLike { msg_id: id2 }) => id1.post_id == id2.post_id && id1.msg_idx == id2.msg_idx,
            _ => false,
        }
    }
}

impl Eq for AccountLike {}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message {
    account: AccountId,
    payload: MessagePayload,
    likes: LookupSet<AccountId>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountStats {
    recent_likes: UnorderedSet<AccountLike>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Call {
    AddMessage { post_id: PostId, text: String },
    LikePost { post_id: PostId },
    UnlikePost { post_id: PostId },
    LikeMessage { msg_id: ExtMessageId },
    UnlikeMessage { msg_id: ExtMessageId },
}

type CallExecutionResult = Result<CallSuccess, CallFailure>;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum CallSuccess {
    AddMessageSuccess { post_id: String, msg_idx: U64 },
    LikePostSuccess,
    UnlikePostSuccess,
    LikeMessageSuccess,
    UnlikeMessageSuccess
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum CallFailure {
    AddMessageFailure,
    LikePostFailure { reason: LikePostFailureReason },
    UnlikePostFailure { reason: UnlikePostFailureReason },
    LikeMessageFailure { reason: LikeMessageFailureReason },
    UnlikeMessageFailure { reason: UnlikeMessageFailureReason },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum CallResponse {
    AddMessageResponse { id: ExtMessageId },
    LikePostResponse,
    UnlikePostResponse,
    LikeMessageResponse,
    UnlikeMessageResponse,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum LikePostFailureReason {
    PostWasLiked
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum UnlikePostFailureReason {
    PostWasNotLiked
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum LikeMessageFailureReason {
    MessageWasLiked
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum UnlikeMessageFailureReason {
    MessageWasNotLiked
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ExtMessageId {
    post_id: PostId,
    msg_idx: U64
}

impl From<ExtMessageId> for MessageId {
    fn from(v: ExtMessageId) -> Self {
        MessageId { 
            post_id: v.post_id, 
            msg_idx: u64::from(v.msg_idx) 
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ExtMessage {
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
            account_stats: LookupMap::new(StorageKeys::AccountsStats),
        }
    }

    pub fn add_message(&mut self, post_id: PostId, text: String) -> Promise {
        self.assert_add_message_call(&post_id, &text);
        self.collect_fee_and_execute_call(Call::AddMessage { post_id, text })
    }

    pub fn like_post(&mut self, post_id: PostId) -> Promise {
        self.assert_like_post_call(&post_id);
        self.collect_fee_and_execute_call(Call::LikePost { post_id })
    }

    pub fn unlike_post(&mut self, post_id: PostId) -> Promise {
        self.assert_unlike_post_call(&post_id);
        self.collect_fee_and_execute_call(Call::UnlikePost { post_id })
    }

    pub fn like_message(&mut self, msg_id: ExtMessageId) -> Promise {
        self.assert_like_message_call(&msg_id);
        self.collect_fee_and_execute_call(Call::LikeMessage { msg_id })
    }

    pub fn unlike_message(&mut self, msg_id: ExtMessageId) -> Promise {
        self.assert_unlike_message_call(&msg_id);
        self.collect_fee_and_execute_call(Call::UnlikeMessage { msg_id })
    }

    pub fn get_post_messages(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<ExtMessage> {
        if let Some(post) = self.posts.get(&post_id) {
            let from = u64::from(from_index);
            let lim = u64::from(limit);
            (from..std::cmp::min(from + lim, post.messages.len()))
                .map(|idx| {
                    let message = post.messages.get(idx).unwrap();
                    match message.payload {
                        MessagePayload::Text { text } => {
                            ExtMessage {
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

        if let Some(post) = self.posts.get(post_id) {
            if post.likes.contains(&account_id) {
                env::panic_str("Post is liked already");
            }
        } 
        // else {
        //     env::panic_str("Post does not exist");
        // }
    }

    fn assert_unlike_post_call(&self, post_id: &PostId) {
        let account_id = env::signer_account_id();

        self.assert_post_id(post_id);

        if let Some(post) = self.posts.get(post_id) {
            if !post.likes.contains(&account_id) {
                env::panic_str("Post is not liked");
            }
        } else {
            env::panic_str("Post does not exist");
        }
    }

    fn assert_like_message_call(&self, msg_id: &ExtMessageId) {
        let account_id = env::signer_account_id();
        self.assert_message_id(msg_id);

        let post_id = &msg_id.post_id;
        let msg_idx: u64 = msg_id.msg_idx.into();
        
        if let Some(post) = self.posts.get(post_id) {
            if let Some(msg) = post.messages.get(msg_idx) {
                if msg.likes.contains(&account_id) {
                    env::panic_str("Message is liked already");
                }
            } else {
                env::panic_str("Message does not exist");
            }
        } else {
            env::panic_str("Post does not exist");
        }
    }

    fn assert_unlike_message_call(&self, msg_id: &ExtMessageId) {
        let account_id = env::signer_account_id();
        self.assert_message_id(msg_id);

        let post_id = &msg_id.post_id;
        let msg_idx: u64 = msg_id.msg_idx.into();
        
        if let Some(post) = self.posts.get(post_id) {
            if let Some(msg) = post.messages.get(msg_idx) {
                if !msg.likes.contains(&account_id) {
                    env::panic_str("Message is not liked");
                }
            } else {
                env::panic_str("Message does not exist");
            }
        } else {
            env::panic_str("Post does not exist");
        }
    }
    
    fn assert_post_id(&self, post_id: &PostId) {
        // TODO: validate 'post_id' format and length
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
    }

    fn assert_message_id(&self, msg_id: &ExtMessageId) {
        let post_id = &msg_id.post_id;
        self.assert_post_id(post_id);
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
        };

        self.posts.insert(post_id, &post);

        post
    }

    fn add_account_stat_storage(&mut self, account_id: &AccountId) -> AccountStats {
        let account_stat = AccountStats {
            recent_likes: UnorderedSet::new(
                StorageKeys::AccountRecentLikes { 
                    account_id: env::sha256(account_id.as_bytes()) 
                }
            )
        };

        self.account_stats.insert(account_id, &account_stat);

        account_stat
    }
    
    // Execute call logic

    fn execute_add_message_call(&mut self, post_id: PostId, text: String) -> CallExecutionResult {
        let account = env::signer_account_id();
        
        let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
            self.add_post_storage(&post_id)
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
        };
        post.messages.push(&msg);
        self.posts.insert(&post_id, &post);

        Ok(CallSuccess::AddMessageSuccess { post_id, msg_idx: U64(msg_idx)})
    }

    fn execute_like_post_call(&mut self, post_id: PostId) -> CallExecutionResult {
        let account_id = env::signer_account_id();

        // Update post stats
        let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
            self.add_post_storage(&post_id)
        });
        if !post.likes.insert(&account_id) {
            return Err(CallFailure::LikePostFailure {
                reason: LikePostFailureReason::PostWasLiked
            })
        }
        self.posts.insert(&post_id, &post);

        // Update account stats
        let mut account_stats = self.account_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });
        let like = AccountLike::PostLike { post_id };
        account_stats.recent_likes.insert(&like);
        self.account_stats.insert(&account_id, &account_stats);

        Ok(CallSuccess::LikePostSuccess)
    }


    fn execute_unlike_post_call(&mut self, post_id: PostId) -> CallExecutionResult {
        let account_id = env::signer_account_id();

        // Update post stats
        let mut post = self.posts.get(&post_id).expect("Post is not found");
        if !post.likes.remove(&account_id) {
            env::panic_str("Post is not liked");
        }
        self.posts.insert(&post_id, &post);

        // Update account stats
        let mut account_stats = self.account_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });
        let like = AccountLike::PostLike { post_id };
        account_stats.recent_likes.remove(&like);
        self.account_stats.insert(&account_id, &account_stats);

        Ok(CallSuccess::UnlikePostSuccess)
    }

    fn execute_like_message_call(&mut self, msg_id: MessageId) -> CallExecutionResult {
        let account_id = env::signer_account_id();

        // Update message stats
        let mut post = self.posts.get(&msg_id.post_id).expect("Post is not found");
        let mut msg = post.messages.get(msg_id.msg_idx).expect("Message is not found");
        if !msg.likes.insert(&account_id) {
            env::panic_str("Message is liked already");
        }
        post.messages.replace(msg_id.msg_idx, &msg);
        self.posts.insert(&msg_id.post_id, &post);

        // Update account stats
        let mut account_stats = self.account_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });
        let like = AccountLike::MessageLike { msg_id };
        account_stats.recent_likes.insert(&like);
        self.account_stats.insert(&account_id, &account_stats);

        Ok(CallSuccess::LikeMessageSuccess)
    }


    fn execute_unlike_message_call(&mut self, msg_id: MessageId) -> CallExecutionResult {
        let account_id = env::signer_account_id();

        // Update message stats
        let mut post = self.posts.get(&msg_id.post_id).expect("Post is not found");
        let mut msg = post.messages.get(msg_id.msg_idx).expect("Message is not found");
        if !msg.likes.remove(&account_id) {
            env::panic_str("Message is not liked");
        }
        post.messages.replace(msg_id.msg_idx, &msg);
        self.posts.insert(&msg_id.post_id, &post);

        // Update account stats
        let mut account_stats = self.account_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });

        let like = AccountLike::MessageLike { msg_id };
        account_stats.recent_likes.remove(&like);
        self.account_stats.insert(&account_id, &account_stats);

        Ok(CallSuccess::UnlikeMessageSuccess)
    }


    fn collect_fee_and_execute_call(&mut self, call: Call) -> Promise {
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
    pub fn on_fee_collected(&mut self, call: Call) -> CallResponse {
        if env::promise_results_count() != 1 {
            env::panic_str("Unexpected promise results count");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                match call {
                    Call::AddMessage { post_id, text } => {
                        match self.execute_add_message_call(post_id, text) {
                            Ok(CallSuccess::AddMessageSuccess { post_id, msg_idx }) => { 
                                CallResponse::AddMessageResponse { id: ExtMessageId { post_id, msg_idx } } 
                            },
                            Err(CallFailure::AddMessageFailure) => {
                                 // TODO: add rollback
                                 env::panic_str("AddMessageFailure");
                            },
                            _ => env::panic_str("todo")
                        }
                    },
                    Call::LikePost { post_id } => {
                        match self.execute_like_post_call(post_id) {
                            Ok(CallSuccess::LikePostSuccess) => {
                                CallResponse::LikePostResponse 
                            },
                            Err(CallFailure::LikePostFailure { reason }) => {
                                match reason {
                                    LikePostFailureReason::PostWasLiked => {
                                        // TODO: add rollback
                                        env::panic_str("LikePostFailureReason::PostWasLiked");
                                    },
                                }
                            },
                            _ => env::panic_str("todo")
                        }
                    },
                    Call::UnlikePost { post_id } => {
                        match self.execute_unlike_post_call(post_id) {
                            Ok(CallSuccess::UnlikePostSuccess) => {
                                CallResponse::UnlikePostResponse 
                            },
                            Err(CallFailure::UnlikePostFailure { reason }) => {
                                match reason {
                                    UnlikePostFailureReason::PostWasNotLiked => {
                                        // TODO: add rollback
                                        env::panic_str("UnlikePostFailureReason::PostWasNotLiked");
                                    },
                                }
                            },
                            _ => env::panic_str("todo")
                        }
                    },
                    Call::LikeMessage { msg_id } => {
                        match self.execute_like_message_call(msg_id.into()) {
                            Ok(CallSuccess::LikeMessageSuccess) => {
                                CallResponse::LikeMessageResponse 
                            },
                            Err(CallFailure::LikeMessageFailure { reason }) => {
                                match reason {
                                    LikeMessageFailureReason::MessageWasLiked => {
                                        // TODO: add rollback
                                        env::panic_str("LikeMessageFailureReason::MessageWasLiked");
                                    },
                                }
                            },
                            _ => env::panic_str("todo")
                        }
                    },
                    Call::UnlikeMessage { msg_id } => {
                        match self.execute_unlike_message_call(msg_id.into()) {
                            Ok(CallSuccess::UnlikeMessageSuccess) => {
                                CallResponse::UnlikeMessageResponse 
                            },
                            Err(CallFailure::UnlikeMessageFailure { reason }) => {
                                match reason {
                                    UnlikeMessageFailureReason::MessageWasNotLiked => {
                                        // TODO: add rollback
                                        env::panic_str("UnlikeMessageFailureReason::MessageWasNotLiked");
                                    },
                                }
                            },
                            _ => env::panic_str("todo")
                        }
                    },
                }
            },
            _ => env::panic_str("Fee was not charged"),
        }
    }
}