use crate::dns::AddressFamily;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn DnsTabs(state: Signal<AppState>, on_change: EventHandler<AddressFamily>) -> Element {
    let active_tab = state.read().active_tab;
    let interface = state.read().selected_interface().cloned();

    if let Some(iface) = interface {
        rsx! {
            div { class: "tabs",
                if iface.has_ipv4 {
                    button {
                        class: if active_tab == AddressFamily::IPv4 { "tab active" } else { "tab" },
                        onclick: move |_| on_change.call(AddressFamily::IPv4),
                        "IPv4"
                    }
                }
                if iface.has_ipv6 {
                    button {
                        class: if active_tab == AddressFamily::IPv6 { "tab active" } else { "tab" },
                        onclick: move |_| on_change.call(AddressFamily::IPv6),
                        "IPv6"
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}
