use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, log, AccountId, Gas, Promise, PanicOnDefault, PromiseResult, Balance};
use near_sdk::json_types::{U128, U64, Base64VecU8};
use near_sdk::collections::{LookupMap, TreeMap, LookupSet, Vector, UnorderedSet, UnorderedMap, LazyOption};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json;
use near_sdk::serde_json::{Result, Value};
use near_sdk::BorshStorageKey;
use std::convert::{From, TryFrom};

pub mod external;
pub use crate::external::*;

const FEE_TO_REPLACE : Balance = 1;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    fee_ft: AccountId,
    settings: Settings,
    // posts: UnorderedMap<u8, u8>,
    posts: Vector<u16>,
    accounts_friends: LookupMap<AccountId, UnorderedSet<AccountId>>,
    accounts_profiles: LookupMap<AccountId, AccountProfile>,
    accounts_stats: LookupMap<AccountId, AccountStats>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Posts,
    Messages /*{ post_id: Vec<u8> }*/,
    PostLikes { post_id: Vec<u8> },
    MessageLikes { post_id: Vec<u8>, msg_idx: u64 },
    AccountsStats,
    AccountRecentLikes { account_id: Vec<u8> },
    AccountsFriends,
    AccountFriends { account_id: Vec<u8> },
    AccountsProfiles,
    AccountProfileImage { account_id: Vec<u8> },
}

// type PostId = String;
type PostId = u8;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct MessageId {
    post_id: PostId,
    msg_idx: u64
}

