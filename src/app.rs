use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos-railway.css"/>

        // sets the document title
        <Title text="Let's spin up new service"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[tracing::instrument(level = "info", fields(error), skip_all)]
#[server(CreateContainer, "/api")]
pub async fn create_container() -> Result<String, ServerFnError> {
    // use dotenvy_macro::dotenv;
    // println!("{}", dotenv!("RAILWAY_TOKEN"));

    Ok("ok".to_string())
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (message, set_message) = create_signal("".to_string());
    let on_click = move |_| {
        // let c = count();
        spawn_local(async move {
            let response = create_container().await.expect("api call failed");
            set_message.update(|message| *message = response)
        });
    };

    view! {
        <h1>"Spin up container!"</h1>
        <button on:click=on_click>"Click Me"</button>
        <p>{message}</p>
    }
}
