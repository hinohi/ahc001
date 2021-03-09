use std::env;

use anyhow::{Context, Result};
use proconio::source::once::OnceSource;
use serde::{Deserialize, Serialize};

use simulated_annealing::{parse_source, run};
use tokio::io::AsyncReadExt;

async fn next_invocation(url_base: &str) -> Result<(String, String)> {
    let url = format!("{}/2018-06-01/runtime/invocation/next", url_base);
    let invocation_resp = reqwest::get(&url).await?;
    let aws_request_id = invocation_resp
        .headers()
        .get("Lambda-Runtime-Aws-Request-Id")
        .context("no aws_request_id")?
        .to_str()?
        .to_owned();
    let data = invocation_resp.text().await?;
    Ok((aws_request_id, data))
}

async fn response(url_base: &str, aws_request_id: &str, body: String) -> Result<()> {
    let url = format!(
        "{}/2018-06-01/runtime/invocation/{}/response",
        url_base, aws_request_id
    );
    let client = reqwest::Client::new();
    client.post(&url).body(body).send().await?;
    Ok(())
}

#[derive(Deserialize)]
struct Body {
    n: u32,
    seed: u32,
    arg: String,
}

async fn calc(data: &str) -> Result<f64> {
    let body: Body = serde_json::from_str(data)?;
    let path = format!("/in2/{:03}/{:04}.txt", body.n, body.seed);
    let mut buf = String::new();
    tokio::fs::File::open(path)
        .await?
        .read_to_string(&mut buf)
        .await?;
    let source = OnceSource::new(buf.as_bytes());
    let input = parse_source(source);

    let (best_score, _) = run(input, Some(body.arg));
    Ok(best_score)
}

#[derive(Deserialize)]
struct SQSEvent {
    #[serde(rename = "Records")]
    records: Vec<SQSRecord>,
}

#[derive(Deserialize)]
struct SQSRecord {
    #[serde(rename = "messageId")]
    message_id: String,
    body: String,
}

#[derive(Serialize)]
struct Response {
    message_id: String,
    score: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let aws_lambda_runtime_api = env::var("AWS_LAMBDA_RUNTIME_API")?;
    let url_base = format!("http://{}", aws_lambda_runtime_api);
    loop {
        let (aws_request_id, data) = next_invocation(&url_base).await?;
        let event: SQSEvent = serde_json::from_str(&data)?;
        let mut r = Vec::new();
        for record in event.records {
            let score = calc(&record.body).await?;
            r.push(Response {
                message_id: record.message_id,
                score,
            });
        }
        response(&url_base, &aws_request_id, serde_json::to_string(&r)?).await?;
    }
}
