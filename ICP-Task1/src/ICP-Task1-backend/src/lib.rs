use ic_cdk::storage;
use ic_cdk_macros::{query, update};
use std::cell::RefCell;

#[derive(Default)]
struct Profile{
    name:String,
    age:i32,
}

thread_local! {
    static PROFILE: RefCell<Profile> = RefCell::new(Profile::default());
}

#[query]
fn get_profile() -> (String, i32) {
    PROFILE.with(|profile| {
        let p = profile.borrow();
        (p.name.clone(), p.age)
    })
}

// Update to set a new profile
#[update]
fn set_profile(name: String, age: i32) {
    PROFILE.with(|profile| {
        let mut p = profile.borrow_mut();
        p.name = name;
        p.age = age;
    });
}

#[query]
fn greet(name: String, age: i32) -> String {
    format!("Hello, {} Your age is {}!", name, age)
}