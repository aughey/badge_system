use std::{cell::RefCell, rc::Rc};

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/web-badge.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

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
    // Creates a reactive value to update the button
    let (count, set_count) = create_signal(0);
    let on_click = move |_| set_count.update(|count| *count += 1);

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
        <Badge/>
    }
}

#[component]
fn Badge() -> impl IntoView {
    let screen_container = create_node_ref::<leptos::html::Div>();

    let display = Rc::new(RefCell::new(None));

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
                .pixel_spacing(1)
                .build();
            let mut text_display =
                WebSimulatorDisplay::new((WIDTH, HEIGHT), &output_settings, Some(&sc));

            badge_draw::draw_display(&mut text_display, INITIAL_TEXT)
                .expect("could not draw display");
            text_display.flush().expect("could not flush buffer");

            display.replace(Some(text_display));
        });
    }

    let update_display = move |text: &str| {
        //        let text = text.get();
        // strip any non-ascii characters
        let text = text.chars().filter(|c| c.is_ascii()).collect::<String>();
        if let Some(display) = display.borrow_mut().as_mut() {
            badge_draw::draw_display(display, text.as_str()).expect("could not draw text");
            display.flush().expect("could not flush buffer");
        }
    };

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

    view! {
        <div>
        <h1>"Badge"</h1>
        <div _ref=screen_container id="custom-container">
        </div>
        <textarea _ref=input_ref
        on:input=move |_| update_display(get_input().as_str())>
        {INITIAL_TEXT}
        </textarea>
        </div>
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
