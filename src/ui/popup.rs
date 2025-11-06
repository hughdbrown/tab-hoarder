/// Popup UI for Tab Hoarder extension

use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use crate::ui::components::*;
use crate::domain::{count_domains, get_top_domains};
use crate::operations::{sort_tabs_by_domain, make_tabs_unique};
use crate::tab_data::TabInfo;
use crate::storage::StorageData;
use crate::tab_data::SavedTab;
use uuid::Uuid;

// Import JS bridge functions
#[wasm_bindgen(module = "/popup.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn getCurrentWindowTabs() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn sortTabsByDomain(tab_ids: JsValue, progress_callback: &js_sys::Function) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn removeTabs(tab_ids: JsValue, progress_callback: &js_sys::Function) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn closeTabs(tab_ids: JsValue, progress_callback: &js_sys::Function) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn getStorage(key: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn setStorage(key: &str, value: JsValue) -> Result<(), JsValue>;

    #[wasm_bindgen(catch)]
    async fn getStorageQuota() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn openCollapsedViewer() -> Result<(), JsValue>;
}

#[derive(Clone, PartialEq)]
struct DomainStat {
    domain: String,
    count: usize,
}

#[derive(Clone, PartialEq)]
enum AppState {
    Idle,
    Loading(String),
    Processing(u8, String), // progress, message
    Error(String),
}

#[derive(Clone, PartialEq)]
enum Tab {
    Search,
    SortUnique,
    Archive,
    Analyze,
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(|| AppState::Idle);
    let domain_stats = use_state(|| Vec::<DomainStat>::new());
    let storage_warning = use_state(|| None::<String>);
    let is_domains_expanded = use_state(|| false);
    let active_tab = use_state(|| Tab::Search);