// #[derive(BorshDeserialize, BorshSerialize)]
// pub struct Post {
//     messages: Vector<Message>,
//     // likes: UnorderedSet<AccountId>
// }

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
    // likes: UnorderedSet<AccountId>
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
pub struct Settings {
    account_recent_likes_limit: u8
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

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Call {
    AddMessageToPost { post_id: u8, text: u16 },
    AddMessageToMessage { parent_msg_id: MessageID, text: String },
    AddFriend { friend_id: AccountId },
    LikePost { post_id: PostId },
    UnlikePost { post_id: PostId },
    LikeMessage { msg_id: MessageID },
    UnlikeMessage { msg_id: MessageID },
    UpdateProfile { profile: AccountProfileData }
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
    MessageUnliked,
    ProfileUpdated
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
pub struct AccountProfileData {
    json_metadata: Option<String>,
    image: Option<Base64VecU8>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MessageDTO {
    msg_idx: U64,
    parent_idx: Option<U64>,
    account: AccountId,
    text: Option<String>,
    timestamp: U64,
    // likes_count: U64
}


#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner: AccountId, fee_ft: AccountId, settings: Settings) -> Self {
        if env::state_exists() == true {
            env::panic_str("Already initialized");
        }
        Self {
            owner,
            fee_ft,
            settings,
            // posts: UnorderedMap::new(b'm'),
            posts: Vector::new(b'm'),
            accounts_friends: LookupMap::new(StorageKeys::AccountsFriends),
            accounts_profiles: LookupMap::new(StorageKeys::AccountsProfiles),
            accounts_stats: LookupMap::new(StorageKeys::AccountsStats)
        }
    }

    fn calc_message_to_post_fee(&self, post_id: &PostId, text: &u8) -> Balance {
        // add_message_to_post
        // let post_id_bytes_count = post_id.len();
        // log!("post_id_bytes_count {}", &post_id_bytes_count);
        let text_bytes_count = 1usize; //text.len();
        log!("text_bytes_count {}", &text_bytes_count);

        let message_storage_prefix_bytes_count = 32usize; // sha256
        let message_storage_separator_bytes_count = "::".len();
        let account_id_bytes_count = env::signer_account_id().as_bytes().len();
        let parent_idx_bytes_count = 1usize; // None as 0
        let payload_bytes_count = (1 as usize) + text_bytes_count; // enum variant + text
        let timestamp_bytes_count = 8usize; // u64

        // let likes_storage_separator_bytes_count = "::".len();
        // let likes_storage_prefix_bytes_count = 32usize + 8usize; // sha256 + u64


        let total_bytes_count = 
            // post_id_bytes_count 
          text_bytes_count 
          + message_storage_prefix_bytes_count 
          + message_storage_separator_bytes_count
          + account_id_bytes_count
          + parent_idx_bytes_count
          + payload_bytes_count
          + timestamp_bytes_count;
          // + likes_storage_separator_bytes_count
          // + likes_storage_prefix_bytes_count
          // + account_id_bytes_count;

        log!("total_bytes_count {}", &total_bytes_count);

        let t = u128::try_from(total_bytes_count).unwrap();
        let fee = Balance::from(t) * env::storage_byte_cost();
        
        log!("fee {}", &fee);

        fee
    }



    //  fn execute_add_message_to_post_call(&mut self, post_id: PostId, text: String) -> (PostId, U64) {
    //     let account_id = env::signer_account_id();
        

    //     let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
    //         self.add_post_storage(&post_id)
    //     });
        
    //     let msg_idx = post.messages.len();
    //     let msg = Message {
    //         account: account_id,
    //         parent_idx: None,
    //         payload: MessagePayload::Text { text },
    //         timestamp: env::block_timestamp(),
    //         likes: UnorderedSet::new(
    //             StorageKeys::MessageLikes { 
    //                 post_id: env::sha256(post_id.as_bytes()),
    //                 msg_idx: msg_idx 
    //             }
    //         )
    //     };
    //     post.messages.push(&msg);
    //     self.posts.insert(&post_id, &post);

    //     let after_post_storage_usage = env::storage_usage();
    //     log!("after_post_storage_usage {}", &after_post_storage_usage);

    //     let diff_storage_usage = env::storage_usage() - before_post_storage_usage;
    //     log!("diff_storage_usage {}", &diff_storage_usage);

    //     (post_id, U64(msg_idx))
    // }

    pub fn add_message_to_post(&mut self, post_id: u8, text: u8) -> Promise {
        let initial_storage_usage = env::storage_usage();
        log!("initial_storage_usage {}", &initial_storage_usage);

        self.assert_add_message_to_post_call(&post_id, &text);
        let fee = self.calc_message_to_post_fee(&post_id, &text);
        self.collect_fee_and_execute_call(fee, Call::AddMessageToPost { post_id, text: 1u16 })
    }

    pub fn add_message_to_message(&mut self, parent_msg_id: MessageID, text: String) -> Promise {
        self.assert_add_message_to_message_call(&parent_msg_id, &text);
        self.collect_fee_and_execute_call(FEE_TO_REPLACE, Call::AddMessageToMessage { parent_msg_id, text })
    }

    pub fn like_post(&mut self, post_id: PostId) -> Promise {
        // self.assert_like_post_call(&post_id);
        self.collect_fee_and_execute_call(FEE_TO_REPLACE, Call::LikePost { post_id })
    }

    pub fn unlike_post(&mut self, post_id: PostId) -> Promise {
        // self.assert_unlike_post_call(&post_id);
        self.collect_fee_and_execute_call(FEE_TO_REPLACE, Call::UnlikePost { post_id })
    }

    pub fn like_message(&mut self, msg_id: MessageID) -> Promise {
        // self.assert_like_message_call(&msg_id);
        self.collect_fee_and_execute_call(FEE_TO_REPLACE, Call::LikeMessage { msg_id })
    }

    pub fn unlike_message(&mut self, msg_id: MessageID) -> Promise {
        // self.assert_unlike_message_call(&msg_id);
        self.collect_fee_and_execute_call(FEE_TO_REPLACE, Call::UnlikeMessage { msg_id })
    }

    pub fn add_friend(&mut self, friend_id: AccountId) -> Promise {
        self.assert_add_friend_call(&friend_id);
        self.collect_fee_and_execute_call(FEE_TO_REPLACE, Call::AddFriend { friend_id })
    }

    pub fn update_profile(&mut self, profile: AccountProfileData) -> Promise {
        self.assert_update_profile_call(&profile);
        self.collect_fee_and_execute_call(FEE_TO_REPLACE, 
          Call::UpdateProfile { profile })
    }

    pub fn update_settings(&mut self, settings: Settings) {
        self.assert_owner();
        self.settings = settings;
    }
    
    // pub fn get_post_messages(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<MessageDTO> {
    //     if let Some(post) = self.posts.get(&post_id) {
    //         let from = u64::from(from_index);
    //         let lim = u64::from(limit);
    //         (from..std::cmp::min(from + lim, post.messages.len()))
    //             .map(|idx| {
    //                 let msg = post.messages.get(idx).unwrap();
    //                 match msg.payload {
    //                     MessagePayload::Text { text } => {
    //                         MessageDTO {
    //                             msg_idx: U64(idx),
    //                             parent_idx: match msg.parent_idx {
    //                                 Some(parent_idx) => Some(U64(parent_idx)),
    //                                 None => None
    //                             },
    //                             account: msg.account,
    //                             text: Some(text),
    //                             timestamp: U64(msg.timestamp),
    //                             // likes_count: U64(msg.likes.len())
    //                         }
    //                     }
    //                 }
    //             })
    //             .collect()
    //     } else {
    //         env::panic_str("Post is not found");
    //     }
    // }

    // pub fn get_post_message(&self, msg_id: MessageID) -> Option<MessageDTO> {
    //     if let Some(post) = self.posts.get(&msg_id.post_id) {
    //         let idx = u64::from(msg_id.msg_idx);
    //         if let Some(msg) = post.messages.get(idx) {
    //             match msg.payload {
    //                 MessagePayload::Text { text } => {
    //                     Some(MessageDTO {
    //                         msg_idx: U64(idx),
    //                         parent_idx: match msg.parent_idx {
    //                             Some(parent_idx) => Some(U64(parent_idx)),
    //                             None => None
    //                         },
    //                         account: msg.account,
    //                         text: Some(text),
    //                         timestamp: U64(msg.timestamp),
    //                         // likes_count: U64(msg.likes.len())
    //                     })
    //                 }
    //             }
    //         } else {
    //             env::panic_str("Message is not found");
    //         }
    //     } else {
    //         env::panic_str("Post is not found");
    //     }
    // }

    // pub fn get_post_likes(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<AccountId> {
    //     if let Some(post) = self.posts.get(&post_id) {
    //         use std::convert::TryFrom;
    //         if let (Ok(from), Ok(lim)) = (usize::try_from(u64::from(from_index)), usize::try_from(u64::from(limit))) {
    //             post.likes
    //                 .iter()
    //                 .skip(from)
    //                 .take(lim)
    //                 .collect()
    //         } else {
    //             env::panic_str("'usize' conversion failed");
    //         }
    //     } else {
    //         env::panic_str("Post is not found");
    //     }
    // }

    // pub fn get_message_likes(&self, msg_id: MessageID, from_index: U64, limit: U64) -> Vec<AccountId> {
    //     if let Some(post) = self.posts.get(&msg_id.post_id) {
    //         let idx = u64::from(msg_id.msg_idx);
    //         if let Some(msg) = post.messages.get(idx) {
    //             use std::convert::TryFrom;
    //             if let (Ok(from), Ok(lim)) = (usize::try_from(u64::from(from_index)), usize::try_from(u64::from(limit))) {
    //                 msg.likes
    //                     .iter()
    //                     .skip(from)
    //                     .take(lim)
    //                     .collect()
    //             } else {
    //                 env::panic_str("'usize' conversion failed");
    //             }
    //         } else {
    //             env::panic_str("Message is not found");
    //         }
    //     } else {
    //         env::panic_str("Post is not found");
    //     }
    // }
    
    // pub fn get_account_last_likes(&self, account_id: AccountId, from_index: u8, limit: u8) -> Vec<(PostId, Option<U64>)> {
    //     if let Some(accounts_stats) = self.accounts_stats.get(&account_id) {
    //         accounts_stats.recent_likes
    //             .into_iter()
    //             .skip(usize::from(from_index))
    //             .take(usize::from(limit))
    //             .map(|item| {
    //                 match item {
    //                     AccountLike::PostLike { post_id } => {
    //                         (post_id, None)
    //                     },
    //                     AccountLike::MessageLike { msg_id } => {
    //                         (msg_id.post_id, Some(U64(msg_id.msg_idx)))
    //                     }
    //                 }
    //             })
    //             .collect()
    //     } else {
    //         Vec::new()
    //     }
    // }

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

    pub fn get_current_settings(&self) -> Settings {
        Settings {
            account_recent_likes_limit: self.settings.account_recent_likes_limit
        }
    }
}


// Private methods

#[near_bindgen]
impl Contract {

