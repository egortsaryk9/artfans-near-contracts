use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, log, Balance, AccountId, Gas, Promise, PanicOnDefault, PromiseResult, StorageUsage, BorshStorageKey};
use near_sdk::json_types::{U128, U64, Base64VecU8};
use near_sdk::collections::{LookupMap, Vector, UnorderedSet, LazyOption};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json;
use near_sdk::serde_json::{Result, Value};
use std::convert::{From, TryFrom};

pub mod external;
pub use crate::external::*;

const MIN_ACCOUNT_ID_LEN : usize = 2;
const MIN_POST_ID_LEN : usize = 24;
const MIN_POST_MESSAGE_LEN : usize = 1;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    fee_ft: AccountId,
    custom_settings: CustomSettings,
    storage_usage_settings: StorageUsageSettings,
    posts_messages: LookupMap<PostId, Vector<Message>>,
    posts_likes: LookupMap<PostId, UnorderedSet<AccountId>>,
    posts_messages_likes: LookupMap<MessageId, UnorderedSet<AccountId>>,
    accounts_friends: LookupMap<AccountId, UnorderedSet<AccountId>>,
    accounts_profiles: LookupMap<AccountId, AccountProfile>,
    accounts_stats: LookupMap<AccountId, AccountStats>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    PostsMessages,
    PostMessages { post_id: Vec<u8> },
    PostsLikes,
    PostLikes { post_id: Vec<u8> },
    PostsMessagesLikes,
    PostMessageLikes { post_id: Vec<u8>, msg_idx: u64 },
    AccountsStats,
    AccountRecentLikes { account_id: Vec<u8> },
    AccountsFriends,
    AccountFriends { account_id: Vec<u8> },
    AccountsProfiles,
    AccountProfileImage { account_id: Vec<u8> },
}

