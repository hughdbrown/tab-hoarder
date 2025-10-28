/// UI module exports
use yew::prelude::*;

// Placeholder modules for now
pub mod popup {
    use yew::prelude::*;

    #[function_component(App)]
    pub fn app() -> Html {
        html! {
            <div>{"Tab Hoarder Popup - Coming Soon"}</div>
        }
    }
}

pub mod collapsed {
    use yew::prelude::*;

    #[function_component(CollapsedViewer)]
    pub fn collapsed_viewer() -> Html {
        html! {
            <div>{"Collapsed Tabs Viewer - Coming Soon"}</div>
        }
    }
}
