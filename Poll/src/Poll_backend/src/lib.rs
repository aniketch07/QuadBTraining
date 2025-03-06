use ic_cdk_macros::{query, update, init};
use ic_cdk::api::{time, caller};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap, Storable
};
use std::{borrow::Cow, collections::HashMap, cell::RefCell};
use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use serde::{self, Serialize};

type Memory = VirtualMemory<DefaultMemoryImpl>;
const OWNER_MEMORY_ID: MemoryId = MemoryId::new(0);
const POLLS_MEMORY_ID: MemoryId = MemoryId::new(1);
const POLL_ID_COUNTER_MEMORY_ID: MemoryId = MemoryId::new(2);

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Poll {
    question: String,
    description: String,
    options: Vec<String>,
    votes: HashMap<String, i32>,
    voters: HashMap<Principal, String>,
    start_time: u64,
    end_time: u64,
}

thread_local! {
    static MEMORY_MANAGER: MemoryManager<DefaultMemoryImpl> = MemoryManager::init(DefaultMemoryImpl::default());

    static OWNER: RefCell<StableBTreeMap<u8, Principal, Memory>> =
        RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.get(OWNER_MEMORY_ID))));

    static POLLS: RefCell<StableBTreeMap<u64, Poll, Memory>> =
        RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.get(POLLS_MEMORY_ID))));

    static POLL_ID_COUNTER: RefCell<StableBTreeMap<u8, u64, Memory>> =
        RefCell::new(StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.get(POLL_ID_COUNTER_MEMORY_ID))));
}

impl Storable for Poll {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Debug)]
pub enum PollError {
    Unauthorized,
    EmptyOptions,
    NotFound,
    PollNotStarted,
    PollEnded,
    AlreadyVoted,
    InvalidOption,
}

// Initialize the owner when the canister is deployed
#[init]
fn init() {
    let caller = ic_cdk::caller();
    ic_cdk::println!("Initializing canister with owner: {}", caller);
    
    OWNER.with(|owner| owner.borrow_mut().insert(0, caller));
    POLL_ID_COUNTER.with(|counter| counter.borrow_mut().insert(0, 0));
}

// Query to get the owner
#[query]
fn get_owner() -> Option<Principal> {
    OWNER.with(|owner| owner.borrow().get(&0))
}

// Helper function to check if the caller is the owner
fn is_owner() -> bool {
    OWNER.with(|owner| owner.borrow().get(&0) == Some(caller()))
}

// Create a new poll (only the owner can call this)
#[update]
fn create_poll(question: String, description: String, options: Vec<String>, duration_seconds: u64) -> Result<u64, PollError> {
    if !is_owner() {
        return Err(PollError::Unauthorized);
    }
    if options.is_empty() {
        return Err(PollError::EmptyOptions);
    }

    let start_time = time() / 1_000_000_000;
    let end_time = start_time + duration_seconds;

    let poll_id = POLL_ID_COUNTER.with(|counter| {
        let mut counter = counter.borrow_mut();
        let current_id = counter.get(&0).unwrap_or(0);
        let new_id = current_id + 1;
        counter.insert(0, new_id);
        new_id
    });

    POLLS.with(|polls| {
        polls.borrow_mut().insert(poll_id, Poll {
            question,
            description,
            options: options.clone(),
            votes: options.into_iter().map(|opt| (opt, 0)).collect(),
            voters: HashMap::new(),
            start_time,
            end_time,
        });
    });

    Ok(poll_id)
}

#[update]
fn vote(poll_id: u64, option: String) -> Result<String, PollError> {
    let now = time() / 1_000_000_000;
    let user = caller();

    POLLS.with(|polls| {
        let mut polls = polls.borrow_mut();
        let mut poll = polls.get(&poll_id).ok_or(PollError::NotFound)?;

        if now < poll.start_time {
            return Err(PollError::PollNotStarted);
        }
        if now >= poll.end_time {
            return Err(PollError::PollEnded);
        }
        if poll.voters.contains_key(&user) {
            return Err(PollError::AlreadyVoted);
        }

        if let Some(count) = poll.votes.get_mut(&option) {
            *count += 1;
            poll.voters.insert(user, option.clone());
            polls.insert(poll_id, poll.clone());
            Ok(format!("Voted for '{}' in poll {}", option, poll_id))
        } else {
            Err(PollError::InvalidOption)
        }
    })
}

// Get poll details
#[query]
fn get_poll(poll_id: u64) -> Result<(String, Vec<String>, u64, u64), PollError> {
    POLLS.with(|polls| {
        polls.borrow().get(&poll_id)
            .map(|p| (p.question.clone(), p.options.clone(), p.start_time, p.end_time))
            .ok_or(PollError::NotFound)
    })
}

// Get results (only after poll ends)
#[query]
fn get_results(poll_id: u64) -> Result<HashMap<String, i32>, PollError> {
    let now = time() / 1_000_000_000;
    POLLS.with(|polls| {
        polls.borrow().get(&poll_id).and_then(|p| {
            if now >= p.end_time {
                Some(p.votes.clone())
            } else {
                None
            }
        }).ok_or(PollError::PollEnded)
    })
}

// Get the winner (only after poll ends)
#[query]
fn get_winner(poll_id: u64) -> Result<String, PollError> {
    let now = time() / 1_000_000_000;
    POLLS.with(|polls| {
        polls.borrow().get(&poll_id).and_then(|p| {
            if now >= p.end_time {
                p.votes.iter().max_by_key(|&(_, v)| v).map(|(winner, _)| winner.clone())
            } else {
                None
            }
        }).ok_or(PollError::PollEnded)
    })
}

// Get all active poll IDs
#[query]
fn get_active_polls() -> Vec<u64> {
    let now = time() / 1_000_000_000;
    POLLS.with(|polls| {
        polls.borrow().iter()
            .filter(|(_, p)| now < p.end_time)
            .map(|(id, _)| id)
            .collect()
    })
}

// Export Candid for frontend integration
ic_cdk::export_candid!();
