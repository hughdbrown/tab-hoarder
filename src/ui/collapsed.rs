/// Collapsed tabs viewer page

use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, console};
use patternfly_yew::prelude::*;
use crate::storage::StorageData;
use crate::tab_data::{CollapsedSession, SavedTab};
use std::collections::HashMap;

// Import JS bridge functions
#[wasm_bindgen(module = "/collapsed.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn getStorage(key: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn setStorage(key: &str, value: JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn restoreTabs(tabs: JsValue, progress_callback: &js_sys::Function) -> Result<(), JsValue>;

    fn exportToFile(data: &str, filename: &str);
}

#[derive(Clone, PartialEq)]
enum ViewState {
    Loading,
    Idle,
    Restoring(u8, String), // progress, message
    Error(String),
}

#[function_component(CollapsedViewer)]
pub fn collapsed_viewer() -> Html {
    let state = use_state(|| ViewState::Loading);
    let storage = use_state(|| StorageData::new());
    let search_query = use_state(|| String::new());
    let editing_session = use_state(|| None::<String>); // session ID being edited
    let edit_input_value = use_state(|| String::new());

    // Load storage on mount
    {
        let state = state.clone();
        let storage = storage.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match load_storage().await {
                    Ok(data) => {
                        storage.set(data);
                        state.set(ViewState::Idle);
                    }
                    Err(e) => {
                        state.set(ViewState::Error(format!("Failed to load: {}", e)));
                    }
                }
            });
            || ()
        });
    }

    // Search handler
    let on_search_input = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                search_query.set(input.value());
            }
        })
    };

    // Delete session handler
    let on_delete_session = {
        let storage = storage.clone();
        let state = state.clone();

        Callback::from(move |session_id: String| {
            let mut new_storage = (*storage).clone();
            new_storage.remove_session(&session_id);
            storage.set(new_storage.clone());

            let state = state.clone();
            spawn_local(async move {
                if let Err(e) = save_storage(&new_storage).await {
                    state.set(ViewState::Error(format!("Failed to save: {}", e)));
                }
            });
        })
    };

    // Start editing session name
    let on_start_edit = {
        let editing_session = editing_session.clone();
        let edit_input_value = edit_input_value.clone();

        Callback::from(move |(session_id, current_name): (String, String)| {
            editing_session.set(Some(session_id));
            edit_input_value.set(current_name);
        })
    };

    // Save edited session name
    let on_save_edit = {
        let editing_session = editing_session.clone();
        let edit_input_value = edit_input_value.clone();
        let storage = storage.clone();
        let state = state.clone();

        Callback::from(move |_| {
            if let Some(session_id) = (*editing_session).clone() {
                let new_name = (*edit_input_value).clone();
                let mut new_storage = (*storage).clone();

                if new_storage.update_session_name(&session_id, new_name) {
                    storage.set(new_storage.clone());

                    let state = state.clone();
                    spawn_local(async move {
                        if let Err(e) = save_storage(&new_storage).await {
                            state.set(ViewState::Error(format!("Failed to save: {}", e)));
                        }
                    });
                }

                editing_session.set(None);
            }
        })
    };

    // Cancel editing
    let on_cancel_edit = {
        let editing_session = editing_session.clone();
        Callback::from(move |_| {
            editing_session.set(None);
        })
    };

    // Edit input change
    let on_edit_input = {
        let edit_input_value = edit_input_value.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                edit_input_value.set(input.value());
            }
        })
    };

    // Restore entire session
    let on_restore_session = {
        let state = state.clone();

        Callback::from(move |session: CollapsedSession| {
            let state = state.clone();
            state.set(ViewState::Restoring(0, "Restoring tabs...".to_string()));

            spawn_local(async move {
                match restore_session_tabs(&session.tabs, state.clone()).await {
                    Ok(_) => {
                        state.set(ViewState::Idle);
                    }
                    Err(e) => {
                        state.set(ViewState::Error(format!("Restore failed: {}", e)));
                    }
                }
            });
        })
    };

    // Restore individual tab
    let on_restore_tab = {
        let state = state.clone();

        Callback::from(move |tab: SavedTab| {
            let state = state.clone();

            spawn_local(async move {
                match restore_session_tabs(&vec![tab], state.clone()).await {
                    Ok(_) => {}
                    Err(e) => {
                        state.set(ViewState::Error(format!("Restore failed: {}", e)));
                    }
                }
            });
        })
    };

    // Delete individual tab
    let on_delete_tab = {
        let storage = storage.clone();
        let state = state.clone();

        Callback::from(move |(session_id, tab_url): (String, String)| {
            let mut new_storage = (*storage).clone();

            // Find session and remove tab
            if let Some(session) = new_storage.sessions.iter_mut().find(|s| s.id == session_id) {
                session.tabs.retain(|t| t.url != tab_url);

                // If no tabs left, remove session
                if session.tabs.is_empty() {
                    new_storage.remove_session(&session_id);
                }

                storage.set(new_storage.clone());

                let state = state.clone();
                spawn_local(async move {
                    if let Err(e) = save_storage(&new_storage).await {
                        state.set(ViewState::Error(format!("Failed to save: {}", e)));
                    }
                });
            }
        })
    };

    // Export all sessions
    let on_export = {
        let storage = storage.clone();

        Callback::from(move |_| {
            match serde_json::to_string_pretty(&*storage) {
                Ok(json) => {
                    let filename = format!("tab-hoarder-export-{}.json", js_sys::Date::now() as i64);
                    exportToFile(&json, &filename);
                }
                Err(e) => {
                    console::log_1(&format!("Export failed: {:?}", e).into());
                }
            }
        })
    };

    // Export single session
    let on_export_session = {
        Callback::from(move |session: CollapsedSession| {
            match serde_json::to_string_pretty(&session) {
                Ok(json) => {
                    let filename = format!("session-{}.json", session.id);
                    exportToFile(&json, &filename);
                }
                Err(e) => {
                    console::log_1(&format!("Export failed: {:?}", e).into());
                }
            }
        })
    };

    // Filter sessions by search query
    let filtered_sessions: Vec<CollapsedSession> = if search_query.is_empty() {
        storage.sessions.clone()
    } else {
        let query = search_query.to_lowercase();
        storage
            .sessions
            .iter()
            .filter(|session| {
                session.name.to_lowercase().contains(&query)
                    || session.tabs.iter().any(|tab| {
                        tab.url.to_lowercase().contains(&query)
                            || tab.title.to_lowercase().contains(&query)
                            || tab.domain.to_lowercase().contains(&query)
                    })
            })
            .cloned()
            .collect()
    };

    html! {
        <div class="container">
            <div class="header">
                <h1 class="main-title">{"Collapsed Tabs"}</h1>
                <Button onclick={on_export} variant={ButtonVariant::Secondary}>
                    {"📥 Export All"}
                </Button>
            </div>

            // Status display
            {match &*state {
                ViewState::Loading => html! {
                    <div class="loading-text-center">
                        <Spinner />
                        <p class="loading-text">{"Loading sessions..."}</p>
                    </div>
                },
                ViewState::Restoring(progress, msg) => html! {
                    <div class="message-container">
                        <p class="message-text">{msg}</p>
                        <Progress value={*progress as f64} />
                    </div>
                },
                ViewState::Error(err) => html! {
                    <Alert r#type={AlertType::Danger} title={"Error"} inline={true}>
                        {err.clone()}
                    </Alert>
                },
                ViewState::Idle => html! {}
            }}

            // Search bar
            <div class="search-container">
                <input
                    type="text"
                    placeholder="Search sessions, domains, or URLs..."
                    value={(*search_query).clone()}
                    oninput={on_search_input}
                    class="search-input"
                />
            </div>

            // Sessions list
            if filtered_sessions.is_empty() {
                <div class="empty-state">
                    if search_query.is_empty() {
                        <p>{"No collapsed sessions yet."}</p>
                        <p class="empty-state-hint">{"Use the popup to collapse tabs."}</p>
                    } else {
                        <p>{"No sessions match your search."}</p>
                    }
                </div>
            } else {
                <div class="sessions-list">
                    {for filtered_sessions.iter().map(|session| {
                        let is_editing = (*editing_session).as_ref() == Some(&session.id);

                        html! {
                            <SessionCard
                                session={session.clone()}
                                is_editing={is_editing}
                                edit_value={(*edit_input_value).clone()}
                                on_delete={on_delete_session.clone()}
                                on_restore={on_restore_session.clone()}
                                on_export={on_export_session.clone()}
                                on_start_edit={on_start_edit.clone()}
                                on_save_edit={on_save_edit.clone()}
                                on_cancel_edit={on_cancel_edit.clone()}
                                on_edit_input={on_edit_input.clone()}
                                on_restore_tab={on_restore_tab.clone()}
                                on_delete_tab={on_delete_tab.clone()}
                            />
                        }
                    })}
                </div>
            }

            // Footer stats
            <div class="footer">
                {format!("{} sessions • {} total tabs",
                    storage.sessions.len(),
                    storage.sessions.iter().map(|s| s.tabs.len()).sum::<usize>()
                )}
            </div>
        </div>
    }
}