    // Assert incoming call

    fn assert_add_message_to_post_call(&self, post_id: &PostId, text: &u8) {
        // TODO: validate 'text' format and length
        // if text.trim().is_empty() {
        //     env::panic_str("'text' is empty or whitespace");
        // };
        // self.assert_post_id(post_id);
    }

    fn assert_add_message_to_message_call(&self, parent_msg_id: &MessageID, text: &String) {
        // TODO: validate 'text' format and length
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        };

        self.assert_message_id(parent_msg_id);

        let post_id = &parent_msg_id.post_id;
        let msg_idx: u64 = parent_msg_id.msg_idx.into();
        
        // if let Some(post) = self.posts.get(post_id) {
        //     if !post.messages.get(msg_idx).is_some() {
        //         env::panic_str("Parent message does not exist");
        //     };
        // } else {
        //     env::panic_str("Post does not exist");
        // };
    }

    // fn assert_like_post_call(&self, post_id: &PostId) {
    //     let account_id = env::signer_account_id();

    //     self.assert_post_id(post_id);

    //     if let Some(post) = self.posts.get(post_id) {
    //         if post.likes.contains(&account_id) {
    //             env::panic_str("Post is liked already");
    //         };
    //     };
    //     // else {
    //     //     env::panic_str("Post does not exist");
    //     // }
    // }

