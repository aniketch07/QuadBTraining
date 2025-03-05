use candid::{Decode, Encode, Principal};
use ic_cdk::{caller, export_candid, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{StableBTreeMap, DefaultMemoryImpl, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

#[derive(candid::CandidType, Serialize, Deserialize, Clone)]
struct Task {
    id: u64,
    name: String,
    principal: Principal,
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static TASKS: RefCell<StableBTreeMap<u64, Task, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))))
    );

    static TASK_COUNTER: RefCell<u64> = RefCell::new(0);
}

impl Storable for Task {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: ic_stable_structures::storable::Bound = ic_stable_structures::storable::Bound::Unbounded;
}

#[update]
fn add_task(task_name: String) -> u64 {
    let user = caller();
    if user == Principal::anonymous() {
        ic_cdk::trap("Unauthorized: Please log in using Internet Identity.");
    }

    let task_id = TASK_COUNTER.with(|counter| {
        let mut count = counter.borrow_mut();
        *count += 1;
        *count
    });

    let task = Task {
        id: task_id,
        name: task_name,
        principal: user,
    };

    TASKS.with(|tasks| {
        tasks.borrow_mut().insert(task_id, task);
    });

    task_id
}

#[query]
fn get_tasks() -> Vec<Task> {
    let user = caller();
    if user == Principal::anonymous() {
        ic_cdk::trap("Unauthorized: Please log in using Internet Identity.");
    }

    TASKS.with(|tasks| {
        tasks
            .borrow()
            .iter()
            .filter_map(|(_, task)| {
                if task.principal == user {
                    Some(task.clone())
                } else {
                    None
                }
            })
            .collect()
    })
}

#[update]
fn delete_task(task_id: u64) {
    let user = caller();
    if user == Principal::anonymous() {
        ic_cdk::trap("Unauthorized: Please log in using Internet Identity.");
    }

    TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        if let Some(task) = tasks.get(&task_id) {
            if task.principal != user {
                ic_cdk::trap("Unauthorized: You can only delete your own tasks.");
            }
            tasks.remove(&task_id);
        }
    });
}

#[query]
fn whoami() -> Principal {
    caller()
}
export_candid!();