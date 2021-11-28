mod the_insecure_proxy;

use the_insecure_proxy::{the_insecure_proxy};

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Server};
use hyper::service::{make_service_fn, service_fn};

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3080));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(the_insecure_proxy))
    });

    let server = Server::bind(&addr).serve(make_svc)
                                    .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
