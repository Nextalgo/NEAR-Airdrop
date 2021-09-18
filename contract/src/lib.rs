
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde_json::{json, from_slice};
use near_sdk::{AccountId, Balance, PromiseOrValue, env, near_bindgen, setup_alloc};
use near_sdk::collections::{LookupMap, UnorderedMap};
use std::collections::HashMap;
use std::fmt::Debug;
use near_sdk::serde::{Serialize, Deserialize};
use near_sdk::{BlockHeight, Gas, PanicOnDefault, Promise, PromiseResult};
use near_sdk::json_types::{U128, U64, ValidAccountId};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;

setup_alloc!();

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Airdrop {
    tokens: UnorderedMap<AccountId, FungibleTokenMetadata>,
    records: Vec<Record>,
    tasks: HashMap<AccountId, Vec<Task>>,
    
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Record {
    creator: AccountId,
    receiver: AccountId,
    token: AccountId,
    amount: U128,          
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Task {
    creator: AccountId,
    total_count: u32,
    amount_per_account: U128,
    token: AccountId,
    index: u32,
    deposit_near: U128,
    claimed_account: HashMap<AccountId, U128>,
}

#[near_bindgen]
impl Airdrop {
    #[init]
    pub fn new() -> Self {
        Self {
            tokens: UnorderedMap::new(b't'),
            records: Vec::new(),
            tasks: HashMap::new(),
        }
    }

    pub fn get_token_list(&self) -> Vec<FungibleTokenMetadata> {
        self.tokens.values().collect()
    }

    pub fn add_token(&mut self, address: AccountId) {
        assert!(self.tokens.get(&address.clone()).is_none(), "token already exist.");
        let promise = env::promise_create(address.clone(), b"ft_metadata", &json!("{}").to_string().as_bytes(), 0, 0);
        let metadata = match env::promise_result(promise) {
            PromiseResult::Successful(v) => v,
            _ => panic!("Get metadata failed."),
        };
        let metadata: FungibleTokenMetadata = from_slice(&metadata).unwrap();
        self.tokens.insert(&address, &metadata);
    }

    #[payable]
    pub fn add_task(&mut self, total_count: u32, amount_per_account: U128, token: AccountId, deposit_near:U128) {
        let total_amount = total_count as u128 * u128::from(amount_per_account);
    }
}

#[near_bindgen]
#[allow(unreachable_code)]
impl FungibleTokenReceiver for Airdrop {
    /// Callback on receiving tokens by this contract.
    /// `msg` format is either "" for deposit or `TokenReceiverMessage`.
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_in = env::predecessor_account_id();
        if msg.is_empty() {
            // Simple deposit.
            self.internal_deposit(sender_id.as_ref(), &token_in, amount.into());
            PromiseOrValue::Value(U128(0))
        } else {
            // [AUDIT14] shutdown instant swap from interface
            env::panic(b"Instant Swap Feature Not Open Yet");

            let message =
                serde_json::from_str::<TokenReceiverMessage>(&msg).expect("ERR_MSG_WRONG_FORMAT");
            match message {
                TokenReceiverMessage::Execute {
                    referral_id,
                    force,
                    actions,
                } => {
                    let referral_id = referral_id.map(|x| x.to_string());
                    let out_amounts = self.internal_direct_actions(
                        token_in,
                        amount.0,
                        sender_id.as_ref(),
                        force != 0,
                        referral_id,
                        &actions,
                    );
                    for (token_out, amount_out) in out_amounts.into_iter() {
                        self.internal_send_tokens(sender_id.as_ref(), &token_out, amount_out);
                    }
                    // Even if send tokens fails, we don't return funds back to sender.
                    PromiseOrValue::Value(U128(0))
                }
            }
        }
    }
}


impl Airdrop {
    pub(crate) fn internal_deposit(&mut self, token: AccountId, amount: Balance, sender: AccountId) {
        
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn set_then_get_greeting() {
        let context = get_context(vec![], false);
        testing_env!(context);
        let mut contract = Airdrop::default();
        contract.set_greeting("howdy".to_string());
        assert_eq!(
            "howdy".to_string(),
            contract.get_greeting("bob_near".to_string())
        );
    }

    #[test]
    fn get_default_greeting() {
        let context = get_context(vec![], true);
        testing_env!(context);
        let contract = Airdrop::default();
        // this test did not call set_greeting so should return the default "Hello" greeting
        assert_eq!(
            "Hello".to_string(),
            contract.get_greeting("francis.near".to_string())
        );
    }
}
