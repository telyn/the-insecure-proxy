use crate::proxy_error::ProxyError;

use bytes::Bytes;
use hyper::http::{HeaderValue};
use hyper::http::uri::{Authority, Uri};
use hyper::client::HttpConnector;
use hyper::service::Service;
use hyper::{Body, Client, Request, Response};
use hyper_tls::HttpsConnector;

pub const DEFAULT_REWRITTEN_MIMES: &[&str] = &[
    "text/html",
    "image/svg",
    "application/javascript",
    "application/rss+xml",
    "application/xhtml+xml",
    "text/css",
    "text/javascript",
];

pub async fn the_insecure_proxy<'a>(req: Request<Body>) -> Result<Response<Body>, ProxyError<'a>> {
    let proxy = TheInsecureProxy {
        client: make_client(),
        rewritten_mimes: Vec::from(DEFAULT_REWRITTEN_MIMES),
    };

    println!("{} {}", req.method(), req.uri());
    let res = proxy.proxy_request(req).await.map_err(|err| {
        println!("  ERR {}", err);
        ProxyError::new("meh")
    });
    res
}

fn make_client() -> Client<HttpsConnector<HttpConnector>, hyper::Body> {
    let https = HttpsConnector::new();
    Client::builder().build(https)
}

trait ConnectorTraits: hyper::client::connect::Connect + Clone + Send + Sync + Service<Uri> {}

pub struct TheInsecureProxy<'a> {
    client: Client<HttpsConnector<HttpConnector>, Body>,
    rewritten_mimes: Vec<&'a str>,
}

impl<'a> TheInsecureProxy<'a> {
    pub async fn proxy_request(
        mut self,
        req: Request<Body>,
    ) -> Result<Response<Body>, Box<dyn std::error::Error>> {
        let req = self.httpsify(req)?;

        self.log_headers('>', req.headers());
        let req_uri = req.uri().clone();
        let req_method = req.method().clone();
        let resp = self.client.request(req).await?;

        let (mut resp_parts, mut resp_body) = resp.into_parts();

        if let Some(location) = resp_parts.headers.get("Location") {
            let new_loc = location.to_str().unwrap().replace("https://", "http://");
            resp_parts
                .headers
                .insert("Location", new_loc.parse().unwrap());
        }
        if let Some(content_type) = resp_parts.headers.get("Content-Type") {
            let content_type = content_type.to_str().unwrap();
            println!("= Received content type is {}", content_type);
            if self.should_rewrite(content_type) {
                println!("= Should rewrite!");
                let new_body_str = self.rewrite_body(resp_body).await.expect("Oh dear");

                resp_parts.headers.insert(
                    "Content-Length",
                    HeaderValue::from_str(&new_body_str.len().to_string()).unwrap(),
                );
                self.log_headers('<', &resp_parts.headers);
                // println!("< {}", new_body_str.replace("\n", "\n< "));
                resp_body = Body::from(new_body_str);
            } else {
                println!("= not rewriting");
            }
        }
        println!(
            "= Completed {} response to {} {}",
            resp_parts.status, req_method, req_uri
        );
        Ok(Response::from_parts(resp_parts, resp_body))
    }

    fn should_rewrite(&self, content_type: &str) -> bool {
        let response_mime = match content_type.find(';') {
            Some(loc) => &content_type[..loc],
            None => content_type,
        };

        self.rewritten_mimes
            .iter()
            .any(|mime| response_mime.eq_ignore_ascii_case(mime))
    }

    fn log_headers(&self, prefix: char, headers: &hyper::header::HeaderMap) {
        for (key, value) in headers.iter() {
            println!("{} {:?}: {:?}", prefix, key, value);
        }
        println!("< ");
    }

    async fn rewrite_body(&self, resp_body: Body) -> Result<Bytes, Box<dyn std::error::Error>> {
        let mut bytes = hyper::body::to_bytes(resp_body).await?;
        let mut rewriter = crate::https_url_rewriter::url_rewriter();
        rewriter.consume_str(&mut bytes);
        Ok(rewriter.move_output())
    }

    fn httpsify(
        &mut self,
        req: Request<Body>,
    ) -> Result<Request<Body>, Box<dyn std::error::Error>> {
        let (mut req_parts, req_body) = req.into_parts();

        let mut parts = req_parts.uri.clone().into_parts();
        let host = req_parts.headers.get("Host").unwrap().clone();
        parts.authority = Some(Authority::from_maybe_shared(host)?);
        parts.scheme = Some(hyper::http::uri::Scheme::HTTPS);
        let uri_replacement = Uri::from_parts(parts).expect("Uri failed to re-parse :S");
        req_parts.uri = uri_replacement;

        Ok(Request::from_parts(req_parts, req_body))
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
            rewritten_mimes: vec![],
        };
        req = proxy.httpsify(req).unwrap();
        assert_eq!(req.uri(), "https://example.com");
    }
}
