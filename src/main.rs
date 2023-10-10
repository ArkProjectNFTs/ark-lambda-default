use ark_dynamodb::{
    init_aws_dynamo_client,
    // Please adjust to the required providers.
    providers::*,
    Client as DynamoClient,
};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use lambda_http_common::{self as common, HttpParamSource};

/// A struct to bundle all init required by the lambda.
/// More providers can be used here if you need them. But having
/// only one `Ctx` struct is handy to avoid changing the inputs.
struct Ctx<P> {
    client: DynamoClient,
    provider: P,
}

/// TODO: adjust the `P` type based on your provider (Contract, Token, Event, Block, ...).
async fn function_handler<P: ArkContractProvider<Client = DynamoClient>>(
    ctx: &Ctx<P>,
    event: Request,
) -> Result<Response<Body>, Error> {
    // You can extract query or path params using the require_* methods from common.
    // It will fail if the parameter is not found.
    let contract_address = match common::require_hex_param(&event, "contract_address", HttpParamSource::Path)
    {
        Ok(a) => a,
        Err(e) => return e.try_into(),
    };

    // If you want optional parameters, you can directly use the `Request` functions:
    // https://docs.rs/lambda_http/0.8.1/lambda_http/type.Request.html

    // Then, call the provider method you want to use.
    if let Some(data) = ctx.provider.get_contract(&ctx.client, &address).await? {
        common::ok_body_rsp(&data)
    } else {
        // The common package contains some usefull functions to return a `Response` with
        // pre-defined status code.
        common::not_found_rsp()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let table_name = std::env::var("ARK_TABLE_NAME").expect("ARK_TABLE_NAME must be set");

    let ctx = Ctx {
        client: init_aws_dynamo_client().await,
        // TODO: adjust the provider here based on Block, Event, Token or Contract.
        // If you need several providers, don't hesitate to add them to the `Ctx` struct.
        provider: DynamoDbContractProvider::new(&table_name),
    };

    run(service_fn(|event: Request| async {
        function_handler(&ctx, event).await
    }))
    .await
}
