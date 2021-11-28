// mod https_url_rewriter;

use std::convert::Infallible;
use hyper::{Request, Body, Response};

pub async fn the_insecure_proxy(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("hello".into()))
}

mod tests {
}
