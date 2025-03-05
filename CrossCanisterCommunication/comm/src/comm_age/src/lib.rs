use ic_cdk_macros::{update, query, export_candid};
use std::cell::RefCell;

thread_local! {
    static AGE: RefCell<u32> = RefCell::new(0);
}

#[update]
fn set_age(age: u32) {
    AGE.with(|a| *a.borrow_mut() = age);
}

#[query]
fn get_age() -> u32 {
    AGE.with(|a| *a.borrow())
}

export_candid!();