// Session card component
#[derive(Properties, PartialEq)]
struct SessionCardProps {
    session: CollapsedSession,
    is_editing: bool,
    edit_value: String,
    on_delete: Callback<String>,
    on_restore: Callback<CollapsedSession>,
    on_export: Callback<CollapsedSession>,
    on_start_edit: Callback<(String, String)>,
    on_save_edit: Callback<()>,
    on_cancel_edit: Callback<()>,
    on_edit_input: Callback<InputEvent>,
    on_restore_tab: Callback<SavedTab>,
    on_delete_tab: Callback<(String, String)>,
}

#[function_component(SessionCard)]
fn session_card(props: &SessionCardProps) -> Html {
    let expanded = use_state(|| false);
    let session = &props.session;

    let toggle_expanded = {
        let expanded = expanded.clone();
        Callback::from(move |_| {
            expanded.set(!*expanded);
        })
    };

    // Group tabs by domain
    let mut domain_groups: HashMap<String, Vec<SavedTab>> = HashMap::new();
    for tab in &session.tabs {
        domain_groups
            .entry(tab.domain.clone())
            .or_insert_with(Vec::new)
            .push(tab.clone());
    }

    let mut domains: Vec<String> = domain_groups.keys().cloned().collect();
    domains.sort();

    let date = js_sys::Date::new(&JsValue::from_f64(session.timestamp));
    let formatted_date = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes()
    );

    html! {
        <div class="session-card">
            // Header
            <div class="session-header">
                <div class="session-title-container">
                    if props.is_editing {
                        <div class="session-title-edit-mode">
                            <input
                                type="text"
                                value={props.edit_value.clone()}
                                oninput={props.on_edit_input.clone()}
                                class="session-title-input"
                            />
                            <Button
                                onclick={props.on_save_edit.reform(|_| ())}
                            >
                                {"✓"}
                            </Button>
                            <Button
                                onclick={props.on_cancel_edit.reform(|_| ())}
                                variant={ButtonVariant::Secondary}
                            >
                                {"✗"}
                            </Button>
                        </div>
                    } else {
                        <div class="session-title-view-mode">
                            <h3
                                class="session-title"
                                onclick={props.on_start_edit.reform({
                                    let session_id = session.id.clone();
                                    let name = session.name.clone();
                                    move |_| (session_id.clone(), name.clone())
                                })}
                            >
                                {&session.name}
                            </h3>
                            <span class="edit-icon">{"✏️"}</span>
                        </div>
                    }
                    <p class="session-date">
                        {format!("{} • {} tabs", formatted_date, session.tabs.len())}
                    </p>
                </div>

                <div class="session-actions">
                    <Button
                        onclick={toggle_expanded.reform(|_| ())}
                        variant={ButtonVariant::Secondary}
                    >
                        {if *expanded { "▲ Collapse" } else { "▼ Expand" }}
                    </Button>
                    <Button
                        onclick={props.on_restore.reform({
                            let session = session.clone();
                            move |_| session.clone()
                        })}
                    >
                        {"🔄 Restore All"}
                    </Button>
                    <Button
                        onclick={props.on_export.reform({
                            let session = session.clone();
                            move |_| session.clone()
                        })}
                        variant={ButtonVariant::Secondary}
                    >
                        {"📥"}
                    </Button>
                    <Button
                        onclick={props.on_delete.reform({
                            let session_id = session.id.clone();
                            move |_| session_id.clone()
                        })}
                        variant={ButtonVariant::Danger}
                    >
                        {"🗑️"}
                    </Button>
                </div>
            </div>

            // Expanded tabs list
            if *expanded {
                <div class="tabs-container">
                    {for domains.iter().map(|domain| {
                        let tabs = domain_groups.get(domain).unwrap();
                        html! {
                            <div key={domain.clone()} class="domain-group">
                                <h4 class="domain-title">
                                    {format!("{} ({})", domain, tabs.len())}
                                </h4>
                                <div class="tabs-list">
                                    {for tabs.iter().map(|tab| {
                                        let tab_clone = tab.clone();
                                        let session_id = session.id.clone();
                                        let tab_url = tab.url.clone();

                                        html! {
                                            <div key={tab.url.clone()} class="tab-item">
                                                <div class="tab-content">
                                                    <div class="tab-title">
                                                        {if tab.pinned { "📌 " } else { "" }}
                                                        {&tab.title}
                                                    </div>
                                                    <div class="tab-url">
                                                        {&tab.url}
                                                    </div>
                                                </div>
                                                <div class="tab-actions">
                                                    <Button
                                                        onclick={props.on_restore_tab.reform(move |_| tab_clone.clone())}
                                                        size={ButtonSize::Small}
                                                    >
                                                        {"🔄"}
                                                    </Button>
                                                    <Button
                                                        onclick={props.on_delete_tab.reform(move |_| (session_id.clone(), tab_url.clone()))}
                                                        variant={ButtonVariant::Danger}
                                                        size={ButtonSize::Small}
                                                    >
                                                        {"✗"}
                                                    </Button>
                                                </div>
                                            </div>
                                        }
                                    })}
                                </div>
                            </div>
                        }
                    })}
                </div>
            }
        </div>
    }
}

