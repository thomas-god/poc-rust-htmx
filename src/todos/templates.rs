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

#[cfg(test)]
mod test {
    use scraper::{Html, Selector};

    use crate::todos::state::Todo;

    use super::todo_view;

    #[test]
    fn test_todo_view_not_done_todo() {
        let todo = Todo {
            content: "not done todo".to_owned(),
            done: false,
            id: 0,
        };
        let fragment = Html::parse_fragment(&todo_view(&todo).into_string());
        let selector = Selector::parse("span").unwrap();

        let span = fragment
            .select(&selector)
            .find(|el| el.inner_html() == "not done todo")
            .expect("span should be present");

        assert!(!span.value().attr("class").unwrap().contains("line-through"));
    }

    #[test]
    fn test_todo_view_done_todo() {
        let todo = Todo {
            content: "done todo".to_owned(),
            done: true,
            id: 0,
        };
        let fragment = Html::parse_fragment(&todo_view(&todo).into_string());
        let selector = Selector::parse("span").unwrap();

        let span = fragment
            .select(&selector)
            .find(|el| el.inner_html() == "done todo")
            .expect("span should be present");

        assert!(span.value().attr("class").unwrap().contains("line-through"));
    }

    #[test]
    fn test_toggle_url() {
        let todo = Todo {
            content: "done todo".to_owned(),
            done: true,
            id: 42,
        };
        let fragment = Html::parse_fragment(&todo_view(&todo).into_string());
        let selector = Selector::parse("li").unwrap();

        let li_element = fragment
            .select(&selector)
            .next()
            .expect("li element should exist");

        assert_eq!(
            li_element.value().attr("hx-post").unwrap(),
            "/todo/42/toggle"
        );
    }

    #[test]
    fn test_delete_url() {
        let todo = Todo {
            content: "done todo".to_owned(),
            done: true,
            id: 42,
        };
        let fragment = Html::parse_fragment(&todo_view(&todo).into_string());
        let selector = Selector::parse("button").unwrap();

        let button = fragment
            .select(&selector)
            .next()
            .expect("button should exist");

        assert_eq!(button.value().attr("hx-delete").unwrap(), "/todo/42");
    }
}
