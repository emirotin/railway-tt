use graphql_client::{reqwest::post_graphql, GraphQLQuery};
use std::{collections::HashMap, env, fmt::Debug};
use thiserror::Error;

#[derive(Debug, Error)]
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

const RAILWAY_GQL_ENDPOINT: &str = "https://backboard.railway.app/graphql/v2";

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

type ServiceVariables = HashMap<String, String>;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/gql/schema.json",
    query_path = "src/gql/create_service.graphql",
    response_derives = "Debug"
)]
struct CreateService;

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
struct CreateServiceDomain;

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
struct ConnectServiceToRepo;

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