use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::{env, fmt::Debug};

#[tracing::instrument(level = "info", fields(error), skip_all)]
#[server(CreateContainer, "/api")]
pub async fn create_container_action() -> Result<String, ServerFnError> {
    use crate::gql;

    let service_data = gql::create_service().await?;
    let service_id = service_data.id;
    gql::connect_to_repo(&service_id).await?;
    let domain_data = gql::add_service_domain(&service_id).await?;
    Ok(domain_data.domain)
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/leptos-railway.css"/>

        // sets the document title
        <Title text="Let's spin up new service"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (domain, set_domain) = create_signal("".to_string());
    let (error_message, set_error_message) = create_signal("".to_string());
    
    let on_click = move |_| {
        spawn_local(async move {
            set_error_message.update(|message| *message = "".to_string());
            let response = create_container_action().await;
            match response {
                Ok(new_domain) => {
                    set_domain.update(|domain: &mut String| *domain = new_domain.to_string());
                },
                Err(error) => {
                    set_error_message.update(|message| *message = error.to_string());
                }
            }
            
        });
    };

    view! {
        <h1>"Spin up new container!"</h1>
        <button on:click=on_click>"Click Me"</button>
        <p>
            <a href=move || format!("https://{}", domain.get())>{domain}</a>
        </p>
        <p>{error_message}</p>
    }
}
