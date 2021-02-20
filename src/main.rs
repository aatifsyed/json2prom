use error_chain::error_chain;
use reqwest;
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
        .and_then(get);

    warp::serve(routes).run(opt.socket).await;
}

async fn get(x_target: String) -> std::result::Result<Box<dyn warp::Reply>, warp::Rejection> {
    let json = match get_json(x_target).await {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(Box::new(
                Response::builder()
                    .status(424) // Failed Dependency - The request failed because it depended on another request and that request failed
                    .body(format!("{:#?}", err)),
            ));
        }
    };

    let prom = json
        .entries()
        .map(|(key, value)| format!("{} \"{}\"", key, value.as_str().unwrap_or_else(|| "")))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(Box::new(prom))
}

async fn get_json(x_target: String) -> Result<json::JsonValue> {
    let response = reqwest::get(&x_target)
        .await
        .chain_err(|| "Unable to forward request")?;
    let text = response
        .text()
        .await
        .chain_err(|| "Unable to retrieve response text")?;
    Ok(json::parse(&text)
        .chain_err(|| format!("Couldn't parse response as JSON. Got...\n{}", text))?)
}
