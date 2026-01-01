use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn NetworkSelector(state: Signal<AppState>, on_change: EventHandler<usize>) -> Element {
    let interfaces = state.read().interfaces.clone();
    let selected_index = state.read().selected_interface_index;

    rsx! {
        div { class: "section",
            div { class: "section-title", "Network Interface" }
            div { class: "form-group",
                label { r#for: "interface-select", "Select Network Adapter" }
                select {
                    id: "interface-select",
                    value: "{selected_index}",
                    onchange: move |evt| {
                        if let Ok(index) = evt.value().parse::<usize>() {
                            on_change.call(index);
                        }
                    },
                    for (index, interface) in interfaces.iter().enumerate() {
                        option {
                            value: "{index}",
                            selected: index == selected_index,
                            "{interface.display_name()}"
                        }
                    }
                }
            }
        }
    }
}
