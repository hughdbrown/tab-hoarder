/// Popup UI for Tab Hoarder extension

use yew::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use patternfly_yew::prelude::*;
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
enum ActiveTab {
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
    let active_tab = use_state(|| ActiveTab::Search);

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
        move |tab: ActiveTab| {
            let active_tab = active_tab.clone();
            Callback::from(move |_| {
                active_tab.set(tab.clone());
            })
        }
    };

    html! {
        <div class="padding-20">
            <h1 class="popup-title">{"Tab Hoarder"}</h1>

            // Storage warning
            if let Some(warning) = (*storage_warning).clone() {
                <Alert r#type={AlertType::Warning} title={warning} inline={true}>
                </Alert>
            }

            // Tab navigation
            <div class="pf-v5-c-tabs tabs-nav">
                <ul class="pf-v5-c-tabs__list">
                    <li class={if *active_tab == ActiveTab::Search { "pf-v5-c-tabs__item pf-m-current" } else { "pf-v5-c-tabs__item" }}>
                        <button
                            class="pf-v5-c-tabs__link"
                            onclick={on_tab_click(ActiveTab::Search)}
                        >
                            <span class="pf-v5-c-tabs__item-text">{"Search"}</span>
                        </button>
                    </li>
                    <li class={if *active_tab == ActiveTab::SortUnique { "pf-v5-c-tabs__item pf-m-current" } else { "pf-v5-c-tabs__item" }}>
                        <button
                            class="pf-v5-c-tabs__link"
                            onclick={on_tab_click(ActiveTab::SortUnique)}
                        >
                            <span class="pf-v5-c-tabs__item-text">{"Sort/unique"}</span>
                        </button>
                    </li>
                    <li class={if *active_tab == ActiveTab::Archive { "pf-v5-c-tabs__item pf-m-current" } else { "pf-v5-c-tabs__item" }}>
                        <button
                            class="pf-v5-c-tabs__link"
                            onclick={on_tab_click(ActiveTab::Archive)}
                        >
                            <span class="pf-v5-c-tabs__item-text">{"Archive"}</span>
                        </button>
                    </li>
                    <li class={if *active_tab == ActiveTab::Analyze { "pf-v5-c-tabs__item pf-m-current" } else { "pf-v5-c-tabs__item" }}>
                        <button
                            class="pf-v5-c-tabs__link"
                            onclick={on_tab_click(ActiveTab::Analyze)}
                        >
                            <span class="pf-v5-c-tabs__item-text">{"Analyze"}</span>
                        </button>
                    </li>
                </ul>
            </div>

            // Status display
            {match &*state {
                AppState::Loading(msg) => html! {
                    <div class="loading-text-center">
                        <Spinner />
                        <p class="loading-text">{msg}</p>
                    </div>
                },
                AppState::Processing(progress, msg) => html! {
                    <div class="message-top-margin">
                        <p class="message-text">{msg}</p>
                        <Progress value={*progress as f64} />
                    </div>
                },
                AppState::Error(err) => html! {
                    <div class="message-top-margin">
                        <Alert r#type={AlertType::Danger} title={"Error"} inline={true}>
                            {err.clone()}
                        </Alert>
                    </div>
                },
                AppState::Idle => html! {}
            }}

            // Tab content
            <div class="tab-pane-content">
                {match &*active_tab {
                    ActiveTab::Search => html! {
                        <div class="flex-column-gap">
                            // Empty for now
                        </div>
                    },
                    ActiveTab::SortUnique => html! {
                        <div class="flex-column-gap">
                            <Button onclick={on_sort} disabled={is_busy} variant={ButtonVariant::Secondary} block={true}>
                                {"🔤 Sort Tabs by Domain"}
                            </Button>
                            <Button onclick={on_unique} disabled={is_busy} variant={ButtonVariant::Secondary} block={true}>
                                {"🗑️ Make Tabs Unique"}
                            </Button>
                        </div>
                    },
                    ActiveTab::Archive => html! {
                        <div class="flex-column-gap">
                            <Button onclick={on_collapse} disabled={is_busy} variant={ButtonVariant::Secondary} block={true}>
                                {"💾 Collapse Tabs"}
                            </Button>
                            <Button onclick={on_view_collapsed} disabled={is_busy} variant={ButtonVariant::Secondary} block={true}>
                                {"📂 View Collapsed Tabs"}
                            </Button>
                        </div>
                    },
                    ActiveTab::Analyze => html! {
                        <div class="flex-column-gap">
                            <Button onclick={on_analyze} disabled={is_busy} variant={ButtonVariant::Secondary} block={true}>
                                {"📊 Analyze Domains"}
                            </Button>

                            // Domain stats (only show when expanded)
                            if *is_domains_expanded && !domain_stats.is_empty() {
                                <div class="stats-container">
                                    <h2 class="stats-title">{"Top 10 Domains"}</h2>
                                    <div class="stats-box">
                                        {for domain_stats.iter().map(|stat| html! {
                                            <div class="stat-item">
                                                <span class="stat-domain">{&stat.domain}</span>
                                                <span class="stat-count">{stat.count}</span>
                                            </div>
                                        })}
                                    </div>
                                </div>
                            }
                        </div>
                    },
                }}
            </div>

            <p class="footer-popup">
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
