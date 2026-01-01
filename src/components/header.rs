use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::md_action_icons::MdDns;

#[component]
pub fn Header() -> Element {
    rsx! {
        div { class: "header",
            Icon {
                width: 28,
                height: 28,
                icon: MdDns
            }
            h1 { "Windows DNS Switcher" }
        }
    }
}
