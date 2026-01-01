use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn ActionButtons(
    state: Signal<AppState>,
    on_save: EventHandler<()>,
    on_apply: EventHandler<()>,
) -> Element {
    let is_loading = state.read().is_loading;

    rsx! {
        div { class: "button-group",
            button {
                class: "secondary",
                disabled: is_loading,
                onclick: move |_| on_save.call(()),
                "Save"
            }
            button {
                class: "primary",
                disabled: is_loading,
                onclick: move |_| on_apply.call(()),
                if is_loading { "Applying..." } else { "Apply" }
            }
        }
    }
}
