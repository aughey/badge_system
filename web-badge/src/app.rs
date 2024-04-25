use std::{cell::RefCell, rc::Rc};

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

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
    view! {
        <Badge/>
    }
}

#[component]
fn Flash(rate: ReadSignal<u64>) -> impl IntoView {
    let (onoff, set_onoff) = create_signal(true);

    create_effect(move |_| {
        gloo_timers::callback::Interval::new(rate().try_into().unwrap(), move || {
            set_onoff.update(|v| *v = !*v);
        })
    });

    view! {
        <div inner_html={move || if onoff() { "o" } else { "&nbsp;" }}></div>
    }
}

#[component]
fn Screen(text: ReadSignal<String>) -> impl IntoView {
    let screen_container = create_node_ref::<leptos::html::Div>();
    let display = Rc::new(RefCell::new(None));

    // Effect to construct the display
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

            badge_draw::draw_display(&mut text_display, "INIT").expect("could not draw display");
            text_display.flush().expect("could not flush buffer");

            display.replace(Some(text_display));
        });
    }

    create_effect(move |_| {
        let text = text.get();
        let text = format_text_for_badge(text);
        if let Some(text_display) = display.borrow_mut().as_mut() {
            badge_draw::draw_display(text_display, &text).expect("could not draw display");
            text_display.flush().expect("could not flush buffer");
        }
    });

    view! {
        <div _ref=screen_container id="custom-container" class="badge">
        </div>
    }
}

#[component]
fn Badge() -> impl IntoView {
    let options = [50, 100, 250, 500, 1000];
    let (value, set_value) = create_signal(1000u64);
    let (messages, set_messages) = create_signal(Vec::new());
    let (badge_text, set_badge_text) = create_signal("Enter Text Here".to_string());

    // Text areas are finicky so we need to use a ref to get the value
    // and this get_input helper function to extract the value.
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

    // Fn to send the text to the badge
    let send_text_to_badge = move || {
        let text = badge_text();
        let freq = value();
        spawn_local(async move {
            update_text(text.clone()).await.unwrap();
            update_frequency(freq).await.unwrap();
            set_messages.update(|m| {
                m.push(format!("Sent text to the server: {}", text));
                m.push(format!("Sent update rate to the server: {}", freq));
            });
            ()
        });
    };

    // Convert options into the view
    let options = options
        .into_iter()
        .map(|v| {
            view! {
                <option selected=move|| if v == value() { "selected" } else { "" }>
                    {v}
                </option>
            }
        })
        .collect_view();

    view! {
        <div>
        <h1>"Badge"</h1>
        <Screen text=badge_text/>
        <Flash rate=value/>
        <textarea _ref=input_ref
        on:input=move |_| {
            set_badge_text(get_input().to_string());
        }>
        {badge_text.get_untracked()}
        </textarea>
        <div>LED Flash Rate (ms)
         <select on:change=move |ev| {
        let new_value = event_target_value(&ev).parse().unwrap();
        set_value(new_value);
    }>
        {options}
    </select>
    </div>
        <button on:click=move |_| send_text_to_badge()>Send this state to Badge</button>
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
    // sort of input validation here so that all downstream actions are safe
    let text = format_text_for_badge(text);
    // truncate text
    crate::badge_channels::set_text(&text);
    Ok(format!("Updated text to {text}"))
}
