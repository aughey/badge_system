use std::{cell::RefCell, rc::Rc};

use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use web_sys::console::log;

use crate::format_text_for_badge;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/web-badge.css"/>

        // sets the document title
        <Title text="Aughey Badge"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // create_effect(move |_| {
    //     let value = value().parse().unwrap();
    //     spawn_local(async move {
    //         update_frequency(value).await.unwrap();
    //         ()
    //     });
    // });

    view! {
        <Badge/>
    }
}

#[component]
fn Badge() -> impl IntoView {
    let screen_container = create_node_ref::<leptos::html::Div>();

    let options = [50, 100, 250, 500, 1000];
    let selected = 1000;
    let (value, set_value) = create_signal(selected.to_string());
    let (messages, set_messages) = create_signal(Vec::new());

    let display = Rc::new(RefCell::new(None));

    let input_ref = create_node_ref::<leptos::html::Textarea>();
    let get_input = {
        let input_ref = input_ref.clone();
        move || {
            input_ref
                .get()
                .map(|v| v.value())
                .unwrap_or_else(|| "".to_string())
        }
    };

    let send_text = move || {
        let text = get_input();
        let freq = value().parse().unwrap();
        spawn_local(async move {
            update_text(text.clone()).await.unwrap();
            update_frequency(freq).await.unwrap();
            set_messages.update(|m| {
                m.insert(0, format!("Sent text to the server: {}", text));
                m.insert(0, format!("Sent update rate to the server: {}", freq));
            });
            ()
        });
    };

    const INITIAL_TEXT: &str = "Enter Text Here";
    {
        let display = display.clone();
        create_effect(move |_| {
            use embedded_graphics_web_simulator::{
                display::WebSimulatorDisplay, output_settings::OutputSettingsBuilder,
            };

            let sc = screen_container.get().unwrap();
            const WIDTH: u32 = 296;
            const HEIGHT: u32 = 128;
            let output_settings = OutputSettingsBuilder::new()
                .scale(1)
                .pixel_spacing(0)
                .build();
            let mut text_display =
                WebSimulatorDisplay::new((WIDTH, HEIGHT), &output_settings, Some(&sc));

            badge_draw::draw_display(&mut text_display, INITIAL_TEXT)
                .expect("could not draw display");
            text_display.flush().expect("could not flush buffer");

            display.replace(Some(text_display));
        });
    }

    const TEXT_LIMIT: usize = 13 * 5;
    let update_display = move |text: &str| {
        //        let text = text.get();
        // strip any non-ascii characters
        if let Some(display) = display.borrow_mut().as_mut() {
            let text = format_text_for_badge(text);
            badge_draw::draw_display(display, text.as_str()).expect("could not draw text");
            display.flush().expect("could not flush buffer");
        }
    };

    view! {
        <div>
        <h1>"Badge"</h1>
        <div _ref=screen_container id="custom-container" class="badge">
        </div>
        <textarea _ref=input_ref
        on:input=move |_| update_display(get_input().as_str())>
        {INITIAL_TEXT}
        </textarea>
        <div>LED Flash Rate (ms)
         <select on:change=move |ev| {
        let new_value = event_target_value(&ev);
        set_value(new_value);
    }>
        {move || options.into_iter().map(|v| v.to_string()).map(|v| view! {
            <option selected=if v == value() { "selected" } else { "" }>
                {v}
            </option>
        }).collect_view()}
    </select>
    </div>
        <button on:click=move |_| send_text()>Send this state to Badge</button>
        <div>
        Only the most recent message is displayed on the badge.
        </div>
        </div>
        <ul>
        {move || messages().iter().map(|m| view! {
            <li>{m}</li>
        }).collect_view()}
        </ul>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}

#[server(UpdateFreq, "/updatefreq")]
async fn update_frequency(freq: u64) -> Result<String, ServerFnError> {
    use tracing::info;
    info!("Updating frequency to {freq}");
    crate::badge_channels::set_frequency(freq);
    Ok(format!("Updated frequency to {freq}"))
}

#[server(UpdateText, "/updatetext")]
async fn update_text(text: String) -> Result<String, ServerFnError> {
    use tracing::info;
    info!("Updating text to {text}");
    crate::badge_channels::set_text(&text);
    Ok(format!("Updated text to {text}"))
}
