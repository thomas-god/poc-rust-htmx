use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Todo {
    pub id: usize,
    pub content: String,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    todos: Vec<Todo>,
    todo_counter: usize,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            todos: Vec::new(),
            todo_counter: 0,
        }
    }

    pub fn todos(&self) -> &[Todo] {
        &self.todos
    }

    pub fn add_todo(&mut self, content: &str) -> &Todo {
        self.todos.push(Todo {
            id: self.todo_counter,
            content: content.to_owned(),
            done: false,
        });
        self.todo_counter += 1;
        self.todos.last().unwrap()
    }

    pub fn toggle_todo(&mut self, todo_id: usize) -> Option<&Todo> {
        let Some(todo) = self.todos.iter_mut().find(|t| t.id == todo_id) else {
            return None;
        };
        todo.done = !todo.done;
        Some(todo)
    }

    pub fn delete_todo(&mut self, todo_id: usize) -> Option<Todo> {
        let position = self.todos.iter().position(|t| t.id == todo_id)?;
        Some(self.todos.swap_remove(position))
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState::new()
    }
}
