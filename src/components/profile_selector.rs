use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn ProfileSelector(
    state: Signal<AppState>,
    disabled: bool,
    on_profile_change: EventHandler<String>,
    on_new_profile: EventHandler<()>,
    on_name_change: EventHandler<String>,
    on_delete: EventHandler<()>,
) -> Element {
    let (profiles, selected_id, current_name, has_profile) = {
        let state = state.read();
        let profiles = state
            .sorted_profiles()
            .into_iter()
            .map(|p| (p.id.clone(), p.name.clone()))
            .collect::<Vec<_>>();
        let selected_id = state.selected_profile_id.clone().unwrap_or_default();
        let current_name = state.current_profile_name.clone();
        let has_profile = state.selected_profile_id.is_some();
        (profiles, selected_id, current_name, has_profile)
    };

    let has_profiles = !profiles.is_empty();

    rsx! {
        div { class: "profile-selector",
            select {
                class: "profile-dropdown",
                disabled: disabled,
                value: "{selected_id}",
                onchange: move |evt: Event<FormData>| {
                    on_profile_change.call(evt.value());
                },
                if !has_profiles {
                    option { value: "", disabled: true, selected: true, "(No profiles)" }
                }
                for (id, name) in profiles {
                    option { value: "{id}", "{name}" }
                }
            }
            input {
                r#type: "text",
                class: "profile-name-input",
                placeholder: "Profile Name",
                disabled: disabled || !has_profile,
                value: "{current_name}",
                oninput: move |evt: Event<FormData>| {
                    on_name_change.call(evt.value());
                },
            }
            button {
                class: "secondary new-profile-btn",
                disabled: disabled,
                onclick: move |_| on_new_profile.call(()),
                "New"
            }
            button {
                class: "secondary danger delete-btn",
                disabled: disabled || !has_profile,
                onclick: move |_| on_delete.call(()),
                "Delete"
            }
        }
    }
}
