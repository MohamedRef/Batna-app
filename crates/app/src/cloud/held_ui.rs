//! Settings › Sync: the changes Batna Cloud refused, or this build couldn't
//! read, with a way to send them again or let them go (audit F12, F13;
//! `store/held.rs`).

use dioxus::prelude::*;

use super::Command;
use super::ui::Cloud;
use crate::i18n::cloud as tx;

#[component]
pub fn HeldCard(held: u64) -> Element {
    let cloud = use_context::<Cloud>();
    let mut asking = use_signal(|| false);
    rsx! {
        crate::ui::Card { class: "sync-held", role: "status",
            h2 { {tx::HELD_TITLE} }
            p { class: "small", {tx::held(held)} }
            if *asking.read() {
                p { class: "small", {tx::HELD_FORGET_SURE} }
                div { class: "sync-actions",
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| {
                            asking.set(false);
                            cloud.send(Command::ForgetHeld);
                        },
                        {tx::HELD_FORGET_YES}
                    }
                    button { class: "btn btn-primary", onclick: move |_| asking.set(false), {tx::HELD_FORGET_NO} }
                }
            } else {
                div { class: "sync-actions",
                    button { class: "btn btn-primary", onclick: move |_| cloud.send(Command::SendHeldAgain), {tx::HELD_SEND_AGAIN} }
                    button { class: "btn btn-ghost", onclick: move |_| asking.set(true), {tx::HELD_FORGET} }
                }
            }
        }
    }
}
