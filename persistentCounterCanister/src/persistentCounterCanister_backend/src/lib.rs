use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::{init, update, query, pre_upgrade, post_upgrade};
use ic_stable_structures::{memory_manager::{MemoryId, MemoryManager, VirtualMemory}, DefaultMemoryImpl, StableCell};
use std::cell::RefCell;


//our memory storage where we keep our persistant data 
type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    // Initialize a `StableBTreeMap` with `MemoryId(0)`.
    static COUNTER: RefCell<StableCell<u64, Memory>> = RefCell::new(
        StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            0
        ).expect("Failed to initialize counter")
    );
}

#[init]
fn init() {
    COUNTER.with(|counter| counter.borrow_mut().set(0).expect("Init failed"));
}

#[update]
fn increment() -> u64 {
    COUNTER.with(|counter| {
        let mut counter_ref = counter.borrow_mut();
        let value = counter_ref.get() + 1;
        counter_ref.set(value).expect("Set failed");
        value
    })
}

#[query]
fn get_counter() -> u64 {
    COUNTER.with(|counter| *counter.borrow().get())
}

#[update]
fn reset() -> u64 {
    COUNTER.with(|counter| {
        counter.borrow_mut().set(0).expect("Reset failed");
        0
    })
}
#[query]
fn whoami() -> Principal {
    caller()
}

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {
    COUNTER.with(|counter| {
        *counter.borrow_mut() = StableCell::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
            0
        ).expect("Failed");
    });
}

ic_cdk::export_candid!();