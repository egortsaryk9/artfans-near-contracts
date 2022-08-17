use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PanicOnDefault, PromiseResult};
use near_sdk::json_types::{U128, U64};
use near_sdk::collections::{LookupMap, Vector, UnorderedSet};
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
    accounts_stats: LookupMap<AccountId, AccountStats>,
    accounts_friends: LookupMap<AccountId, UnorderedSet<AccountId>>
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Posts,
    Messages { post_id: Vec<u8> },
    PostLikes { post_id: Vec<u8> },
    MessageLikes { post_id: Vec<u8>, msg_idx: u64 },
    AccountsStats,
    AccountRecentLikes { account_id: Vec<u8> },
    AccountsFriends,
    AccountFriends { account_id: Vec<u8> },
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct MessageId {
    post_id: PostId,
    msg_idx: u64
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Post {
    messages: Vector<Message>,
    likes: UnorderedSet<AccountId>
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
            (PostLike { post_id: first }, PostLike { post_id: second } ) => {
                first == second
            },
            (MessageLike { msg_id: first }, MessageLike { msg_id: second }) => {
                first.post_id == second.post_id && first.msg_idx == second.msg_idx
            },
            _ => false,
        }
    }
}

impl Eq for AccountLike {}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message {
    account: AccountId,
    parent_idx: Option<u64>,
    payload: MessagePayload,
    likes: UnorderedSet<AccountId>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountStats {
    recent_likes: UnorderedSet<AccountLike>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Call {
    AddMessageToPost { post_id: PostId, text: String },
    AddMessageToMessage { parent_msg_id: MessageID, text: String },
    AddFriend { friend_id: AccountId },
    LikePost { post_id: PostId },
    UnlikePost { post_id: PostId },
    LikeMessage { msg_id: MessageID },
    UnlikeMessage { msg_id: MessageID },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum CallResult {
    MessageToPostAdded { id: MessageID },
    MessageToMessageAdded { id: MessageID },
    FriendAdded,
    PostLiked,
    PostUnliked,
    MessageLiked,
    MessageUnliked
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum AddMessageFailure {}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum LikePostFailure {
    PostIsLikedAlready
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum UnlikePostFailure {
    PostIsNotFound,
    PostIsNotLikedYet
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum LikeMessageFailure {
    PostIsNotFound,
    MessageIsNotFound,
    MessageIsLikedAlready
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum UnlikeMessageFailure {
    PostIsNotFound,
    MessageIsNotFound,
    MessageIsNotLikedYet
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MessageID {
    post_id: PostId,
    msg_idx: U64
}

impl From<MessageID> for MessageId {
    fn from(v: MessageID) -> Self {
        MessageId {
            post_id: v.post_id, 
            msg_idx: u64::from(v.msg_idx) 
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MessageDTO {
    msg_idx: U64,
    parent_idx: Option<U64>,
    account: AccountId,
    text: Option<String>,
    likes_count: U64
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
            accounts_stats: LookupMap::new(StorageKeys::AccountsStats),
            accounts_friends: LookupMap::new(StorageKeys::AccountsFriends)
        }
    }

    pub fn add_message_to_post(&mut self, post_id: PostId, text: String) -> Promise {
        self.assert_add_message_to_post_call(&post_id, &text);
        self.collect_fee_and_execute_call(Call::AddMessageToPost { post_id, text })
    }

    pub fn add_message_to_message(&mut self, parent_msg_id: MessageID, text: String) -> Promise {
        self.assert_add_message_to_message_call(&parent_msg_id, &text);
        self.collect_fee_and_execute_call(Call::AddMessageToMessage { parent_msg_id, text })
    }

    pub fn like_post(&mut self, post_id: PostId) -> Promise {
        self.assert_like_post_call(&post_id);
        self.collect_fee_and_execute_call(Call::LikePost { post_id })
    }

    pub fn add_friend(&mut self, friend_id: AccountId) -> Promise {
        self.assert_add_friend_call(&friend_id);
        self.collect_fee_and_execute_call(Call::AddFriend { friend_id })
    }

    pub fn unlike_post(&mut self, post_id: PostId) -> Promise {
        self.assert_unlike_post_call(&post_id);
        self.collect_fee_and_execute_call(Call::UnlikePost { post_id })
    }

    pub fn like_message(&mut self, msg_id: MessageID) -> Promise {
        self.assert_like_message_call(&msg_id);
        self.collect_fee_and_execute_call(Call::LikeMessage { msg_id })
    }

    pub fn unlike_message(&mut self, msg_id: MessageID) -> Promise {
        self.assert_unlike_message_call(&msg_id);
        self.collect_fee_and_execute_call(Call::UnlikeMessage { msg_id })
    }

    pub fn get_post_messages(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<MessageDTO> {
        if let Some(post) = self.posts.get(&post_id) {
            let from = u64::from(from_index);
            let lim = u64::from(limit);
            (from..std::cmp::min(from + lim, post.messages.len()))
                .map(|idx| {
                    let message = post.messages.get(idx).unwrap();
                    match message.payload {
                        MessagePayload::Text { text } => {
                            MessageDTO {
                                msg_idx: U64(idx),
                                parent_idx: match message.parent_idx {
                                    Some(parent_idx) => Some(U64(parent_idx)),
                                    None => None
                                },
                                account: message.account,
                                text: Some(text),
                                likes_count: U64(message.likes.len())
                            }
                        }
                    }
                })
                .collect()
        } else {
          Vec::new()
        }
    }

    pub fn get_post_likes(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<AccountId> {
        if let Some(post) = self.posts.get(&post_id) {
            use std::convert::TryFrom;
            if let (Ok(from), Ok(lim)) = (usize::try_from(u64::from(from_index)), usize::try_from(u64::from(limit))) {
                post.likes
                    .iter()
                    .skip(from)
                    .take(lim)
                    .collect()
            } else {
                env::panic_str("'usize' conversion failed");
            }
        } else {
          Vec::new()
        }
    }

    pub fn get_message_likes(&self, msg_id: MessageID, from_index: U64, limit: U64) -> Vec<AccountId> {
        if let Some(post) = self.posts.get(&msg_id.post_id) {
            let idx = u64::from(msg_id.msg_idx);
            if let Some(msg) = post.messages.get(idx) {
                use std::convert::TryFrom;
                if let (Ok(from), Ok(lim)) = (usize::try_from(u64::from(from_index)), usize::try_from(u64::from(limit))) {
                    msg.likes
                        .iter()
                        .skip(from)
                        .take(lim)
                        .collect()
                } else {
                    env::panic_str("'usize' conversion failed");
                }
            } else {
                Vec::new()
            }
        } else {
          Vec::new()
        }
    }
    
    pub fn get_account_last_likes(&self, account_id: AccountId, from_index: U64, limit: U64) -> Vec<(PostId, Option<U64>)> {
        if let Some(accounts_stats) = self.accounts_stats.get(&account_id) {
            use std::convert::TryFrom;
            if let (Ok(from), Ok(lim)) = (usize::try_from(u64::from(from_index)), usize::try_from(u64::from(limit))) {
                accounts_stats.recent_likes
                    .iter()
                    .skip(from)
                    .take(lim)
                    .map(|item| {
                        match item {
                            AccountLike::PostLike { post_id } => {
                                (post_id, None)
                            },
                            AccountLike::MessageLike { msg_id } => {
                                (msg_id.post_id, Some(U64(msg_id.msg_idx)))
                            }
                        }
                    })
                    .collect()
            } else {
                env::panic_str("'usize' conversion failed");
            }
        } else {
          Vec::new()
        }
    }

    pub fn get_account_friends(&self, account_id: AccountId, from_index: U64, limit: U64) -> Vec<AccountId> {
        if let Some(account_friends) = self.accounts_friends.get(&account_id) {
            use std::convert::TryFrom;
            if let (Ok(from), Ok(lim)) = (usize::try_from(u64::from(from_index)), usize::try_from(u64::from(limit))) {
                account_friends
                    .iter()
                    .skip(from)
                    .take(lim)
                    .collect()
            } else {
                env::panic_str("'usize' conversion failed");
            }
        } else {
          Vec::new()
        }
    }

}


// Private functions
#[near_bindgen]
impl Contract {



    // Assert incoming call

    fn assert_add_message_to_post_call(&self, post_id: &PostId, text: &String) {
        // TODO: validate 'text' format and length
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        }

        self.assert_post_id(post_id);
    }

    fn assert_add_message_to_message_call(&self, parent_msg_id: &MessageID, text: &String) {
        // TODO: validate 'text' format and length
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        }

        self.assert_message_id(parent_msg_id);

        let post_id = &parent_msg_id.post_id;
        let msg_idx: u64 = parent_msg_id.msg_idx.into();
        
        if let Some(post) = self.posts.get(post_id) {
            if !post.messages.get(msg_idx).is_some() {
                env::panic_str("Parent message does not exist");
            }
        } else {
            env::panic_str("Post does not exist");
        }
    }

    fn assert_add_friend_call(&self, friend_id: &AccountId) {
        let account_id = env::signer_account_id();

        if let Some(account_friends) = self.accounts_friends.get(&account_id) {
            if account_friends.contains(friend_id) {
                env::panic_str("Friend is added already");
            }
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

    fn assert_like_message_call(&self, msg_id: &MessageID) {
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

    fn assert_unlike_message_call(&self, msg_id: &MessageID) {
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

    fn assert_message_id(&self, msg_id: &MessageID) {
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
            likes: UnorderedSet::new(
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

        self.accounts_stats.insert(account_id, &account_stat);

        account_stat
    }
    
    fn add_account_friends_storage(&mut self, account_id: &AccountId) -> UnorderedSet<AccountId> {
       let account_friends = UnorderedSet::new(
            StorageKeys::AccountFriends { 
                account_id: env::sha256(account_id.as_bytes()) 
            }
        );

        self.accounts_friends.insert(account_id, &account_friends);

        account_friends
    }
    
    
    
    // Execute call logic

    fn execute_add_message_to_post_call(&mut self, post_id: PostId, text: String) -> (PostId, U64) {
        let account_id = env::signer_account_id();
        
        let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
            self.add_post_storage(&post_id)
        });
        
        let msg_idx = post.messages.len();
        let msg = Message {
            account: account_id,
            parent_idx: None,
            payload: MessagePayload::Text { text },
            likes: UnorderedSet::new(
                StorageKeys::MessageLikes { 
                    post_id: env::sha256(post_id.as_bytes()),
                    msg_idx: msg_idx 
                }
            )
        };
        post.messages.push(&msg);
        self.posts.insert(&post_id, &post);

        (post_id, U64(msg_idx))
    }

    fn execute_add_message_to_message_call(&mut self, parent_msg_id: MessageId, text: String) -> (PostId, U64) {
        let account_id = env::signer_account_id();
        
        let mut post = self.posts.get(&parent_msg_id.post_id).expect("Post is not found");
        
        let msg_idx = post.messages.len();
        let msg = Message {
            account: account_id,
            parent_idx: Some(parent_msg_id.msg_idx),
            payload: MessagePayload::Text { text },
            likes: UnorderedSet::new(
                StorageKeys::MessageLikes {
                    post_id: env::sha256(parent_msg_id.post_id.as_bytes()),
                    msg_idx: msg_idx 
                }
            )
        };
        post.messages.push(&msg);
        self.posts.insert(&parent_msg_id.post_id, &post);

        (parent_msg_id.post_id, U64(msg_idx))
    }

    fn execute_add_friend_call(&mut self, friend_id: AccountId) {
        let account_id = env::signer_account_id();

        let mut account_friends = self.accounts_friends.get(&account_id).unwrap_or_else(|| {
            self.add_account_friends_storage(&account_id)
        });

        account_friends.insert(&friend_id);
    }
    
    fn execute_like_post_call(&mut self, post_id: PostId) {
        let account_id = env::signer_account_id();

        // Update post stats
        let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
            self.add_post_storage(&post_id)
        });
        post.likes.insert(&account_id);
        self.posts.insert(&post_id, &post);

        // Update account stats
        let mut accounts_stats = self.accounts_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });
        let like = AccountLike::PostLike { post_id };
        accounts_stats.recent_likes.insert(&like);
        self.accounts_stats.insert(&account_id, &accounts_stats);
    }

    fn execute_unlike_post_call(&mut self, post_id: PostId) {
        let account_id = env::signer_account_id();
        
        // Update post stats
        let mut post = self.posts.get(&post_id).expect("Post is not found");
        post.likes.remove(&account_id);                
        self.posts.insert(&post_id, &post);

        // Update account stats
        let mut accounts_stats = self.accounts_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });
        let like = AccountLike::PostLike { post_id };
        accounts_stats.recent_likes.remove(&like);
        self.accounts_stats.insert(&account_id, &accounts_stats);
    }

    fn execute_like_message_call(&mut self, msg_id: MessageId) {
        let account_id = env::signer_account_id();

        // Update message stats
        let mut post = self.posts.get(&msg_id.post_id).expect("Post is not found");
        let mut msg = post.messages.get(msg_id.msg_idx).expect("Message is not found");
        msg.likes.insert(&account_id);
        post.messages.replace(msg_id.msg_idx, &msg);
        self.posts.insert(&msg_id.post_id, &post);

        // Update account stats
        let mut accounts_stats = self.accounts_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });
        let like = AccountLike::MessageLike { msg_id };
        accounts_stats.recent_likes.insert(&like);
        self.accounts_stats.insert(&account_id, &accounts_stats);
    }

    fn execute_unlike_message_call(&mut self, msg_id: MessageId) {
        let account_id = env::signer_account_id();
        
        // Update message stats
        let mut post = self.posts.get(&msg_id.post_id).expect("Post is not found");
        let mut msg = post.messages.get(msg_id.msg_idx).expect("Message is not found");
        msg.likes.remove(&account_id);
        post.messages.replace(msg_id.msg_idx, &msg);
        self.posts.insert(&msg_id.post_id, &post);

        // Update account stats
        let mut accounts_stats = self.accounts_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });

        let like = AccountLike::MessageLike { msg_id };
        accounts_stats.recent_likes.remove(&like);
        self.accounts_stats.insert(&account_id, &accounts_stats);
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
    pub fn on_fee_collected(&mut self, call: Call) -> CallResult {

        if env::promise_results_count() != 1 {
            env::panic_str("Unexpected promise results count");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                match call {
                    Call::AddMessageToPost { post_id, text } => {
                        let (post_id, msg_idx) = self.execute_add_message_to_post_call(post_id, text);
                        CallResult::MessageToPostAdded { id: MessageID { post_id, msg_idx } }
                    },
                    Call::AddMessageToMessage { parent_msg_id, text } => {
                        let (post_id, msg_idx) = self.execute_add_message_to_message_call(parent_msg_id.into(), text);
                        CallResult::MessageToMessageAdded { id: MessageID { post_id, msg_idx } }
                    },
                    Call::AddFriend { friend_id } => {
                        self.execute_add_friend_call(friend_id);
                        CallResult::FriendAdded
                    },
                    Call::LikePost { post_id } => {
                        self.execute_like_post_call(post_id);
                        CallResult::PostLiked
                    },
                    Call::UnlikePost { post_id } => {
                        self.execute_unlike_post_call(post_id);
                        CallResult::PostUnliked
                    },
                    Call::LikeMessage { msg_id } => {
                        self.execute_like_message_call(msg_id.into());
                        CallResult::MessageLiked
                    },
                    Call::UnlikeMessage { msg_id } => {
                        self.execute_unlike_message_call(msg_id.into());
                        CallResult::MessageUnliked
                    },
                }
            },
            _ => env::panic_str("Fee was not charged"),
        }
    }
}