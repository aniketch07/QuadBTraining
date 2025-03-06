use ic_cdk_macros::{query, update, init};
use ic_cdk::api::{time, caller};
use std::collections::HashMap;
use std::cell::RefCell;
use candid::Principal;

// Store the deployer (owner) of the canister and poll data
thread_local! {
    static OWNER: RefCell<Option<Principal>> = RefCell::new(None);
    static POLLS: RefCell<HashMap<u64, Poll>> = RefCell::new(HashMap::new());
    static POLL_ID_COUNTER: RefCell<u64> = RefCell::new(0);
}

// Poll structure
struct Poll {
    question: String,
    description:String,
    options: Vec<String>,
    votes: HashMap<String, i32>,
    voters: HashMap<Principal, String>,
    start_time: u64,
    end_time: u64,
}

// Initialize the owner when the canister is deployed
#[init]
fn init() {
    let caller = ic_cdk::caller();
    ic_cdk::println!("Initializing canister with owner: {}", caller); // Debug print
    OWNER.with(|owner| {
        *owner.borrow_mut() = Some(caller);
    });
}

// Query to get the owner
#[query]
fn get_owner() -> Option<Principal> {
    OWNER.with(|owner| *owner.borrow())
}

// Helper function to check if the caller is the owner
fn is_owner() -> bool {
    OWNER.with(|owner| *owner.borrow() == Some(caller()))
}

// Create a new poll (only the owner can call this)
#[update]
fn create_poll(question: String, description:String,options: Vec<String>, duration_seconds: u64) -> Result<u64, String> {
    if !is_owner() {
        return Err("Error: Only the owner can create polls.".to_string());
    }
    
    if options.is_empty() {
        return Err("Error: Poll must have at least one option.".to_string());
    }

    let start_time = time() / 1_000_000_000; // Convert nanoseconds to seconds
    let end_time = start_time + duration_seconds;

    POLL_ID_COUNTER.with(|counter| {
        let mut id = counter.borrow_mut();
        *id += 1;

        POLLS.with(|polls| {
            polls.borrow_mut().insert(*id, Poll {
                question,
                description,
                options: options.clone(),
                votes: options.into_iter().map(|opt| (opt, 0)).collect(),
                voters: HashMap::new(),
                start_time,
                end_time,
            });
        });

        Ok(*id) // Return poll ID
    })
}

// Vote on a poll
#[update]
fn vote(poll_id: u64, option: String) -> String {
    let now = time() / 1_000_000_000;
    let user = caller();

    POLLS.with(|polls| {
        let mut polls = polls.borrow_mut();
        if let Some(p) = polls.get_mut(&poll_id) {
            if now < p.start_time {
                return "Poll has not started yet.".to_string();
            }
            if now >= p.end_time {
                return "Poll has already ended.".to_string();
            }
            if p.voters.contains_key(&user) {
                return "Error: You have already voted in this poll.".to_string();
            }
            if let Some(count) = p.votes.get_mut(&option) {
                *count += 1;
                p.voters.insert(user, option.clone());
                return format!("Voted for '{}' in poll {}", option, poll_id);
            }
            return "Error: Invalid option.".to_string();
        }
        "Error: Poll not found.".to_string()
    })
}

// Get poll details
#[query]
fn get_poll(poll_id: u64) -> Option<(String, Vec<String>, u64, u64)> {
    POLLS.with(|polls| {
        polls.borrow().get(&poll_id).map(|p| (
            p.question.clone(),
            p.options.clone(),
            p.start_time,
            p.end_time,
        ))
    })
}

// Get results (only after poll ends)
#[query]
fn get_results(poll_id: u64) -> Option<HashMap<String, i32>> {
    let now = time() / 1_000_000_000;

    POLLS.with(|polls| {
        polls.borrow().get(&poll_id).and_then(|p| {
            if now >= p.end_time {
                Some(p.votes.clone())
            } else {
                None
            }
        })
    })
}

// Get the winner (only after poll ends)
#[query]
fn get_winner(poll_id: u64) -> Option<String> {
    let now = time() / 1_000_000_000;

    POLLS.with(|polls| {
        polls.borrow().get(&poll_id).and_then(|p| {
            if now >= p.end_time {
                p.votes.iter().max_by_key(|&(_, v)| v).map(|(winner, _)| winner.clone())
            } else {
                None
            }
        })
    })
}

// Get all active poll IDs
#[query]
fn get_active_polls() -> Vec<u64> {
    let now = time() / 1_000_000_000;

    POLLS.with(|polls| {
        polls.borrow()
            .iter()
            .filter(|(_, p)| now < p.end_time)
            .map(|(&id, _)| id)
            .collect()
    })
}

// Export Candid for frontend integration
ic_cdk::export_candid!();
