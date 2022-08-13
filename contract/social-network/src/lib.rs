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
    TogglePostLike { post_id: PostId },
    ToggleMessageLike { message_id: MessageId },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ContractCallResult {
    AddMessageResult { message_id: MessageId },
    TogglePostLikeResult { is_liked: bool },
    ToggleMessageLikeResult { is_liked: bool }
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
    idx: U64,
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
            posts: LookupMap::new(b'p'),
            likes: LookupMap::new(b'l'),
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

    pub fn toggle_message_like(&mut self, message_id: MessageId) -> Promise {
        self.assert_toggle_message_like_call(&message_id);
        self.collect_fee_and_execute_call(ContractCall::ToggleMessageLike { message_id })
    }

    pub fn get_post_messages(&self, post_id: PostId, from_index: U64, limit: U64) -> Vec<MessageDTO> {
        if let Some(post) = self.posts.get(&post_id) {
            let from = u64::from(from_index);
            let lim = u64::from(limit);
            (from..std::cmp::min(from + lim, post.messages.len()))
                .map(|idx| {
                    let message = post.messages.get(idx).unwrap();
                    MessageDTO {
                        idx: U64(idx),
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

    fn assert_add_message_call(&self, post_id: &PostId, text: &String) {
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
        if text.trim().is_empty() {
            env::panic_str("'text' is empty or whitespace");
        }
    }

    fn execute_add_message_call(&mut self, post_id: &PostId, text: String) -> MessageId {
        let account = env::signer_account_id();

        let mut post = self.posts.get(post_id).unwrap_or_else(|| {
            let mut pref = Vec::with_capacity(33);
            pref.push(b'p');
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
        self.posts.insert(post_id, &post);

        MessageId {
            post_id: post_id.clone(), 
            message_idx: U64(post.messages.len() - 1)
        }
    }

    fn assert_toggle_post_like_call(&self, post_id: &PostId) {
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }
    }

    fn execute_toggle_post_like_call(&mut self, post_id: &PostId) -> bool {
        let account = env::signer_account_id();

        let mut likes_stat = self.likes.get(&account).unwrap_or_else(|| {
            // Initialize account likes statistic for this post
            let mut pref_1 = Vec::with_capacity(33);
            pref_1.push(b'l');
            pref_1.extend(env::sha256(account.as_bytes()));

            let mut likes_stat = AccountLikesStat {
                posts: LookupMap::new(pref_1)
            };

            let mut pref_2 = Vec::with_capacity(65);
            pref_2.push(b'l');
            pref_2.extend(env::sha256(account.as_bytes()));
            pref_2.extend(env::sha256(post_id.as_bytes()));

            let liked_post = AccountLikedPostState {
                is_post_liked: false,
                liked_messages_idx: LookupSet::new(pref_2)
            };

            likes_stat.posts.insert(post_id, &liked_post);
            likes_stat
        });

        let mut liked_post = likes_stat.posts.get(post_id).unwrap();
        let is_liked = !liked_post.is_post_liked;
        liked_post.is_post_liked = is_liked;
        likes_stat.posts.insert(post_id, &liked_post);
        self.likes.insert(&account, &likes_stat);

        // TODO: Revise
        let mut post = self.posts.get(post_id).unwrap();
        if is_liked {
            post.likes_count += 1;
        } else {
            post.likes_count -= 1;
        }
        self.posts.insert(post_id, &post);

        is_liked
    }


    fn assert_toggle_message_like_call(&self, message_id: &MessageId) {
        let post_id = &message_id.post_id;
        if post_id.trim().is_empty() {
            env::panic_str("'post_id' is empty or whitespace");
        }

        let msg_idx = u64::from(message_id.message_idx.clone());
        if let Some(post) = self.posts.get(post_id) {
            let max_idx = post.messages.len() - 1;
            if msg_idx > max_idx {
                env::panic_str("'message_idx' is out of bounds");
            }
        }
    }


    fn execute_toggle_message_like_call(&mut self, message_id: &MessageId) -> bool {
        let account = env::signer_account_id();
        let post_id = &message_id.post_id;
        let msg_idx = u64::from(message_id.message_idx.clone());

        let mut likes_stat = self.likes.get(&account).unwrap_or_else(|| {
            // Initialize account likes statistic for this post
            let mut pref_1 = Vec::with_capacity(33);
            pref_1.push(b'l');
            pref_1.extend(env::sha256(account.as_bytes()));

            let mut likes_stat = AccountLikesStat {
                posts: LookupMap::new(pref_1)
            };

            let mut pref_2 = Vec::with_capacity(65);
            pref_2.push(b'l');
            pref_2.extend(env::sha256(account.as_bytes()));
            pref_2.extend(env::sha256(post_id.as_bytes()));

            let liked_post = AccountLikedPostState {
                is_post_liked: false,
                liked_messages_idx: LookupSet::new(pref_2)
            };

            likes_stat.posts.insert(post_id, &liked_post);
            likes_stat
        });

        let mut liked_post = likes_stat.posts.get(post_id).unwrap();
        let is_liked = !liked_post.liked_messages_idx.contains(&msg_idx);
        if is_liked {
            liked_post.liked_messages_idx.remove(&msg_idx);
        } else {
            liked_post.liked_messages_idx.insert(&msg_idx);
        }
        likes_stat.posts.insert(post_id, &liked_post);
        self.likes.insert(&account, &likes_stat);

        // TODO: Revise
        let mut post = self.posts.get(post_id).unwrap(); // revise
        let mut message = post.messages.get(msg_idx).unwrap();
        if is_liked {
            message.likes_count += 1;
        } else {
            message.likes_count -= 1;
        }
        post.messages.replace(msg_idx, &message);
        self.posts.insert(post_id, &post);

        is_liked
    }

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