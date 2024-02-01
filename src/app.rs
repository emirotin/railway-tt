use crate::error_template::{AppError, ErrorTemplate};
use graphql_client::GraphQLQuery;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::env;

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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.json",
    query_path = "src/gql/sample.graphql",
    response_derives = "Debug",
    variables_derive = "Debug"
)]
pub struct SampleQuery;

fn get_request_client() -> Result<reqwest::Client, ServerFnError> {
    let railway_token = env::var("RAILWAY_TOKEN").unwrap_or("".to_string());

    return Ok(reqwest::Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", railway_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()?);
}

async fn create_container(
    railway_project_id: String,
) -> Result<sample_query::ResponseData, ServerFnError> {
    let variables = sample_query::Variables {
        id: railway_project_id.to_string(),
    };

    let request_body = SampleQuery::build_query(variables);

    let client = get_request_client();

    let res = client
        .unwrap()
        .post("https://backboard.railway.app/graphql/v2")
        .json(&request_body)
        .send()
        .await?;
    let response_body: graphql_client::Response<sample_query::ResponseData> = res.json().await?;
    let response_data = response_body.data.expect("response data");

    return Ok(response_data);
}

#[tracing::instrument(level = "info", fields(error), skip_all)]
#[server(CreateContainer, "/api")]
pub async fn create_container_action() -> Result<String, ServerFnError> {
    let railway_project_id = env::var("RAILWAY_PROJECT_ID").unwrap_or("".to_string());

    let response_data = create_container(railway_project_id).await?;

    Ok(response_data.project.name)
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (message, set_message) = create_signal("".to_string());
    let on_click = move |_| {
        spawn_local(async move {
            let response = create_container_action().await.expect("api call failed");
            set_message.update(|message| *message = response)
        });
    };

    view! {
        <h1>"Spin up container!"</h1>
        <button on:click=on_click>"Click Me"</button>
        <p>{message}</p>
    }
}
