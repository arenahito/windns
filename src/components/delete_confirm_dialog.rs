use dioxus::prelude::*;

#[component]
pub fn DeleteConfirmDialog(
    profile_name: String,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "dialog-overlay",
            div { class: "delete-confirm-dialog",
                h3 { "Delete Profile" }
                p { "Are you sure you want to delete \"{profile_name}\"?" }
                div { class: "dialog-buttons",
                    button {
                        class: "secondary",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "primary danger",
                        onclick: move |_| on_confirm.call(()),
                        "Delete"
                    }
                }
            }
        }
    }
}