type PostId = String;

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct MessageId {
    post_id: PostId,
    msg_idx: u64
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum MessagePayload {
    Text { text: String }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Message {
    account: AccountId,
    parent_idx: Option<u64>,
    payload: MessagePayload,
    timestamp: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountStats {
    recent_likes: Vec<AccountLike>
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum AccountLike {
    PostLike { post_id: PostId },
    MessageLike { msg_id: MessageId }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AccountProfile {
    json_metadata: String,
    image: LazyOption<Vec<u8>>
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct CustomSettings {
    account_recent_likes_limit: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageUsageSettings {
    min_message_size: StorageUsage,
    messages_collection_size: StorageUsage,
    min_post_like_size: StorageUsage,
    post_likes_collection_size: StorageUsage,
    min_message_like_size: StorageUsage,
    message_likes_collection_size: StorageUsage,
    min_account_friend_size: StorageUsage,
    account_friends_collection_size: StorageUsage,
    min_account_profile_size: StorageUsage,
    min_account_stat_like_size: StorageUsage,
    account_stat_likes_collection_size: StorageUsage
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


impl PartialEq for MessageId {
    fn eq(&self, other: &Self) -> bool {
        self.post_id == other.post_id && self.msg_idx == other.msg_idx
    }
}

impl Eq for MessageId {}

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
    UpdateProfile { profile: AccountProfileData }
}

#[derive(Serialize, Deserialize, Clone)]
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

impl From<&MessageID> for MessageId {
    fn from(v: &MessageID) -> Self {
        MessageId {
            post_id: v.post_id.clone(), 
            msg_idx: u64::from(v.msg_idx) 
        }
    }
}

impl From<MessageId> for MessageID {
    fn from(v: MessageId) -> Self {
        MessageID {
            post_id: v.post_id, 
            msg_idx: U64(v.msg_idx) 
        }
    }
}

impl From<&MessageId> for MessageID {
    fn from(v: &MessageId) -> Self {
        MessageID {
            post_id: v.post_id.clone(), 
            msg_idx: U64(v.msg_idx) 
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountProfileData {
    json_metadata: Option<String>,
    image: Option<Base64VecU8>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CustomSettingsData {
    account_recent_likes_limit: Option<u8>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MessageDTO {
    msg_idx: U64,
    parent_idx: Option<U64>,
    account: AccountId,
    text: Option<String>,
    timestamp: U64,
    likes_count: U64
}


#[near_bindgen]
impl Contract {

    #[init]
    pub fn new(owner: AccountId, fee_ft: AccountId, settings: CustomSettingsData) -> Self {
        if env::state_exists() == true {
            env::panic_str("Already initialized");
        }
        let mut this = Self {
            owner,
            fee_ft,
            custom_settings: CustomSettings {
              account_recent_likes_limit: match settings.account_recent_likes_limit {
                Some(account_recent_likes_limit) => account_recent_likes_limit,
                None => 0
              },
            }, 
            storage_usage_settings: StorageUsageSettings {
                min_message_size: 0,
                messages_collection_size: 0,
                min_post_like_size: 0,
                post_likes_collection_size: 0,
                min_message_like_size: 0,
                message_likes_collection_size: 0,
                min_account_friend_size: 0,
                account_friends_collection_size: 0,
                min_account_profile_size: 0,
                min_account_stat_like_size: 0,
                account_stat_likes_collection_size: 0
            },
            posts_messages: LookupMap::new(StorageKeys::PostsMessages),
            posts_likes: LookupMap::new(StorageKeys::PostsLikes),
            posts_messages_likes: LookupMap::new(StorageKeys::PostsMessagesLikes),
            accounts_friends: LookupMap::new(StorageKeys::AccountsFriends),
            accounts_profiles: LookupMap::new(StorageKeys::AccountsProfiles),
            accounts_stats: LookupMap::new(StorageKeys::AccountsStats)
        };

        this.update_storage_usage_settings();

        this
    }

    pub fn add_message_to_post(&mut self, post_id: PostId, text: String) -> Promise {
        let account_id = env::signer_account_id();
        self.assert_add_message_to_post_call(&post_id, &text);
        let fee = self.calc_add_message_to_post_fee(&account_id, &post_id, &text);
        self.collect_fee_and_execute_call(fee, Call::AddMessageToPost { post_id, text })
    }

    pub fn add_message_to_message(&mut self, parent_msg_id: MessageID, text: String) -> Promise {
        let account_id = env::signer_account_id();
        self.assert_add_message_to_message_call(&parent_msg_id, &text);
        let fee = self.calc_add_message_to_message_fee(&account_id, &parent_msg_id, &text);
        self.collect_fee_and_execute_call(fee, Call::AddMessageToMessage { parent_msg_id, text })
    }

    pub fn like_post(&mut self, post_id: PostId) -> Promise {
        let account_id = env::signer_account_id();
        self.assert_like_post_call(&post_id);
        let fee = self.calc_like_post_fee(&account_id, &post_id);
        self.collect_fee_and_execute_call(fee, Call::LikePost { post_id })
    }

    pub fn unlike_post(&mut self, post_id: PostId) -> Promise {
        self.assert_unlike_post_call(&post_id);
        self.collect_fee_and_execute_call(FIXED_FEE, Call::UnlikePost { post_id })
    }

    pub fn like_message(&mut self, msg_id: MessageID) -> Promise {
        let account_id = env::signer_account_id();
        self.assert_like_message_call(&msg_id);
        let fee = self.calc_like_message_fee(&account_id, &msg_id);
        self.collect_fee_and_execute_call(fee, Call::LikeMessage { msg_id })
    }

    pub fn unlike_message(&mut self, msg_id: MessageID) -> Promise {
        self.assert_unlike_message_call(&msg_id);
        self.collect_fee_and_execute_call(FIXED_FEE, Call::UnlikeMessage { msg_id })
    }

    pub fn add_friend(&mut self, friend_id: AccountId) -> Promise {
        self.assert_add_friend_call(&friend_id);
        self.collect_fee_and_execute_call(FIXED_FEE, Call::AddFriend { friend_id })
    }

    pub fn update_profile(&mut self, profile: AccountProfileData) -> Promise {
        self.assert_update_profile_call(&profile);
        self.collect_fee_and_execute_call(FIXED_FEE, Call::UpdateProfile { profile })
    }

    pub fn update_settings(&mut self, settings: CustomSettingsData) {
        self.assert_owner();
        if let Some(account_recent_likes_limit) = settings.account_recent_likes_limit {
            self.custom_settings.account_recent_likes_limit = account_recent_likes_limit;
        }
    }
    
    pub fn get_post_messages(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<MessageDTO> {
        if let Some(post_messages) = self.posts_messages.get(&post_id) {
            let from = u64::from(from_index);
            let lim = u64::from(limit);
            
            (from..std::cmp::min(from + lim, post_messages.len()))
                .map(|idx| {
                    let msg = post_messages.get(idx).unwrap();
                    match msg.payload {
                        MessagePayload::Text { text } => {
                            MessageDTO {
                                msg_idx: U64(idx),
                                parent_idx: match msg.parent_idx {
                                    Some(parent_idx) => Some(U64(parent_idx)),
                                    None => None
                                },
                                account: msg.account,
                                text: Some(text),
                                timestamp: U64(msg.timestamp),
                                likes_count: match self.posts_likes.get(&post_id) {
                                    Some(post_likes) => U64(post_likes.len()),
                                    None => U64(0)
                                }
                            }
                        }
                    }
                })
                .collect()
        } else {
            env::panic_str("Post is not found");
        }
    }

    pub fn get_post_message(&self, msg_id: MessageID) -> Option<MessageDTO> {
        if let Some(post_messages) = self.posts_messages.get(&msg_id.post_id) {
            let id : MessageId = msg_id.into();
            if let Some(msg) = post_messages.get(id.msg_idx) {
                match msg.payload {
                    MessagePayload::Text { text } => {
                        Some(MessageDTO {
                            msg_idx: U64(id.msg_idx),
                            parent_idx: match msg.parent_idx {
                                Some(parent_idx) => Some(U64(parent_idx)),
                                None => None
                            },
                            account: msg.account,
                            text: Some(text),
                            timestamp: U64(msg.timestamp),
                            likes_count: match self.posts_messages_likes.get(&id) {
                                Some(post_message_likes) => U64(post_message_likes.len()),
                                None => U64(0)
                            }
                        })
                    }
                }
            } else {
                env::panic_str("Message is not found");
            }
        } else {
            env::panic_str("Post is not found");
        }
    }

    pub fn get_post_likes(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<AccountId> {
        if let Some(post_likes) = self.posts_likes.get(&post_id) {
            use std::convert::TryFrom;
            if let (Ok(from), Ok(lim)) = (usize::try_from(u64::from(from_index)), usize::try_from(u64::from(limit))) {
                post_likes
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
        if let Some(post_message_like) = self.posts_messages_likes.get(&msg_id.into()) {
            use std::convert::TryFrom;
            if let (Ok(from), Ok(lim)) = (usize::try_from(u64::from(from_index)), usize::try_from(u64::from(limit))) {
                post_message_like
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
    
    pub fn get_account_last_likes(&self, account_id: AccountId, from_index: u8, limit: u8) -> Vec<(PostId, Option<U64>)> {
        if let Some(accounts_stats) = self.accounts_stats.get(&account_id) {
            accounts_stats.recent_likes
                .into_iter()
                .skip(usize::from(from_index))
                .take(usize::from(limit))
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

    pub fn get_profile(&self, account_id: AccountId) -> Option<AccountProfileData> {
        if let Some(account_profile) = self.accounts_profiles.get(&account_id) {
            Some(AccountProfileData {
              json_metadata: Some(account_profile.json_metadata),
              image: match account_profile.image.get() {
                  Some(vec) => Some(Base64VecU8::from(vec)),
                  None => None
              }
            })
        } else {
            None
        }
    }

    pub fn get_custom_settings(&self) -> CustomSettings {
        self.custom_settings.clone()
    }

    pub fn get_storage_settings(&self) -> StorageUsageSettings {
        self.storage_usage_settings.clone()
    }

}

// Private methods

#[near_bindgen]
impl Contract {

    // Assert incoming call

    fn assert_add_message_to_post_call(&self, post_id: &PostId, text: &String) {
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        };
        
        if text.len() < MIN_POST_MESSAGE_LEN {
            env::panic_str("'text' length is too small");
        };

        self.assert_post_id(post_id);
    }

    fn assert_add_message_to_message_call(&self, parent_msg_id: &MessageID, text: &String) {
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        };

        if text.len() < MIN_POST_MESSAGE_LEN {
            env::panic_str("'text' length is too small");
        };

        self.assert_message_id(parent_msg_id);

        let post_id = &parent_msg_id.post_id;
        let msg_idx: u64 = parent_msg_id.msg_idx.into();
        
        if let Some(post_messages) = self.posts_messages.get(post_id) {
            if !post_messages.get(msg_idx).is_some() {
                env::panic_str("Parent message does not exist");
            };
        } else {
            env::panic_str("Post does not exist");
        };
    }

    fn assert_like_post_call(&self, post_id: &PostId) {
        let account_id = env::signer_account_id();

        self.assert_post_id(post_id);

        if let Some(post_likes) = self.posts_likes.get(post_id) {
            if post_likes.contains(&account_id) {
                env::panic_str("Post is liked already");
            };
        };
    }

    fn assert_unlike_post_call(&self, post_id: &PostId) {
        let account_id = env::signer_account_id();

        self.assert_post_id(post_id);

        if let Some(post_likes) = self.posts_likes.get(post_id) {
            if !post_likes.contains(&account_id) {
                env::panic_str("Post is not liked");
            };
        } else {
            env::panic_str("Post is not liked");
        };
    }

    fn assert_like_message_call(&self, msg_id: &MessageID) {
        let account_id = env::signer_account_id();
        
        self.assert_message_id(msg_id);

        if let Some(post_message_likes) = self.posts_messages_likes.get(&msg_id.into()) {
            if post_message_likes.contains(&account_id) {
                env::panic_str("Message is liked already");
            };
        };
    }

    fn assert_unlike_message_call(&self, msg_id: &MessageID) {
        let account_id = env::signer_account_id();
        
        self.assert_message_id(msg_id);

        if let Some(post_message_likes) = self.posts_messages_likes.get(&msg_id.into()) {
            if !post_message_likes.contains(&account_id) {
                env::panic_str("Message is not liked");
            };
        } else {
            env::panic_str("Message is not liked");
        };
    }

    fn assert_add_friend_call(&self, friend_id: &AccountId) {
        let account_id = env::signer_account_id();

        if let Some(account_friends) = self.accounts_friends.get(&account_id) {
            if account_friends.contains(friend_id) {
                env::panic_str("Friend is added already");
            };
        };
    }

    fn assert_update_profile_call(&self, profile: &AccountProfileData) {
        if let Some(json_metadata) = &profile.json_metadata {
            let result : Result<Value> = serde_json::from_str(json_metadata);
            if result.is_err() {
                env::panic_str("Metadata is not a valid json string");
            };
        };
    }
    
    fn assert_post_id(&self, post_id: &PostId) {
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        };

        if post_id.len() < MIN_POST_ID_LEN {
            env::panic_str("'post_id' length is too small");
        }
    }

    fn assert_message_id(&self, msg_id: &MessageID) {
        let post_id = &msg_id.post_id;
        self.assert_post_id(post_id);
    }

    fn calc_add_message_to_post_fee(&mut self, account_id: &AccountId, post_id: &PostId, text: &String) -> u128 {
        let account_extra_bytes = u64::try_from(account_id.as_str().len() - MIN_ACCOUNT_ID_LEN).unwrap();
        let post_id_extra_bytes = u64::try_from(post_id.len() - MIN_POST_ID_LEN).unwrap();
        let text_extra_bytes = u64::try_from(text.len() - MIN_POST_MESSAGE_LEN).unwrap();
        let collection_bytes = match self.posts_messages.contains_key(post_id) {
            false => self.storage_usage_settings.messages_collection_size,
            true => 0u64
        };

        let storage_size = self.storage_usage_settings.min_message_size 
          + account_extra_bytes 
          + text_extra_bytes 
          + post_id_extra_bytes
          + collection_bytes;

        let storage_fee = Balance::from(storage_size) * env::storage_byte_cost();
        storage_fee.into()
    }

    fn calc_add_message_to_message_fee(&mut self, account_id: &AccountId, msg_id: &MessageID, text: &String) -> u128 {
        let account_extra_bytes = u64::try_from(account_id.as_str().len() - MIN_ACCOUNT_ID_LEN).unwrap();
        let post_id_extra_bytes = u64::try_from(msg_id.post_id.len() - MIN_POST_ID_LEN).unwrap();
        let text_extra_bytes = u64::try_from(text.len() - MIN_POST_MESSAGE_LEN).unwrap();
        let msg_idx_bytes = 8u64;
        
        let storage_size = self.storage_usage_settings.min_message_size 
          + account_extra_bytes 
          + text_extra_bytes 
          + post_id_extra_bytes
          + msg_idx_bytes;

        let storage_fee = Balance::from(storage_size) * env::storage_byte_cost();
        storage_fee.into()
    }

    fn calc_like_post_fee(&mut self, account_id: &AccountId, post_id: &PostId) -> u128 {
        let account_extra_bytes = u64::try_from(account_id.as_str().len() - MIN_ACCOUNT_ID_LEN).unwrap();
        let post_id_extra_bytes = u64::try_from(post_id.len() - MIN_POST_ID_LEN).unwrap();
        let collection_bytes = match self.posts_likes.contains_key(post_id) {
            false => self.storage_usage_settings.post_likes_collection_size,
            true => 0u64
        };
        
        let storage_size = self.storage_usage_settings.min_post_like_size 
          + account_extra_bytes 
          + post_id_extra_bytes
          + collection_bytes;

        let storage_fee = Balance::from(storage_size) * env::storage_byte_cost();
        storage_fee.into()
    }

    fn calc_like_message_fee(&mut self, account_id: &AccountId, msg_id: &MessageID) -> u128 {
        let account_extra_bytes = u64::try_from(account_id.as_str().len() - MIN_ACCOUNT_ID_LEN).unwrap();
        let post_id_extra_bytes = u64::try_from(msg_id.post_id.len() - MIN_POST_ID_LEN).unwrap();
        let collection_bytes = match self.posts_messages_likes.contains_key(&msg_id.clone().into()) {
            false => self.storage_usage_settings.message_likes_collection_size,
            true => 0u64
        };
        
        let storage_size = self.storage_usage_settings.min_message_like_size 
          + account_extra_bytes 
          + post_id_extra_bytes
          + collection_bytes;

        let storage_fee = Balance::from(storage_size) * env::storage_byte_cost();
        storage_fee.into()
    }
    
    // Execute call logic

    fn execute_add_message_to_post_call(&mut self, account_id: AccountId, post_id: PostId, text: String) -> MessageID {
        let mut post_messages = self.posts_messages.get(&post_id).unwrap_or_else(|| {
            self.add_post_messages_storage(&post_id)
        });
        
        let msg_idx = post_messages.len();
        let msg = Message {
            account: account_id,
            parent_idx: None,
            payload: MessagePayload::Text { text },
            timestamp: env::block_timestamp()
        };

        post_messages.push(&msg);
        self.posts_messages.insert(&post_id, &post_messages);

        let msg_id = MessageId { post_id, msg_idx };
        msg_id.into()
    }

    fn execute_add_message_to_message_call(&mut self, account_id: AccountId, parent_msg_id: MessageId, text: String) -> MessageID {
        let mut post_messages = self.posts_messages.get(&parent_msg_id.post_id).expect("Post is not found");
        
        let msg_idx = post_messages.len();
        let msg = Message {
            account: account_id,
            parent_idx: Some(parent_msg_id.msg_idx),
            payload: MessagePayload::Text { text },
            timestamp: env::block_timestamp()
        };
        post_messages.push(&msg);
        self.posts_messages.insert(&parent_msg_id.post_id, &post_messages);

        let msg_id = MessageId { post_id: parent_msg_id.post_id, msg_idx };
        msg_id.into()
    }
    
    fn execute_like_post_call(&mut self, account_id: AccountId, post_id: PostId) -> AccountLike {
        let mut post_likes = self.posts_likes.get(&post_id).unwrap_or_else(|| {
            self.add_post_likes_storage(&post_id)
        });
        post_likes.insert(&account_id);
        self.posts_likes.insert(&post_id, &post_likes);

        AccountLike::PostLike { post_id }
    }

    fn execute_unlike_post_call(&mut self, account_id: AccountId, post_id: PostId) -> AccountLike {
        let mut post_likes = self.posts_likes.get(&post_id).expect("Post like is not found");
        post_likes.remove(&account_id);                
        self.posts_likes.insert(&post_id, &post_likes);

        AccountLike::PostLike { post_id }
    }

    fn execute_like_message_call(&mut self, account_id: AccountId, msg_id: MessageId) -> AccountLike {
        let mut post_message_likes = self.posts_messages_likes.get(&msg_id).unwrap_or_else(|| {
            self.add_post_message_likes_storage(&msg_id)
        });
        post_message_likes.insert(&account_id);
        self.posts_messages_likes.insert(&msg_id, &post_message_likes);

        AccountLike::MessageLike { msg_id }
    }

    fn execute_unlike_message_call(&mut self, account_id: AccountId, msg_id: MessageId) -> AccountLike  {
        let mut post_message_likes = self.posts_messages_likes.get(&msg_id).expect("Message like is not found");
        post_message_likes.remove(&account_id);
        self.posts_messages_likes.insert(&msg_id, &post_message_likes);

        AccountLike::MessageLike { msg_id }
    }

    fn execute_add_friend_call(&mut self, account_id: AccountId, friend_id: AccountId) {
        let mut account_friends = self.accounts_friends.get(&account_id).unwrap_or_else(|| {
            self.add_account_friends_storage(&account_id)
        });

        account_friends.insert(&friend_id);
        self.accounts_friends.insert(&account_id, &account_friends);
    }

    fn execute_update_profile_call(&mut self, account_id: AccountId, json_metadata: Option<String>, image: Option<Vec<u8>>) {
        let mut account_profile = self.accounts_profiles.get(&account_id).unwrap_or_else(|| {
            self.add_account_profile_storage(&account_id)
        });

        if let Some(metadata) = json_metadata {
            account_profile.json_metadata = metadata;
        };

        if let Some(bytes) = image {
            account_profile.image.set(&bytes);
        };

        self.accounts_profiles.insert(&account_id, &account_profile);
    }

    fn add_like_to_account_likes_stat(&mut self, account_id: AccountId, like: AccountLike) {
        let mut account_stats = self.accounts_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });

        let account_recent_likes_limit = usize::from(self.custom_settings.account_recent_likes_limit);

        let updated_account_stats = if account_stats.recent_likes.len() > 0 && account_recent_likes_limit == 0 {
            account_stats.recent_likes.clear();
            account_stats
        } else {
            if account_stats.recent_likes.len() > account_recent_likes_limit {
                let skip = account_stats.recent_likes.len() - account_recent_likes_limit;
                account_stats.recent_likes = account_stats.recent_likes.into_iter().skip(skip + 1).collect();
                account_stats.recent_likes.push(like);
                account_stats
            } else if account_stats.recent_likes.len() == account_recent_likes_limit {
                let skip = 1;
                account_stats.recent_likes = account_stats.recent_likes.into_iter().skip(skip).collect();
                account_stats.recent_likes.push(like);
                account_stats
            } else {
                account_stats.recent_likes.push(like);
                account_stats
            }
        };

        self.accounts_stats.insert(&account_id, &updated_account_stats);
    }

    fn remove_like_from_account_likes_stat(&mut self, account_id: AccountId, like: AccountLike) {
        let mut account_stats = self.accounts_stats.get(&account_id).unwrap_or_else(|| {
            self.add_account_stat_storage(&account_id)
        });

        let updated_account_stats = if let Some(idx) = account_stats.recent_likes.iter().position(|l| l == &like) {
            account_stats.recent_likes.remove(idx);
            account_stats
        } else {
            account_stats
        };

        self.accounts_stats.insert(&account_id, &updated_account_stats);
    }


    // Add storage collections

    fn add_post_messages_storage(&mut self, post_id: &PostId) -> Vector<Message> {
        let post_messages = Vector::new(
            StorageKeys::PostMessages { 
                post_id: env::sha256(post_id.as_bytes()) 
            }
        );

        self.posts_messages.insert(post_id, &post_messages);
        post_messages
    }

    fn remove_post_messages_storage(&mut self, post_id: &PostId) {
        let mut post_messages = self.posts_messages.get(&post_id).expect("Post messages storage is not found");
        post_messages.clear();
        self.posts_messages.remove(&post_id);
    }

    fn add_post_likes_storage(&mut self, post_id: &PostId) -> UnorderedSet<AccountId> {
        let post_likes = UnorderedSet::new(
            StorageKeys::PostLikes {
                post_id: env::sha256(post_id.as_bytes())
            }
        );

        self.posts_likes.insert(post_id, &post_likes);
        post_likes
    }

    fn remove_post_likes_storage(&mut self, post_id: &PostId) {
        let mut post_likes = self.posts_likes.get(&post_id).expect("Post likes storage is not found");
        post_likes.clear();
        self.posts_likes.remove(&post_id);
    }

    fn add_post_message_likes_storage(&mut self, msg_id: &MessageId) -> UnorderedSet<AccountId> {
        let post_message_likes = UnorderedSet::new(
            StorageKeys::PostMessageLikes {
                post_id: env::sha256(msg_id.post_id.as_bytes()),
                msg_idx: msg_id.msg_idx 
            }
        );

        self.posts_messages_likes.insert(&msg_id, &post_message_likes);
        post_message_likes
    }

    fn remove_post_message_likes_storage(&mut self, msg_id: &MessageId) {
        let mut post_message_likes = self.posts_messages_likes.get(&msg_id).expect("Messages likes storage is not found");
        post_message_likes.clear();
        self.posts_messages_likes.remove(&msg_id);
    }

    fn add_account_stat_storage(&mut self, account_id: &AccountId) -> AccountStats {
        let account_stat = AccountStats {
            recent_likes: Vec::new()
        };

        self.accounts_stats.insert(account_id, &account_stat);
        account_stat
    }

    fn remove_account_stat_storage(&mut self, account_id: &AccountId) {
        let mut account_stat = self.accounts_stats.get(&account_id).expect("Account stats storage is not found");
        account_stat.recent_likes.clear();
        self.accounts_stats.remove(&account_id);
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

    fn remove_account_friends_storage(&mut self, account_id: &AccountId) {
        let mut account_friends = self.accounts_friends.get(&account_id).expect("Account friends storage is not found");
        account_friends.clear();
        self.accounts_friends.remove(&account_id);
    }

    fn add_account_profile_storage(&mut self, account_id: &AccountId) -> AccountProfile {
        let account_profile = AccountProfile {
            json_metadata: "".to_string(),
            image: LazyOption::new(
                StorageKeys::AccountProfileImage { 
                    account_id: env::sha256(account_id.as_bytes()),
                },
                None
            )
        };
        
        self.accounts_profiles.insert(account_id, &account_profile);
        account_profile
    }

    fn remove_account_profile_storage(&mut self, account_id: &AccountId) {
        let mut account_profile = self.accounts_profiles.get(&account_id).expect("Account profile storage is not found");
        account_profile.image.remove();
        self.accounts_profiles.remove(&account_id);
    }


    // Measure post storage usage

    fn update_storage_usage_settings(&mut self) {
        self.measure_message_storage_usage();
        self.measure_post_likes_storage_usage();
        self.measure_message_likes_storage_usage();
        self.measure_account_friends_storage_usage();
        self.measure_account_profile_storage_usage();
        self.measure_account_likes_stat_storage_usage();
    }

    fn measure_message_storage_usage(&mut self) {
        let account_id = AccountId::new_unchecked("a".repeat(MIN_ACCOUNT_ID_LEN));
        let post_id = String::from("a".repeat(MIN_POST_ID_LEN));
        let text = String::from("a".repeat(MIN_POST_MESSAGE_LEN));

        let initial_storage_usage = env::storage_usage();

        self.execute_add_message_to_post_call(
            account_id.clone(), 
            post_id.clone(), 
            text.clone()
        );
        let after_first_message_storage_usage = env::storage_usage();
        
        self.execute_add_message_to_post_call(
            account_id, 
            post_id.clone(),
            text
        );
        let after_second_message_storage_usage = env::storage_usage();
      
        self.storage_usage_settings.min_message_size = after_second_message_storage_usage - after_first_message_storage_usage;
        self.storage_usage_settings.messages_collection_size = after_first_message_storage_usage - initial_storage_usage - self.storage_usage_settings.min_message_size;

        self.remove_post_messages_storage(&post_id);

        let final_storage_usage = env::storage_usage();
        if initial_storage_usage != final_storage_usage {
            env::panic_str("Measurement of message storage aborted due to data leak");
        }
    }

    fn measure_post_likes_storage_usage(&mut self) {
        let post_id = String::from("a".repeat(MIN_POST_ID_LEN));
        let account_1 = AccountId::new_unchecked("a".repeat(MIN_ACCOUNT_ID_LEN));
        let account_2 = AccountId::new_unchecked("b".repeat(MIN_ACCOUNT_ID_LEN));

        let initial_storage_usage = env::storage_usage();

        self.execute_like_post_call(
            account_1.clone(), 
            post_id.clone()
        );
        let after_first_post_like_storage_usage = env::storage_usage();

        self.execute_like_post_call(
            account_2.clone(), 
            post_id.clone()
        );
        let after_second_post_like_storage_usage = env::storage_usage();

        self.storage_usage_settings.min_post_like_size = after_second_post_like_storage_usage - after_first_post_like_storage_usage;
        self.storage_usage_settings.post_likes_collection_size = after_first_post_like_storage_usage - initial_storage_usage - self.storage_usage_settings.min_post_like_size;

        self.remove_post_likes_storage(&post_id);

        let final_storage_usage = env::storage_usage();
        if initial_storage_usage != final_storage_usage {
            env::panic_str("Measurement of post likes storage aborted due to data leak");
        }
    }

    fn measure_message_likes_storage_usage(&mut self) {
        let msg_id = MessageId { post_id: String::from("a".repeat(MIN_POST_ID_LEN)), msg_idx: 1 };
        let account_1 = AccountId::new_unchecked("a".repeat(MIN_ACCOUNT_ID_LEN));
        let account_2 = AccountId::new_unchecked("b".repeat(MIN_ACCOUNT_ID_LEN));

        let initial_storage_usage = env::storage_usage();

        self.execute_like_message_call(
            account_1.clone(), 
            msg_id.clone()
        );
        let after_first_message_like_storage_usage = env::storage_usage();

        self.execute_like_message_call(
            account_2.clone(), 
            msg_id.clone()
        );
        let after_second_message_like_storage_usage = env::storage_usage();

        self.storage_usage_settings.min_message_like_size = after_second_message_like_storage_usage - after_first_message_like_storage_usage;
        self.storage_usage_settings.message_likes_collection_size = after_first_message_like_storage_usage - initial_storage_usage - self.storage_usage_settings.min_message_like_size;

        self.remove_post_message_likes_storage(&msg_id);

        let final_storage_usage = env::storage_usage();
        if initial_storage_usage != final_storage_usage {
            env::panic_str("Measurement of message likes storage aborted due to data leak");
        }
    }

    fn measure_account_likes_stat_storage_usage(&mut self) {
        let account_id = AccountId::new_unchecked("a".repeat(MIN_ACCOUNT_ID_LEN));

        let initial_storage_usage = env::storage_usage();

        self.add_like_to_account_likes_stat(
            account_id.clone(), 
            AccountLike::PostLike { post_id: String::from("a".repeat(MIN_POST_ID_LEN)) }
        );
        let after_first_account_like_storage_usage = env::storage_usage();

        self.add_like_to_account_likes_stat(
            account_id.clone(), 
            AccountLike::PostLike { post_id: String::from("b".repeat(MIN_POST_ID_LEN)) }
        );
        let after_second_account_like_storage_usage = env::storage_usage();

        self.storage_usage_settings.min_account_stat_like_size = after_second_account_like_storage_usage - after_first_account_like_storage_usage;
        self.storage_usage_settings.account_stat_likes_collection_size = after_first_account_like_storage_usage - initial_storage_usage - self.storage_usage_settings.min_account_stat_like_size;

        self.remove_account_stat_storage(&account_id);

        let final_storage_usage = env::storage_usage();
        if initial_storage_usage != final_storage_usage {
            env::panic_str("Measurement of account stat likes storage aborted due to data leak");
        }
    }

    fn measure_account_friends_storage_usage(&mut self) {
        let account_id = AccountId::new_unchecked("a".repeat(MIN_ACCOUNT_ID_LEN));

        let initial_storage_usage = env::storage_usage();

        self.execute_add_friend_call(
            account_id.clone(),
            AccountId::new_unchecked("b".repeat(MIN_ACCOUNT_ID_LEN))
        );
        let after_first_friend_storage_usage = env::storage_usage();

        self.execute_add_friend_call(
            account_id.clone(),
            AccountId::new_unchecked("c".repeat(MIN_ACCOUNT_ID_LEN))
        );
        let after_second_friend_storage_usage = env::storage_usage();

        self.storage_usage_settings.min_account_friend_size = after_second_friend_storage_usage - after_first_friend_storage_usage;
        self.storage_usage_settings.account_friends_collection_size = after_first_friend_storage_usage - initial_storage_usage - self.storage_usage_settings.min_account_friend_size;

        self.remove_account_friends_storage(&account_id);

        let final_storage_usage = env::storage_usage();
        if initial_storage_usage != final_storage_usage {
            env::panic_str("Measurement of account friends storage aborted due to data leak");
        }
    }

    fn measure_account_profile_storage_usage(&mut self) {
        let account_id = AccountId::new_unchecked("a".repeat(MIN_ACCOUNT_ID_LEN));

        let initial_storage_usage = env::storage_usage();

        self.execute_update_profile_call(
            account_id.clone(),
            Some(String::from("")), 
            Some(Vec::new())
        );
        let after_profile_update_storage_usage = env::storage_usage();

        self.storage_usage_settings.min_account_profile_size = after_profile_update_storage_usage - initial_storage_usage;

        self.remove_account_profile_storage(&account_id);

        let final_storage_usage = env::storage_usage();
        if initial_storage_usage != final_storage_usage {
            env::panic_str("Measurement of account profile storage aborted due to data leak");
        }
    }


    fn collect_fee_and_execute_call(&mut self, fee: u128, call: Call) -> Promise {
        ext_ft::ext(self.fee_ft.clone())
            .with_static_gas(Gas(5*TGAS))
            .ft_collect_fee(U128::from(fee))
                .then(
                    ext_self::ext(env::current_account_id())
                    .with_static_gas(Gas(5*TGAS))
                    .on_fee_collected(call)
                )
    }


    #[private]
    pub fn on_fee_collected(&mut self, call: Call) -> Option<String> {

        if env::promise_results_count() != 1 {
            env::panic_str("Unexpected promise results count");
        }

        let account_id = env::signer_account_id();

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                match call {
                    Call::AddMessageToPost { post_id, text } => {
                        let msg_id = self.execute_add_message_to_post_call(account_id, post_id, text);
                        serde_json::to_string(&msg_id).ok()
                    },
                    Call::AddMessageToMessage { parent_msg_id, text } => {
                        let msg_id = self.execute_add_message_to_message_call(account_id, parent_msg_id.into(), text);
                        serde_json::to_string(&msg_id).ok()
                    },
                    Call::LikePost { post_id } => {
                        let like = self.execute_like_post_call(account_id.clone(), post_id);
                        self.add_like_to_account_likes_stat(account_id, like);
                        None
                    },
                    Call::UnlikePost { post_id } => {
                        let like = self.execute_unlike_post_call(account_id.clone(), post_id);
                        self.remove_like_from_account_likes_stat(account_id, like);
                        None
                    },
                    Call::LikeMessage { msg_id } => {
                        let like = self.execute_like_message_call(account_id.clone(), msg_id.into());
                        self.add_like_to_account_likes_stat(account_id, like);
                        None
                    },
                    Call::UnlikeMessage { msg_id } => {
                        let like = self.execute_unlike_message_call(account_id.clone(), msg_id.into());
                        self.remove_like_from_account_likes_stat(account_id, like);
                        None
                    },
                    Call::AddFriend { friend_id } => {
                        self.execute_add_friend_call(account_id, friend_id);
                        None
                    },
                    Call::UpdateProfile { profile } => {
                        let image: Option<Vec<u8>> = match profile.image {
                            Some(vec) => Some(vec.into()),
                            None => None
                        };
                        self.execute_update_profile_call(account_id, profile.json_metadata, image);
                        None
                    },
                }
            },
            _ => env::panic_str("Fee was not charged"),
        }
    }
}

pub trait Ownable {
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.get_owner(),
            "This operation is restricted to the contract owner."
        );
    }
    fn get_owner(&self) -> AccountId;
    fn set_owner(&mut self, owner: AccountId);
}

#[near_bindgen]
impl Ownable for Contract {
    fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    fn set_owner(&mut self, owner: AccountId) {
        self.assert_owner();
        self.owner = owner;
    }
}