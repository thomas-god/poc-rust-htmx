use maud::{Markup, html};

use crate::Todo;

pub fn todos_view(todos: &[Todo]) -> Markup {
    html! {
        ul.list.bg-base-100.rounded-box.shadow-md.m-6 id="todos-list" {
            @for todo in todos {
                (todo_view(todo))
            }
        }
    }
}

pub fn todo_view(todo: &Todo) -> Markup {
    html! {
        li.list-row.hover:bg-base-300 {
            div.list-col-grow {
                (todo.content)
            }
        }
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
                fiedlset.fieldset.w-xs.bg-base-200.border.border-base-300.p-4.rounded-box {
                    legend.fieldset-legend { "New todo" }
                    div.join {
                        input.input.join-item #todo type="text" name="content" {}
                        // label for="todo" { "Todo content" }
                        button.btn.btn-primary.join-item {"Add"}
                    }
                }
            }
        }
    )
}
