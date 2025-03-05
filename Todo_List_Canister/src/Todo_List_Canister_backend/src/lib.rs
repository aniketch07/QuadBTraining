use candid::Principal;
use ic_cdk::caller;
use ic_cdk::export_candid;
use ic_cdk_macros::{query, update};
use std::cell::RefCell;

#[derive(Debug, Default)]
struct Task {
    name: String,
    doneby: String,
}

#[derive(Debug, Default)]
struct Todo {
    task: Task,
    completed: bool,
}

thread_local! {
    static TODOS: RefCell<Vec<Todo>> = RefCell::new(Vec::new());
}

// Add a new todo
#[update]
fn add_todo(task_name: String, done_by: String) {
    TODOS.with(|todos| {
        let mut todos = todos.borrow_mut();
        todos.push(Todo {
            task: Task {
                name: task_name,
                doneby: done_by,
            },
            completed: false,
        });
    });
}

// Get the list of todos
#[query]
fn get_todos() -> Vec<(String, String, bool)> {
    TODOS.with(|todos| {
        let todos = todos.borrow();
        todos
            .iter()
            .map(|todo| (todo.task.name.clone(), todo.task.doneby.clone(), todo.completed))
            .collect()
    })
}

// Toggle a todo's completion status
#[update]
fn toggle_todo(index: u64) {  // Changed from usize to u64
    TODOS.with(|todos| {
        let mut todos = todos.borrow_mut();
        let index = index as usize;  // Convert u64 to usize for Vec indexing
        if let Some(todo) = todos.get_mut(index) {
            todo.completed = !todo.completed;
        }
    });
}

#[query]
fn whoami() -> Principal {
    caller()
}

export_candid!();