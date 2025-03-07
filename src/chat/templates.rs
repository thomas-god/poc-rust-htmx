use maud::{Markup, html};

use super::state::ChatMessage;

pub fn username_form() -> Markup {
    html! {
    form.mx-auto #username-form
        ws-send
        fiedlset.fieldset.w-xs.bg-base-200.border.border-base-300.p-4.mt-8.rounded-box {
            legend.fieldset-legend { "Choose a username" }
            div.join {
                input.input.join-item #todo type="text" name="username" {}
                button.btn.btn-primary.join-item {"Rejoindre"}
            }
        }
    }
}

pub fn chat(user: &str, messages: Vec<ChatMessage>) -> Markup {
    html! {
        div #username-form hx-swap-oob="true" {
            div.chat-container {
                div #messages {
                    @for message in messages {
                        (chat_message(user, message))
                    }
                }
                (new_message_form())
            }
        }
    }
}

pub fn chat_message(user: &str, message: ChatMessage) -> Markup {
    let class = if message.username == user {
        "chat chat-end"
    } else {
        "chat chat-start"
    };
    html! {
        div class=(class) {
            div.chat-bubble {
                (message.content)
            }
        }
    }
}

pub fn new_chat_message(user: &str, message: ChatMessage) -> Markup {
    html! {
        div hx-swap-oob="beforeend:#messages" {
            (chat_message(user, message))
        }
    }
}

pub fn new_message_form() -> Markup {
    html! {
    form.mx-auto #new-message
        ws-send
        hx-on::ws-after-send="document.querySelector('#new-message').reset()"
        fiedlset.fieldset.w-xs.bg-base-200.border.border-base-300.p-4.mt-8.rounded-box {
            legend.fieldset-legend { "Nouveau message" }
            div.join {
                input.input.join-item #msg-input type="text" name="content" {}
                button.btn.btn-primary.join-item {"Envoyer"}
            }
        }
    }
}