    // fn assert_unlike_post_call(&self, post_id: &PostId) {
    //     let account_id = env::signer_account_id();

    //     self.assert_post_id(post_id);

    //     if let Some(post) = self.posts.get(post_id) {
    //         if !post.likes.contains(&account_id) {
    //             env::panic_str("Post is not liked");
    //         };
    //     } else {
    //         env::panic_str("Post does not exist");
    //     };
    // }

    // fn assert_like_message_call(&self, msg_id: &MessageID) {
    //     let account_id = env::signer_account_id();
        
    //     self.assert_message_id(msg_id);

    //     let post_id = &msg_id.post_id;
    //     let msg_idx: u64 = msg_id.msg_idx.into();
        
    //     if let Some(post) = self.posts.get(post_id) {
    //         if let Some(msg) = post.messages.get(msg_idx) {
    //             if msg.likes.contains(&account_id) {
    //                 env::panic_str("Message is liked already");
    //             };
    //         } else {
    //             env::panic_str("Message does not exist");
    //         };
    //     } else {
    //         env::panic_str("Post does not exist");
    //     };
    // }

    // fn assert_unlike_message_call(&self, msg_id: &MessageID) {
    //     let account_id = env::signer_account_id();
    //     self.assert_message_id(msg_id);

    //     let post_id = &msg_id.post_id;
    //     let msg_idx: u64 = msg_id.msg_idx.into();
        
