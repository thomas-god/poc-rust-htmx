use maud::{Markup, html};

use crate::Todo;

pub fn todos_view(todos: &[Todo]) -> Markup {
    html! {
        div {
            ol id="todos-list" {
                @for todo in todos {
                    (todo_view(todo))
                }
            }
        }
    }
}

pub fn todo_view(todo: &Todo) -> Markup {
    html! {
        li { (todo.content) }
    }
}

pub fn todo_form() -> Markup {
    html!(
        div {
            form
                hx-post="/todo"
                hx-target="#todos-list"
                hx-swap="beforeend"
                hx-on::after-request="if(event.detail.successful) {this.reset();}" {
                    input #todo type="text" name="content" .input {}
                    label for="todo" { "Todo content" }
                    button {"Add todo"}
            }
        }
    )
}
