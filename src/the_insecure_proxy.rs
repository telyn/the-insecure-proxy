use crate::proxy_error::ProxyError;

use http::uri::{Uri, Authority};
use hyper::{Request, Body, Response, Client};
use http::header::HeaderValue;
use hyper::client::HttpConnector;
use hyper::service::Service;
use hyper_tls::HttpsConnector;

pub const DEFAULT_REWRITTEN_MIMES : &[&'static str] = &[
    "text/html",
    "image/svg",
    "application/javascript",
    "application/rss+xml",
    "application/xhtml+xml",
    "text/css",
    "text/javascript"
];

pub async fn the_insecure_proxy<'a>(req: Request<Body>) -> Result<Response<Body>, ProxyError<'a>> {
    let proxy = TheInsecureProxy {
        client: make_client(),
        rewritten_mimes: Vec::from(DEFAULT_REWRITTEN_MIMES)
    };

    println!("Proxying request for {}", req.uri());
    proxy.proxy_request(req).await.map_err(|err| {
        println!("{}", err);
        return ProxyError::new("meh");
    })


}

fn make_client() -> Client<HttpsConnector<HttpConnector>, hyper::Body> {
    let https = HttpsConnector::new();
    Client::builder().build(https)
}

trait ConnectorTraits:
    hyper::client::connect::Connect + Clone + Send + Sync + Service<Uri> {}

pub struct TheInsecureProxy<'a> {
    client: Client<HttpsConnector<HttpConnector>, Body>,
    rewritten_mimes: Vec<&'a str>
}

impl<'a> TheInsecureProxy<'a> {
    pub async fn proxy_request(mut self, req: Request<Body>) -> Result<Response<Body>, Box<dyn std::error::Error>> {
        let req = self.httpsify(req)?;

        let resp = self.client.request(req).await?;

        let (mut resp_parts, mut resp_body) = resp.into_parts();

        if let Some(location) = resp_parts.headers.get("Location") {

            let new_loc = location.to_str()
                .unwrap()
                .replace("https://", "http://");
            resp_parts.headers.insert("Location",
                                      new_loc
                                          .parse()
                                          .unwrap());
        }
        if let Some(content_type) = resp_parts.headers.get("Content-Type") {
            if self.should_rewrite(content_type.to_str().unwrap()) {
                println!("Should rewrite!");
                let (new_resp_body, len) = self.rewrite_body(resp_body).await.expect("Oh dear");
                resp_body = new_resp_body;
                resp_parts
                    .headers
                    .insert("Content-Length",
                            HeaderValue::from_str(&len.to_string()).unwrap());
            }
        }
        Ok(Response::from_parts(resp_parts, resp_body))
    }

    fn should_rewrite(&self, content_type: &str) -> bool {
        let response_mime = match content_type.find(';') {
            Some(loc) => &content_type[..loc],
            None => content_type
        };

        self.rewritten_mimes.iter().any(|mime| {
            response_mime.eq_ignore_ascii_case(mime)
        })
    }

    async fn rewrite_body(&self, resp_body: Body) -> Result<(Body, usize), Box<dyn std::error::Error>> {
        let bytes = hyper::body::to_bytes(resp_body).await?;
        let mut rewriter = crate::https_url_rewriter::url_rewriter();
        let body_string = std::str::from_utf8(bytes.as_ref())?;
        rewriter.consume_str(&body_string);
        let body_str = rewriter.move_output();
        let len = body_str.len();
        Ok( (Body::from(body_str), len) )
    }

    fn httpsify(&mut self, req: Request<Body>) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        let (mut req_parts, req_body) = req.into_parts();

        let mut parts = req_parts.uri.clone().into_parts();
        let host = req_parts.headers.get("Host").unwrap().clone();
        parts.authority = Some(Authority::from_maybe_shared(host)?);
        parts.scheme = Some(http::uri::Scheme::HTTPS);
        let uri_replacement = Uri::from_parts(parts).expect("Uri failed to re-parse :S");
        req_parts.uri = uri_replacement;

        return Ok(Request::from_parts(req_parts, req_body));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn httpsify_replaces_uri_scheme() {

        let mut req = Request::builder()
            .uri("http://example.com")
            .header("Host", "example.com")
            .body("hello".into())
            .unwrap();
        let mut proxy = TheInsecureProxy {
            client: make_client(),
            rewritten_mimes: vec![]
        };
        req = proxy.httpsify(req).unwrap();
        assert_eq!(req.uri(), "https://example.com");
    }
}