    //     if let Some(post) = self.posts.get(post_id) {
    //         if let Some(msg) = post.messages.get(msg_idx) {
    //             if !msg.likes.contains(&account_id) {
    //                 env::panic_str("Message is not liked");
    //             };
    //         } else {
    //             env::panic_str("Message does not exist");
    //         };
    //     } else {
    //         env::panic_str("Post does not exist");
    //     };
    // }

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
        // TODO: validate 'post_id' format and length
        // if post_id.trim().is_empty() {
        //     env::panic_str("'post_id' is empty or whitespace");
        // };
    }

    fn assert_message_id(&self, msg_id: &MessageID) {
        let post_id = &msg_id.post_id;
        self.assert_post_id(post_id);
    }

    // Add storage collections

    // fn add_post_storage(&mut self, post_id: &PostId) -> Post {
    //     let post = Post {
    //         messages: Vector::new(b'm'
    //             /* StorageKeys::Messages { // 36
    //                 post_id: env::sha256(post_id.as_bytes())  
    //             } */
    //         ),
    //         // likes: UnorderedSet::new(
    //         //     StorageKeys::PostLikes { 
    //         //         post_id: env::sha256(post_id.as_bytes())
    //         //     }
    //         // )
    //     };

    //     self.posts.insert(post_id, &post);

    //     post
    // }

    fn add_account_stat_storage(&mut self, account_id: &AccountId) -> AccountStats {
        let account_stat = AccountStats {
            recent_likes: Vec::new()
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
    
    
    // Execute call logic


    fn execute_add_message_to_post_call(&mut self, key: u8, value: u16) {        
        let storage_usage_before = env::storage_usage();
        log!("storage_usage_before {}", &storage_usage_before);

        // self.posts.insert(&key, &value);
        self.posts.push(&value);

        let storage_usage_after = env::storage_usage();
        log!("storage_usage_after {}", &storage_usage_after);

        let diff_storage_usage = storage_usage_after - storage_usage_before;
        log!("diff {}", &diff_storage_usage);
    }

    // fn execute_add_message_to_post_call(&mut self, post_id: PostId, text: String) -> (PostId, U64) {
    //     let account_id = env::signer_account_id();
        
    //     let before_post_storage_usage = env::storage_usage();
    //     log!("before_post_storage_usage {}", &before_post_storage_usage);

    //     let post = 1u8;
    //     // let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
    //     //     self.add_post_storage(&post_id)
    //     // });
        
    //     // let msg_idx = post.messages.len();
    //     // let msg = Message {
    //     //     account: account_id,
    //     //     parent_idx: None,
    //     //     payload: MessagePayload::Text { text },
    //     //     timestamp: env::block_timestamp(),
    //     //     // likes: UnorderedSet::new(
    //     //     //     StorageKeys::MessageLikes { 
    //     //     //         post_id: env::sha256(post_id.as_bytes()),
    //     //     //         msg_idx: msg_idx 
    //     //     //     }
    //     //     // )
    //     // };

    //     // let msg_vec = msg.try_to_vec().unwrap();
    //     // log!("msg_vec {}", msg_vec.len());

    //     // post.messages.push(&msg);
    //     self.posts.insert(&post_id, &post);

    //     let after_post_storage_usage = env::storage_usage();
    //     log!("after_post_storage_usage {}", &after_post_storage_usage);

    //     let diff_storage_usage = env::storage_usage() - before_post_storage_usage;
    //     log!("diff_storage_usage {}", &diff_storage_usage);

    //     // (post_id, U64(msg_idx))
    //     (post_id, U64(0))
    // }

    fn execute_add_message_to_message_call(&mut self, parent_msg_id: MessageId, text: String) -> (PostId, U64) {
        let account_id = env::signer_account_id();
        
        // let mut post = self.posts.get(&parent_msg_id.post_id).expect("Post is not found");
        
        // let msg_idx = post.messages.len();
        // let msg = Message {
        //     account: account_id,
        //     parent_idx: Some(parent_msg_id.msg_idx),
        //     payload: MessagePayload::Text { text },
        //     timestamp: env::block_timestamp(),
        //     // likes: UnorderedSet::new(
        //     //     StorageKeys::MessageLikes {
        //     //         post_id: env::sha256(parent_msg_id.post_id.as_bytes()),
        //     //         msg_idx: msg_idx 
        //     //     }
        //     // )
        // };
        // post.messages.push(&msg);
        // self.posts.insert(&parent_msg_id.post_id, &post);

        (parent_msg_id.post_id, U64(0))
    }

    fn execute_add_friend_call(&mut self, friend_id: AccountId) {
        let account_id = env::signer_account_id();

        let mut account_friends = self.accounts_friends.get(&account_id).unwrap_or_else(|| {
            self.add_account_friends_storage(&account_id)
        });

        account_friends.insert(&friend_id);
        self.accounts_friends.insert(&account_id, &account_friends);
    }
    
    // fn execute_like_post_call(&mut self, post_id: PostId) {
    //     let account_id = env::signer_account_id();

    //     // Update post stats
    //     let mut post = self.posts.get(&post_id).unwrap_or_else(|| {
    //         self.add_post_storage(&post_id)
    //     });
    //     post.likes.insert(&account_id);
    //     self.posts.insert(&post_id, &post);

    //     // Update account stats
    //     let like = AccountLike::PostLike { post_id };
    //     self.add_like_to_account_likes_stat(account_id, like);
    // }

    // fn execute_unlike_post_call(&mut self, post_id: PostId) {
    //     let account_id = env::signer_account_id();
        
    //     // Update post stats
    //     let mut post = self.posts.get(&post_id).expect("Post is not found");
    //     post.likes.remove(&account_id);                
    //     self.posts.insert(&post_id, &post);

    //     // Update account stats
    //     let like = AccountLike::PostLike { post_id };
    //     self.remove_like_from_account_likes_stat(account_id, like);
    // }

    // fn execute_like_message_call(&mut self, msg_id: MessageId) {
    //     let account_id = env::signer_account_id();

    //     // Update message stats
    //     let mut post = self.posts.get(&msg_id.post_id).expect("Post is not found");
    //     let mut msg = post.messages.get(msg_id.msg_idx).expect("Message is not found");
    //     msg.likes.insert(&account_id);
    //     post.messages.replace(msg_id.msg_idx, &msg);
    //     self.posts.insert(&msg_id.post_id, &post);

    //     // Update account stats
    //     let like = AccountLike::MessageLike { msg_id };
    //     self.add_like_to_account_likes_stat(account_id, like);
    // }

    // fn execute_unlike_message_call(&mut self, msg_id: MessageId) {
    //     let account_id = env::signer_account_id();
        
    //     // Update message stats
    //     let mut post = self.posts.get(&msg_id.post_id).expect("Post is not found");
    //     let mut msg = post.messages.get(msg_id.msg_idx).expect("Message is not found");
    //     msg.likes.remove(&account_id);
    //     post.messages.replace(msg_id.msg_idx, &msg);
    //     self.posts.insert(&msg_id.post_id, &post);

    //     // Update account stats
    //     let like = AccountLike::MessageLike { msg_id };
    //     self.remove_like_from_account_likes_stat(account_id, like);
    // }

    fn execute_update_profile_call(&mut self, json_metadata: Option<String>, image: Option<Vec<u8>>) {
        let account_id = env::signer_account_id();

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

    // fn add_like_to_account_likes_stat(&mut self, account_id: AccountId, like: AccountLike) {
    //     let mut account_stats = self.accounts_stats.get(&account_id).unwrap_or_else(|| {
    //         self.add_account_stat_storage(&account_id)
    //     });

    //     let account_recent_likes_limit = usize::from(self.settings.account_recent_likes_limit);

    //     let updated_account_stats = if account_stats.recent_likes.len() > 0 && account_recent_likes_limit == 0 {
    //         account_stats.recent_likes.clear();
    //         account_stats
    //     } else {
    //         if account_stats.recent_likes.len() > account_recent_likes_limit {
    //             let skip = account_stats.recent_likes.len() - account_recent_likes_limit;
    //             account_stats.recent_likes = account_stats.recent_likes.into_iter().skip(skip + 1).collect();
    //             account_stats.recent_likes.push(like);
    //             account_stats
    //         } else if account_stats.recent_likes.len() == account_recent_likes_limit {
    //             let skip = 1;
    //             account_stats.recent_likes = account_stats.recent_likes.into_iter().skip(skip).collect();
    //             account_stats.recent_likes.push(like);
    //             account_stats
    //         } else {
    //             account_stats.recent_likes.push(like);
    //             account_stats
    //         }
    //     };

    //     self.accounts_stats.insert(&account_id, &updated_account_stats);
    // }

    // fn remove_like_from_account_likes_stat(&mut self, account_id: AccountId, like: AccountLike) {
    //     let mut account_stats = self.accounts_stats.get(&account_id).unwrap_or_else(|| {
    //         self.add_account_stat_storage(&account_id)
    //     });

    //     let updated_account_stats = if let Some(idx) = account_stats.recent_likes.iter().position(|l| l == &like) {
    //         account_stats.recent_likes.remove(idx);
    //         account_stats
    //     } else {
    //         account_stats
    //     };

    //     self.accounts_stats.insert(&account_id, &updated_account_stats);
    // }

    fn collect_fee_and_execute_call(&mut self, fee: Balance, call: Call) -> Promise {
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
    pub fn on_fee_collected(&mut self, call: Call) -> CallResult {

        if env::promise_results_count() != 1 {
            env::panic_str("Unexpected promise results count");
        }

        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                match call {
                    Call::AddMessageToPost { post_id, text } => {
                        self.execute_add_message_to_post_call(post_id, text);
                        CallResult::MessageToPostAdded { id: MessageID { post_id, msg_idx: U64(0) } }
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
                        // self.execute_like_post_call(post_id);
                        CallResult::PostLiked
                    },
                    Call::UnlikePost { post_id } => {
                        // self.execute_unlike_post_call(post_id);
                        CallResult::PostUnliked
                    },
                    Call::LikeMessage { msg_id } => {
                        // self.execute_like_message_call(msg_id.into());
                        CallResult::MessageLiked
                    },
                    Call::UnlikeMessage { msg_id } => {
                        // self.execute_unlike_message_call(msg_id.into());
                        CallResult::MessageUnliked
                    },
                    Call::UpdateProfile { profile } => {
                        let image: Option<Vec<u8>> = match profile.image {
                            Some(vec) => Some(vec.into()),
                            None => None
                        };
                        self.execute_update_profile_call(profile.json_metadata, image);
                        CallResult::ProfileUpdated
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