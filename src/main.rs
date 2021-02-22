use error_chain::error_chain;
use json2prom::{PromFormat, JQ};
use reqwest;
use serde_json as json;
use std::net::SocketAddr;
use structopt::{self, StructOpt};
use tokio;
use warp::{self, http::Response, Filter};

error_chain! {}

#[derive(StructOpt)]
#[structopt(about)]
struct Opt {
    /// Like 127.0.0.1:80
    #[structopt(short, long)]
    socket: SocketAddr,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let routes = warp::path("get")
        .and(warp::header::<String>("X-Target"))
        .and(warp::header::optional::<String>("X-JQ"))
        .and_then(get);

    warp::serve(routes).run(opt.socket).await;
}

async fn get(
    x_target: String,
    x_jq: Option<String>,
) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection> {
    let mut json = match get_json(x_target).await {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(Box::new(
                Response::builder()
                    .status(424) // Failed Dependency - The request failed because it depended on another request and that request failed
                    .body(format!("{:#?}", err)),
            ));
        }
    };

    if let Some(query) = x_jq {
        json = match json.jq(&query) {
            Ok(new) => new,
            Err(err) => {
                return Ok(Box::new(
                    Response::builder()
                        .status(400) // Bad request
                        .body(format!("Error handling X-JQ:\n{}", err)),
                ));
            }
        };
    }

    Ok(Box::new(json.prom_format()))
}

async fn get_json(x_target: String) -> Result<json::Value> {
    let response = reqwest::get(&x_target)
        .await
        .chain_err(|| "Unable to forward request")?;
    let text = response
        .text()
        .await
        .chain_err(|| "Unable to retrieve response text")?;
    let json: json::Value = json::from_str(&text).chain_err(|| {
        format!(
            "Couldn't parse JSON from the following response body: \n{}",
            text
        )
    })?;
    Ok(json)
}
