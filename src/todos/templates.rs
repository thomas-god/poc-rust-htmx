use maud::{Markup, html};

use crate::todos::state::Todo;

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
    let toggle_url = format!("/todo/{}/toggle", todo.id);
    let delete_url = format!("/todo/{}", todo.id);
    let content_style = if todo.done {
        "line-through text-gray-500"
    } else {
        ""
    };
    html! {
        li.list-row.hover:bg-base-300
            hx-post=(toggle_url)
            hx-trigger="click"
            hx-target="closest li"
            hx-swap="outerHTML" {
            div.list-col-grow
            {
                span class=(content_style) {
                    (todo.content)
                }
            }
            div {
                button.btn.btn-secondary
                hx-delete=(delete_url)
                hx-target="closest li"
                hx-trigger="click consume" {
                    "X"
                }
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
                        button.btn.btn-primary.join-item {"Add"}
                    }
                }
            }
        }
    )
}