    // Check storage quota on mount
    {
        let storage_warning = storage_warning.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(quota_js) = getStorageQuota().await {
                    if let Ok(quota) = serde_wasm_bindgen::from_value::<serde_json::Value>(quota_js) {
                        if let Some(percent) = quota.get("percentUsed").and_then(|v| v.as_u64()) {
                            if percent >= 90 {
                                storage_warning.set(Some(format!("Storage {}% full!", percent)));
                            }
                        }
                    }
                }
            });
            || ()
        });
    }

    // Analyze domains handler
    let on_analyze = {
        let state = state.clone();
        let domain_stats = domain_stats.clone();
        let is_domains_expanded = is_domains_expanded.clone();

        Callback::from(move |_| {
            // If already expanded with data, just collapse
            if *is_domains_expanded && !domain_stats.is_empty() {
                is_domains_expanded.set(false);
                return;
            }

            // Otherwise, analyze and expand
            let state = state.clone();
            let domain_stats = domain_stats.clone();
            let is_domains_expanded = is_domains_expanded.clone();

            state.set(AppState::Loading("Analyzing domains...".to_string()));

            spawn_local(async move {
                match get_current_tabs().await {
                    Ok(tabs) => {
                        let urls: Vec<String> = tabs.iter().map(|t| t.url.clone()).collect();
                        let counts = count_domains(&urls);
                        let top_10 = get_top_domains(&counts, 10);

                        let stats: Vec<DomainStat> = top_10
                            .into_iter()
                            .map(|(domain, count)| DomainStat { domain, count })
                            .collect();

                        domain_stats.set(stats);
                        is_domains_expanded.set(true);
                        state.set(AppState::Idle);
                    }
                    Err(e) => {
                        state.set(AppState::Error(format!("Failed to analyze: {}", e)));
                    }
                }
            });
        })
    };

    // Sort tabs handler
    let on_sort = {
        let state = state.clone();

        Callback::from(move |_| {
            let state = state.clone();

            state.set(AppState::Loading("Sorting tabs...".to_string()));

            spawn_local(async move {
                match get_current_tabs().await {
                    Ok(tabs) => {
                        let sorted = sort_tabs_by_domain(&tabs);
                        let tab_ids: Vec<i32> = sorted.iter().map(|t| t.id).collect();

                        match sort_tabs_with_progress(tab_ids, state.clone()).await {
                            Ok(_) => {
                                state.set(AppState::Idle);
                            }
                            Err(e) => {
                                state.set(AppState::Error(format!("Sort failed: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        state.set(AppState::Error(format!("Failed to get tabs: {}", e)));
                    }
                }
            });
        })
    };

    // Make unique handler
    let on_unique = {
        let state = state.clone();

        Callback::from(move |_| {
            let state = state.clone();

            state.set(AppState::Loading("Removing duplicates...".to_string()));

            spawn_local(async move {
                match get_current_tabs().await {
                    Ok(tabs) => {
                        let (_, remove_ids) = make_tabs_unique(&tabs);

                        if remove_ids.is_empty() {
                            state.set(AppState::Idle);
                            console::log_1(&"No duplicates found".into());
                        } else {
                            match remove_tabs_with_progress(remove_ids, state.clone()).await {
                                Ok(_) => {
                                    state.set(AppState::Idle);
                                }
                                Err(e) => {
                                    state.set(AppState::Error(format!("Remove failed: {}", e)));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        state.set(AppState::Error(format!("Failed to get tabs: {}", e)));
                    }
                }
            });
        })
    };

    // Collapse tabs handler
    let on_collapse = {
        let state = state.clone();

        Callback::from(move |_| {
            let state = state.clone();

            state.set(AppState::Loading("Collapsing tabs...".to_string()));

            spawn_local(async move {
                match get_current_tabs().await {
                    Ok(tabs) => {
                        // Sort and make unique before collapsing
                        let sorted = sort_tabs_by_domain(&tabs);
                        let (unique_tabs, _) = make_tabs_unique(&sorted);

                        // Create session
                        let session_id = Uuid::new_v4().to_string();
                        let now = js_sys::Date::now();
                        let date = js_sys::Date::new(&JsValue::from_f64(now));
                        let name = format!("Session {}", format_date(&date));

                        let saved_tabs: Vec<SavedTab> = unique_tabs.iter().filter_map(|tab| {
                            match crate::domain::extract_domain(&tab.url) {
                                Some(domain) => {
                                    Some(SavedTab {
                                        url: tab.url.clone(),
                                        title: tab.title.clone(),
                                        domain,
                                        pinned: tab.pinned,
                                    })
                                },
                                None => None,
                            }
                        }).collect();

                        let session = crate::tab_data::CollapsedSession {
                            id: session_id,
                            name,
                            timestamp: now,
                            tabs: saved_tabs,
                        };

                        // Save to storage
                        match save_session(session).await {
                            Ok(_) => {
                                // Close tabs
                                let tab_ids: Vec<i32> = unique_tabs.iter().map(|t| t.id).collect();
                                match close_tabs_with_progress(tab_ids, state.clone()).await {
                                    Ok(_) => {
                                        state.set(AppState::Idle);
                                    }
                                    Err(e) => {
                                        state.set(AppState::Error(format!("Failed to close: {}", e)));
                                    }
                                }
                            }
                            Err(e) => {
                                state.set(AppState::Error(format!("Failed to save: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        state.set(AppState::Error(format!("Failed to get tabs: {}", e)));
                    }
                }
            });
        })
    };

    // View collapsed tabs handler
    let on_view_collapsed = {
        Callback::from(move |_| {
            spawn_local(async move {
                let _ = openCollapsedViewer().await;
            });
        })
    };

    let is_busy = !matches!(*state, AppState::Idle);

    // Tab click handlers
    let on_tab_click = {
        let active_tab = active_tab.clone();
        move |tab: Tab| {
            let active_tab = active_tab.clone();
            Callback::from(move |_| {
                active_tab.set(tab.clone());
            })
        }
    };

    html! {
        <div style="padding: 20px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
            <h1 style="margin: 0 0 20px 0; font-size: 24px; color: #333;">{"Tab Hoarder"}</h1>

            // Storage warning
            if let Some(warning) = (*storage_warning).clone() {
                <Alert message={warning} alert_type={AlertType::Warning} />
            }

            // Tab navigation
            <div style="display: flex; border-bottom: 2px solid #e0e0e0; margin-bottom: 20px;">
                <button
                    onclick={on_tab_click(Tab::Search)}
                    style={format!(
                        "flex: 1; padding: 12px 16px; background: none; border: none; cursor: pointer; \
                        font-size: 14px; font-weight: 500; transition: all 0.2s; \
                        border-bottom: 3px solid {}; color: {};",
                        if *active_tab == Tab::Search { "#5B4FE8" } else { "transparent" },
                        if *active_tab == Tab::Search { "#5B4FE8" } else { "#666" }
                    )}
                >
                    {"Search"}
                </button>
                <button
                    onclick={on_tab_click(Tab::SortUnique)}
                    style={format!(
                        "flex: 1; padding: 12px 16px; background: none; border: none; cursor: pointer; \
                        font-size: 14px; font-weight: 500; transition: all 0.2s; \
                        border-bottom: 3px solid {}; color: {};",
                        if *active_tab == Tab::SortUnique { "#5B4FE8" } else { "transparent" },
                        if *active_tab == Tab::SortUnique { "#5B4FE8" } else { "#666" }
                    )}
                >
                    {"Sort/unique"}
                </button>
                <button
                    onclick={on_tab_click(Tab::Archive)}
                    style={format!(
                        "flex: 1; padding: 12px 16px; background: none; border: none; cursor: pointer; \
                        font-size: 14px; font-weight: 500; transition: all 0.2s; \
                        border-bottom: 3px solid {}; color: {};",
                        if *active_tab == Tab::Archive { "#5B4FE8" } else { "transparent" },
                        if *active_tab == Tab::Archive { "#5B4FE8" } else { "#666" }
                    )}
                >
                    {"Archive"}
                </button>
                <button
                    onclick={on_tab_click(Tab::Analyze)}
                    style={format!(
                        "flex: 1; padding: 12px 16px; background: none; border: none; cursor: pointer; \
                        font-size: 14px; font-weight: 500; transition: all 0.2s; \
                        border-bottom: 3px solid {}; color: {};",
                        if *active_tab == Tab::Analyze { "#5B4FE8" } else { "transparent" },
                        if *active_tab == Tab::Analyze { "#5B4FE8" } else { "#666" }
                    )}
                >
                    {"Analyze"}
                </button>
            </div>

            // Status display
            {match &*state {
                AppState::Loading(msg) => html! {
                    <Spinner message={msg.clone()} />
                },
                AppState::Processing(progress, msg) => html! {
                    <div>
                        <p style="color: #666; margin-bottom: 5px;">{msg}</p>
                        <ProgressBar progress={*progress} />
                    </div>
                },
                AppState::Error(err) => html! {
                    <Alert message={err.clone()} alert_type={AlertType::Error} />
                },
                AppState::Idle => html! {}
            }}

            // Tab content
            <div style="margin-top: 20px;">
                {match &*active_tab {
                    Tab::Search => html! {
                        <div style="display: flex; flex-direction: column; gap: 10px;">
                            // Empty for now
                        </div>
                    },
                    Tab::SortUnique => html! {
                        <div style="display: flex; flex-direction: column; gap: 10px;">
                            <Button onclick={on_sort} disabled={is_busy}>
                                {"🔤 Sort Tabs by Domain"}
                            </Button>
                            <Button onclick={on_unique} disabled={is_busy}>
                                {"🗑️ Make Tabs Unique"}
                            </Button>
                        </div>
                    },
                    Tab::Archive => html! {
                        <div style="display: flex; flex-direction: column; gap: 10px;">
                            <Button onclick={on_collapse} disabled={is_busy} variant={ButtonVariant::Secondary}>
                                {"💾 Collapse Tabs"}
                            </Button>
                            <Button onclick={on_view_collapsed} disabled={is_busy} variant={ButtonVariant::Secondary}>
                                {"📂 View Collapsed Tabs"}
                            </Button>
                        </div>
                    },
                    Tab::Analyze => html! {
                        <div style="display: flex; flex-direction: column; gap: 10px;">
                            <Button onclick={on_analyze} disabled={is_busy}>
                                {if *is_domains_expanded && !domain_stats.is_empty() {
                                    "📊 Analyze Domains ▼"
                                } else {
                                    "📊 Analyze Domains ▶"
                                }}
                            </Button>

                            // Domain stats (only show when expanded)
                            if *is_domains_expanded && !domain_stats.is_empty() {
                                <div style="margin-top: 10px;">
                                    <h2 style="font-size: 16px; color: #666; margin-bottom: 10px;">{"Top 10 Domains"}</h2>
                                    <div style="background-color: #f5f5f5; border-radius: 4px; padding: 10px;">
                                        {for domain_stats.iter().map(|stat| html! {
                                            <div style="display: flex; justify-content: space-between; padding: 5px 0; border-bottom: 1px solid #ddd;">
                                                <span style="color: #333;">{&stat.domain}</span>
                                                <span style="color: #5B4FE8; font-weight: bold;">{stat.count}</span>
                                            </div>
                                        })}
                                    </div>
                                </div>
                            }
                        </div>
                    },
                }}
            </div>

            <p style="margin-top: 20px; font-size: 12px; color: #999; text-align: center;">
                {"Tab Hoarder v0.1.0"}
            </p>
        </div>
    }
}

// Helper functions

async fn get_current_tabs() -> Result<Vec<TabInfo>, String> {
    match getCurrentWindowTabs().await {
        Ok(tabs_js) => {
            let tabs: Vec<TabInfo> = serde_wasm_bindgen::from_value(tabs_js)
                .map_err(|e| format!("Failed to parse tabs: {:?}", e))?;
            Ok(tabs)
        }
        Err(e) => Err(format!("Failed to get tabs: {:?}", e)),
    }
}

async fn sort_tabs_with_progress(tab_ids: Vec<i32>, state: UseStateHandle<AppState>) -> Result<(), String> {
    let progress_callback = Closure::wrap(Box::new(move |progress: u8| {
        state.set(AppState::Processing(progress, "Sorting tabs...".to_string()));
    }) as Box<dyn Fn(u8)>);

    let tab_ids_js = serde_wasm_bindgen::to_value(&tab_ids)
        .map_err(|e| format!("Failed to serialize: {:?}", e))?;

    sortTabsByDomain(tab_ids_js, progress_callback.as_ref().unchecked_ref())
        .await
        .map_err(|e| format!("Sort failed: {:?}", e))?;

    Ok(())
}

async fn remove_tabs_with_progress(tab_ids: Vec<i32>, state: UseStateHandle<AppState>) -> Result<(), String> {
    let progress_callback = Closure::wrap(Box::new(move |progress: u8| {
        state.set(AppState::Processing(progress, "Removing duplicates...".to_string()));
    }) as Box<dyn Fn(u8)>);

    let tab_ids_js = serde_wasm_bindgen::to_value(&tab_ids)
        .map_err(|e| format!("Failed to serialize: {:?}", e))?;

    removeTabs(tab_ids_js, progress_callback.as_ref().unchecked_ref())
        .await
        .map_err(|e| format!("Remove failed: {:?}", e))?;

    Ok(())
}

async fn close_tabs_with_progress(tab_ids: Vec<i32>, state: UseStateHandle<AppState>) -> Result<(), String> {
    let progress_callback = Closure::wrap(Box::new(move |progress: u8| {
        state.set(AppState::Processing(progress, "Closing tabs...".to_string()));
    }) as Box<dyn Fn(u8)>);

    let tab_ids_js = serde_wasm_bindgen::to_value(&tab_ids)
        .map_err(|e| format!("Failed to serialize: {:?}", e))?;

    closeTabs(tab_ids_js, progress_callback.as_ref().unchecked_ref())
        .await
        .map_err(|e| format!("Close failed: {:?}", e))?;

    Ok(())
}

async fn save_session(session: crate::tab_data::CollapsedSession) -> Result<(), String> {
    // Load existing storage
    let storage_js = getStorage("tab_hoarder_data").await
        .map_err(|e| format!("Failed to get storage: {:?}", e))?;

    let mut storage: StorageData = if storage_js.is_null() || storage_js.is_undefined() {
        StorageData::new()
    } else {
        serde_wasm_bindgen::from_value(storage_js)
            .map_err(|e| format!("Failed to parse storage: {:?}", e))?
    };

    storage.add_session(session);

    let storage_js = serde_wasm_bindgen::to_value(&storage)
        .map_err(|e| format!("Failed to serialize storage: {:?}", e))?;

    setStorage("tab_hoarder_data", storage_js)
        .await
        .map_err(|e| format!("Failed to save storage: {:?}", e))?;

    Ok(())
}

fn format_date(date: &js_sys::Date) -> String {
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        date.get_full_year(),
        date.get_month() + 1,
        date.get_date(),
        date.get_hours(),
        date.get_minutes(),
        date.get_seconds()
    )
}
