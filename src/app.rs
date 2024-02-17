use crate::error_template::{AppError, ErrorTemplate};
use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use std::{collections::HashMap, env, fmt::Debug};

fn get_request_client() -> Result<reqwest::Client, reqwest::Error> {
    let railway_token = env::var("RAILWAY_TOKEN").unwrap();


    reqwest::Client::builder()
        .user_agent("graphql-rust/0.10.0")
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", railway_token))
                    .unwrap(),
            ))
            .collect(),
        )
        .build()
}

const RAILWAY_GQL_ENDPOINT: &str = "https://backboard.railway.app/graphql/v2";

type ServiceVariables = HashMap<String, String>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.json",
    query_path = "src/gql/create_service.graphql",
    response_derives = "Debug"
)]
pub struct CreateService;

#[derive(Debug, thiserror::Error)]
pub enum GraphQLError {
    #[error("ReqwestError {0}")]
    ReqwestError(reqwest::Error),
    #[error("ServerResponseError {0}")]
    ServerResponseError(String),
}

impl From<reqwest::Error> for GraphQLError {
    fn from(err: reqwest::Error) -> GraphQLError {
        GraphQLError::ReqwestError(err)
    }
}

pub async fn create_service() -> Result<create_service::CreateServiceServiceCreate, GraphQLError>
{
    use nanoid::nanoid;

    let next_level: i32 = 1 + env::var("LEVEL")
        .unwrap_or("0".to_string())
        .parse()
        .unwrap_or(0);
    let service_uid = nanoid!();
    let service_name = format!("{}_level_{}", service_uid, next_level);

    let mut env_variables = HashMap::new();
    env_variables.insert("LEVEL".to_string(), next_level.to_string());
    env_variables.insert(
        "RAILWAY_TOKEN".to_string(),
        env::var("RAILWAY_TOKEN").unwrap_or("".to_string()),
    );

    let variables = create_service::Variables {
        input: create_service::ServiceCreateInput {
            project_id: env::var("RAILWAY_PROJECT_ID").unwrap_or("".to_string()),
            environment_id: Some(env::var("RAILWAY_ENVIRONMENT_ID").unwrap_or("".to_string())),
            branch: None,
            source: None,
            variables: Some(env_variables),
            name: Some(service_name),
        },
    };

    let client = get_request_client()?;

    let response =
        post_graphql::<CreateService, &str>(&client, RAILWAY_GQL_ENDPOINT, variables).await?;

    if let Some(errors) = response.errors {
        let error_message = errors.iter().map(|err| {
            format!("{}", err)
        }).collect::<String>();
        print!("1: {error_message}");
        return Err(GraphQLError::ServerResponseError(error_message));
    }

    Ok(response.data.unwrap().service_create)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.json",
    query_path = "src/gql/create_service_domain.graphql",
    response_derives = "Debug"
)]
pub struct CreateServiceDomain;

pub async fn add_service_domain(
    service_id: &str,
) -> Result<create_service_domain::CreateServiceDomainServiceDomainCreate, GraphQLError> {
    let variables = create_service_domain::Variables {
        input: create_service_domain::ServiceDomainCreateInput {
            service_id: service_id.to_string(),
            environment_id: env::var("RAILWAY_ENVIRONMENT_ID").unwrap_or("".to_string()),
        },
    };

    let client = get_request_client()?;

    let response =
        post_graphql::<CreateServiceDomain, &str>(&client, RAILWAY_GQL_ENDPOINT, variables).await?;

    if let Some(errors) = response.errors {
        let error_message = errors.iter().map(|err| {
            format!("{}", err)
        }).collect::<String>();
        print!("2: {error_message}");
        return Err(GraphQLError::ServerResponseError(error_message));
    }

    Ok(response.data.unwrap().service_domain_create)
}

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.json",
    query_path = "src/gql/connect_service_to_repo.graphql",
    response_derives = "Debug"
)]
pub struct ConnectServiceToRepo;

pub async fn connect_to_repo(
    service_id: &str,
) -> Result<connect_service_to_repo::ConnectServiceToRepoServiceConnect, GraphQLError> {
    let github_owner = env::var("RAILWAY_GIT_REPO_OWNER").unwrap_or("".to_string());
    let github_repo_name = env::var("RAILWAY_GIT_REPO_NAME").unwrap_or("".to_string());
    let github_branch = env::var("RAILWAY_GIT_BRANCH").unwrap_or("".to_string());
    let github_repo = format!("{}/{}", github_owner, github_repo_name);

    let variables = connect_service_to_repo::Variables {
        id: service_id.to_string(),
        input: connect_service_to_repo::ServiceConnectInput {
            repo: Some(github_repo),
            branch: Some(github_branch),
            image: None
        },
    };

    let client = get_request_client()?;

    let response =
        post_graphql::<ConnectServiceToRepo, &str>(&client, RAILWAY_GQL_ENDPOINT, variables).await?;

    if let Some(errors) = response.errors {
        let error_message = errors.iter().map(|err| {
            format!("{}", err)
        }).collect::<String>();
        print!("2: {error_message}");
        return Err(GraphQLError::ServerResponseError(error_message));
    }

    Ok(response.data.unwrap().service_connect)
}

#[tracing::instrument(level = "info", fields(error), skip_all)]
#[server(CreateContainer, "/api")]
pub async fn create_container_action() -> Result<String, ServerFnError> {
    let service_data = create_service().await?;
    let service_id = service_data.id;
    connect_to_repo(&service_id).await?;
    let domain_data = add_service_domain(&service_id).await?;
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
                    set_domain.update(|domain: &mut String| *domain = new_domain);
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
            <a href=format!("https://{}", domain())>{domain}</a>
        </p>
        <p>{error_message}</p>
    }
}
