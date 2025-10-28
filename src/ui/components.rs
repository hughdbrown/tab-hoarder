/// Reusable UI components

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProgressBarProps {
    pub progress: u8, // 0-100
}

#[function_component(ProgressBar)]
pub fn progress_bar(props: &ProgressBarProps) -> Html {
    let progress = props.progress.min(100);

    html! {
        <div style="width: 100%; background-color: #e0e0e0; border-radius: 4px; height: 20px; overflow: hidden; margin: 10px 0;">
            <div style={format!("width: {}%; background-color: #5B4FE8; height: 100%; transition: width 0.3s ease; display: flex; align-items: center; justify-content: center; color: white; font-size: 12px; font-weight: bold;", progress)}>
                {format!("{}%", progress)}
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SpinnerProps {
    #[prop_or_default]
    pub message: Option<String>,
}

#[function_component(Spinner)]
pub fn spinner(props: &SpinnerProps) -> Html {
    html! {
        <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 20px;">
            <div style="
                border: 4px solid #f3f3f3;
                border-top: 4px solid #5B4FE8;
                border-radius: 50%;
                width: 40px;
                height: 40px;
                animation: spin 1s linear infinite;
            "></div>
            if let Some(msg) = &props.message {
                <p style="margin-top: 10px; color: #666;">{msg}</p>
            }
            <style>
                {"@keyframes spin { 0% { transform: rotate(0deg); } 100% { transform: rotate(360deg); } }"}
            </style>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    pub onclick: Callback<MouseEvent>,
    pub children: Children,
    #[prop_or(false)]
    pub disabled: bool,
    #[prop_or_default]
    pub variant: ButtonVariant,
}

#[derive(PartialEq, Clone)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
}

impl Default for ButtonVariant {
    fn default() -> Self {
        ButtonVariant::Primary
    }
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    let base_style = "padding: 10px 20px; border: none; border-radius: 4px; font-size: 14px; cursor: pointer; font-weight: 500; transition: all 0.2s;";

    let variant_style = match props.variant {
        ButtonVariant::Primary => "background-color: #5B4FE8; color: white;",
        ButtonVariant::Secondary => "background-color: #e0e0e0; color: #333;",
        ButtonVariant::Danger => "background-color: #f44336; color: white;",
    };

    let disabled_style = if props.disabled {
        "opacity: 0.5; cursor: not-allowed;"
    } else {
        ""
    };

    let style = format!("{} {} {}", base_style, variant_style, disabled_style);

    html! {
        <button
            onclick={props.onclick.clone()}
            disabled={props.disabled}
            style={style}
        >
            {props.children.clone()}
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub struct AlertProps {
    pub message: String,
    #[prop_or_default]
    pub alert_type: AlertType,
}

#[derive(PartialEq, Clone)]
pub enum AlertType {
    Info,
    Success,
    Warning,
    Error,
}

impl Default for AlertType {
    fn default() -> Self {
        AlertType::Info
    }
}

#[function_component(Alert)]
pub fn alert(props: &AlertProps) -> Html {
    let (bg_color, border_color) = match props.alert_type {
        AlertType::Info => ("#e3f2fd", "#2196f3"),
        AlertType::Success => ("#e8f5e9", "#4caf50"),
        AlertType::Warning => ("#fff3e0", "#ff9800"),
        AlertType::Error => ("#ffebee", "#f44336"),
    };

    html! {
        <div style={format!("padding: 12px; border-radius: 4px; background-color: {}; border-left: 4px solid {}; margin: 10px 0;", bg_color, border_color)}>
            <p style="margin: 0; color: #333; font-size: 14px;">{&props.message}</p>
        </div>
    }
}
