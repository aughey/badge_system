pub mod app;

#[cfg(feature = "ssr")]
pub mod badge_channels;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    use leptos::*;

    console_error_panic_hook::set_once();

    mount_to_body(App);
}

pub fn format_text_for_badge(text: impl AsRef<str>) -> String {
    // truncate text to 13x5 characters
    const TEXT_LIMIT: usize = 13 * 5;
    text.as_ref()
        .chars()
        .take(TEXT_LIMIT)
        .filter(|c| c.is_ascii())
        .collect::<String>()
}
