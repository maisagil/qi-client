pub mod configuration;
mod qitech;

use configuration::get_configuration;
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use qitech::{AskBalanceRequest, QiTechProvider};
use std::sync::OnceLock;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

static INSTANCE: OnceLock<QiTechProvider> = OnceLock::new();

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
async fn function_handler(event: Request) -> Result<Response<Body>, anyhow::Error> {
    // Extract some useful information from the request
    let who = event
        .query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or("world");
    let message = format!("Hello {who}, this is an AWS Lambda HTTP request");
    let request = AskBalanceRequest {
        document_number: "05577889944".to_string(),
    };
    let provider = INSTANCE.get().unwrap();
    match provider.ask_for_balance(request).await {
        Ok(response) => println!("{:#?}", &response),
        Err(e) => println!("{:#?}", e),
    }

    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let provider = QiTechProvider::new(configuration.qi_client);
    INSTANCE.set(provider).ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}
