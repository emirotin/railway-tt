use crate::error_template::{AppError, ErrorTemplate};
use graphql_client::GraphQLQuery;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::{collections::HashMap, env};

pub fn get_request_client() -> Result<reqwest::Client, ServerFnError> {
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

pub const RAILWAY_GQL_ENDPOINT: &str = "https://backboard.railway.app/graphql/v2";

type ServiceVariables = HashMap<String, String>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.json",
    query_path = "src/gql/create_service.graphql",
    response_derives = "Debug"
)]
pub struct CreateService;

pub async fn create_container() -> Result<create_service::ResponseData, ServerFnError> {
    use nanoid::nanoid;

    let github_owner = env::var("RAILWAY_GIT_REPO_OWNER").unwrap_or("".to_string());
    let github_repo_name = env::var("RAILWAY_GIT_REPO_NAME").unwrap_or("".to_string());
    let github_branch = env::var("RAILWAY_GIT_BRANCH").unwrap_or("".to_string());
    let github_repo = format!("{}/{}", github_owner, github_repo_name);
    let next_level = 1 + env::var("LEVEL")
        .unwrap_or("0".to_string())
        .parse::<i32>()
        .unwrap_or(0);
    let service_id = nanoid!();
    let name = format!("{}_level_{}", service_id, next_level);

    let mut service_variables = HashMap::new();
    service_variables.insert("LEVEL".to_string(), next_level.to_string());
    service_variables.insert(
        "RAILWAY_TOKEN".to_string(),
        env::var("RAILWAY_TOKEN").unwrap_or("".to_string()),
    );

    let variables = create_service::Variables {
        input: create_service::ServiceCreateInput {
            project_id: env::var("RAILWAY_PROJECT_ID").unwrap_or("".to_string()),
            environment_id: Some(env::var("RAILWAY_ENVIRONMENT_ID").unwrap_or("".to_string())),
            branch: Some(github_branch),
            source: Some(create_service::ServiceSourceInput {
                repo: Some(github_repo),
                image: None,
            }),
            variables: Some(service_variables),
            name: Some(name),
        },
    };

    let request_body = CreateService::build_query(variables);

    let client = get_request_client();

    let res = client
        .unwrap()
        .post(RAILWAY_GQL_ENDPOINT)
        .json(&request_body)
        .send()
        .await?;
    let response_body: graphql_client::Response<create_service::ResponseData> = res.json().await?;
    let response_data = response_body.data.expect("response data");

    return Ok(response_data);
}

#[tracing::instrument(level = "info", fields(error), skip_all)]
#[server(CreateContainer, "/api")]
pub async fn create_container_action() -> Result<String, ServerFnError> {
    let response_data = create_container().await?;
    Ok(response_data.service_create.id)
}

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
