use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Gas, Promise, PanicOnDefault, PromiseResult};
use near_sdk::json_types::{U128};
use near_sdk::collections::{UnorderedMap, Vector};

pub mod external;
pub use crate::external::*;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    owner: AccountId,
    fee_ft: AccountId,
    posted_messages: UnorderedMap<PostId, Vector<PostedMessage>>
}

type PostId = String;

#[derive(BorshDeserialize, BorshSerialize)]
struct PostedMessage {
    text: String,
    sender: AccountId
}

#[near_bindgen]
impl Contract {
  #[init]
  pub fn new(owner: AccountId, fee_ft: AccountId) -> Self {
      assert!(!env::state_exists(), "Already initialized");
      Self {
          owner,
          fee_ft,
          posted_messages: UnorderedMap::new(b"p".to_vec()),
      }
  }

  pub fn add_message(&mut self, post_id: PostId, text: String) {
    self.collect_fee(ContractAction::AddMessage { post_id, text });
  }

  fn internal_add_message(&mut self, post_id: PostId, text: String) {
      let message = PostedMessage {
          text,
          sender: env::signer_account_id().clone()
      };

      let mut v = self.posted_messages.get(&post_id).unwrap_or_else(|| {
          let mut prefix = Vec::with_capacity(33);
          prefix.push(b'm');
          prefix.extend(env::sha256(post_id.as_bytes()));
          Vector::new(prefix)
      });

      v.push(&message);
      self.posted_messages.insert(&post_id, &v);
  }

  pub fn get_post_messages_count(&self, from_index: u64, limit: u64) -> Vec<(String, u64)> {
      let keys = self.posted_messages.keys_as_vector();
      let values = self.posted_messages.values_as_vector();
      (from_index..std::cmp::min(from_index + limit, self.posted_messages.len()))
          .map(|index| {
            let key: PostId = keys.get(index).unwrap();
            let value = values.get(index).unwrap();
            (key, value.len())
          })
          .collect()
  }


  fn collect_fee(&mut self, action: ContractAction) -> Promise {
      ext_ft::ext(self.fee_ft.clone())
          .with_static_gas(Gas(5*TGAS))
          .ft_collect_fee(U128::from(FIXED_FEE))
              .then(
                  ext_self::ext(env::current_account_id())
                  .with_static_gas(Gas(5*TGAS))
                  .on_fee_collected(action)
              )
  }

  #[private]
  pub fn on_fee_collected(&mut self, action: ContractAction) -> String {
      assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");

      match env::promise_result(0) {
          PromiseResult::Successful(_) => {
              match action {
                ContractAction::AddMessage { post_id, text } => {
                  self.internal_add_message(post_id, text);
                }
                _ => {}
              }
              return "Success".to_string();
          },
          PromiseResult::NotReady => env::panic_str("Not Ready"),
          PromiseResult::Failed => env::panic_str("Failed"),
      };
  }
}