// Helper functions

async fn load_storage() -> Result<StorageData, String> {
    let storage_js = getStorage("tab_hoarder_data")
        .await
        .map_err(|e| format!("Failed to get storage: {:?}", e))?;

    if storage_js.is_null() || storage_js.is_undefined() {
        Ok(StorageData::new())
    } else {
        serde_wasm_bindgen::from_value(storage_js)
            .map_err(|e| format!("Failed to parse storage: {:?}", e))
    }
}

async fn save_storage(storage: &StorageData) -> Result<(), String> {
    let storage_js = serde_wasm_bindgen::to_value(storage)
        .map_err(|e| format!("Failed to serialize storage: {:?}", e))?;

    setStorage("tab_hoarder_data", storage_js)
        .await
        .map_err(|e| format!("Failed to save storage: {:?}", e))
}

async fn restore_session_tabs(tabs: &[SavedTab], state: UseStateHandle<ViewState>) -> Result<(), String> {
    let progress_callback = Closure::wrap(Box::new(move |progress: u8| {
        state.set(ViewState::Restoring(progress, "Restoring tabs...".to_string()));
    }) as Box<dyn Fn(u8)>);

    let tabs_js = serde_wasm_bindgen::to_value(tabs)
        .map_err(|e| format!("Failed to serialize tabs: {:?}", e))?;

    restoreTabs(tabs_js, progress_callback.as_ref().unchecked_ref())
        .await
        .map_err(|e| format!("Restore failed: {:?}", e))?;

    Ok(())
}
