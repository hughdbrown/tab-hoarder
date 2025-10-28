/// Collapsed tabs viewer page

use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, console};
use crate::ui::components::*;
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
        <div style="max-width: 1200px; margin: 0 auto; padding: 20px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
                <h1 style="margin: 0; font-size: 28px; color: #333;">{"Collapsed Tabs"}</h1>
                <Button onclick={on_export} variant={ButtonVariant::Secondary}>
                    {"📥 Export All"}
                </Button>
            </div>

            // Status display
            {match &*state {
                ViewState::Loading => html! {
                    <Spinner message={"Loading sessions...".to_string()} />
                },
                ViewState::Restoring(progress, msg) => html! {
                    <div style="margin-bottom: 20px;">
                        <p style="color: #666; margin-bottom: 5px;">{msg}</p>
                        <ProgressBar progress={*progress} />
                    </div>
                },
                ViewState::Error(err) => html! {
                    <Alert message={err.clone()} alert_type={AlertType::Error} />
                },
                ViewState::Idle => html! {}
            }}

            // Search bar
            <div style="margin-bottom: 20px;">
                <input
                    type="text"
                    placeholder="Search sessions, domains, or URLs..."
                    value={(*search_query).clone()}
                    oninput={on_search_input}
                    style="width: 100%; padding: 12px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px;"
                />
            </div>

            // Sessions list
            if filtered_sessions.is_empty() {
                <div style="text-align: center; padding: 40px; color: #999;">
                    if search_query.is_empty() {
                        <p>{"No collapsed sessions yet."}</p>
                        <p style="font-size: 12px;">{"Use the popup to collapse tabs."}</p>
                    } else {
                        <p>{"No sessions match your search."}</p>
                    }
                </div>
            } else {
                <div style="display: flex; flex-direction: column; gap: 20px;">
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
            <div style="margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; color: #999; font-size: 12px; text-align: center;">
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
        <div style="border: 1px solid #ddd; border-radius: 8px; padding: 20px; background-color: white; box-shadow: 0 2px 4px rgba(0,0,0,0.05);">
            // Header
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                <div style="flex: 1;">
                    if props.is_editing {
                        <div style="display: flex; gap: 10px; align-items: center;">
                            <input
                                type="text"
                                value={props.edit_value.clone()}
                                oninput={props.on_edit_input.clone()}
                                style="flex: 1; padding: 8px; border: 1px solid #5B4FE8; border-radius: 4px; font-size: 16px; font-weight: 600;"
                            />
                            <button
                                onclick={props.on_save_edit.reform(|_| ())}
                                style="padding: 8px 16px; background-color: #5B4FE8; color: white; border: none; border-radius: 4px; cursor: pointer;"
                            >
                                {"✓"}
                            </button>
                            <button
                                onclick={props.on_cancel_edit.reform(|_| ())}
                                style="padding: 8px 16px; background-color: #ddd; color: #333; border: none; border-radius: 4px; cursor: pointer;"
                            >
                                {"✗"}
                            </button>
                        </div>
                    } else {
                        <div style="display: flex; gap: 10px; align-items: center;">
                            <h3
                                style="margin: 0; font-size: 18px; color: #333; cursor: pointer;"
                                onclick={props.on_start_edit.reform({
                                    let session_id = session.id.clone();
                                    let name = session.name.clone();
                                    move |_| (session_id.clone(), name.clone())
                                })}
                            >
                                {&session.name}
                            </h3>
                            <span style="font-size: 14px; color: #999;">{"✏️"}</span>
                        </div>
                    }
                    <p style="margin: 5px 0 0 0; font-size: 12px; color: #999;">
                        {format!("{} • {} tabs", formatted_date, session.tabs.len())}
                    </p>
                </div>

                <div style="display: flex; gap: 10px;">
                    <button
                        onclick={toggle_expanded.reform(|_| ())}
                        style="padding: 8px 16px; background-color: #e0e0e0; color: #333; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;"
                    >
                        {if *expanded { "▲ Collapse" } else { "▼ Expand" }}
                    </button>
                    <button
                        onclick={props.on_restore.reform({
                            let session = session.clone();
                            move |_| session.clone()
                        })}
                        style="padding: 8px 16px; background-color: #5B4FE8; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;"
                    >
                        {"🔄 Restore All"}
                    </button>
                    <button
                        onclick={props.on_export.reform({
                            let session = session.clone();
                            move |_| session.clone()
                        })}
                        style="padding: 8px 16px; background-color: #e0e0e0; color: #333; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;"
                    >
                        {"📥"}
                    </button>
                    <button
                        onclick={props.on_delete.reform({
                            let session_id = session.id.clone();
                            move |_| session_id.clone()
                        })}
                        style="padding: 8px 16px; background-color: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 14px;"
                    >
                        {"🗑️"}
                    </button>
                </div>
            </div>

            // Expanded tabs list
            if *expanded {
                <div style="margin-top: 15px; padding-top: 15px; border-top: 1px solid #eee;">
                    {for domains.iter().map(|domain| {
                        let tabs = domain_groups.get(domain).unwrap();
                        html! {
                            <div key={domain.clone()} style="margin-bottom: 20px;">
                                <h4 style="margin: 0 0 10px 0; font-size: 14px; color: #5B4FE8; font-weight: 600;">
                                    {format!("{} ({})", domain, tabs.len())}
                                </h4>
                                <div style="display: flex; flex-direction: column; gap: 5px;">
                                    {for tabs.iter().map(|tab| {
                                        let tab_clone = tab.clone();
                                        let session_id = session.id.clone();
                                        let tab_url = tab.url.clone();

                                        html! {
                                            <div key={tab.url.clone()} style="display: flex; justify-content: space-between; align-items: center; padding: 8px; background-color: #f9f9f9; border-radius: 4px; font-size: 13px;">
                                                <div style="flex: 1; min-width: 0;">
                                                    <div style="font-weight: 500; color: #333; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
                                                        {if tab.pinned { "📌 " } else { "" }}
                                                        {&tab.title}
                                                    </div>
                                                    <div style="color: #999; font-size: 11px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
                                                        {&tab.url}
                                                    </div>
                                                </div>
                                                <div style="display: flex; gap: 5px; margin-left: 10px;">
                                                    <button
                                                        onclick={props.on_restore_tab.reform(move |_| tab_clone.clone())}
                                                        style="padding: 4px 8px; background-color: #5B4FE8; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 11px;"
                                                    >
                                                        {"🔄"}
                                                    </button>
                                                    <button
                                                        onclick={props.on_delete_tab.reform(move |_| (session_id.clone(), tab_url.clone()))}
                                                        style="padding: 4px 8px; background-color: #f44336; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 11px;"
                                                    >
                                                        {"✗"}
                                                    </button>
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
