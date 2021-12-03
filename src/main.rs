mod proxy_error;
mod the_insecure_proxy;
mod https_url_rewriter;

use the_insecure_proxy::{the_insecure_proxy};

use std::convert::Infallible;
use std::{env,net};
use hyper::{Server};
use hyper::service::{make_service_fn, service_fn};

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn listen_addr() -> Result<net::SocketAddr, Box<dyn std::error::Error>> {
    let default_address = "0.0.0.0";
    let default_port = 3080;

    let address = match env::var("BIND_ADDRESS") {
        Ok(addr) => Ok(addr),
        Err(env::VarError::NotPresent) => Ok(String::from(default_address)),
        Err(_) => Err("BIND_ADDRESS was not valid")
    }?;

    let port: u16 = match env::var("PORT") {
        Ok(port) => str::parse(&port).or(Err("PORT was not a valid integer")),
        Err(env::VarError::NotPresent) => Ok(default_port),
        Err(env::VarError::NotUnicode(_)) => Err("PORT was not valid")
    }?;


    let ip_addr : net::IpAddr = address.parse()?;

    Ok(net::SocketAddr::from((ip_addr, port)))
}

#[tokio::main]
async fn main() {
    let addr = listen_addr();
    match addr {
        Ok(addr) => {
            println!("Booting server on {}", addr);
        },
        Err(x) => {
            println!("{}", x);
            std::process::exit(1);
        }
    }


    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(the_insecure_proxy))
    });

    let server = Server::bind(&addr.unwrap()).serve(make_svc)
                                    .with_graceful_shutdown(shutdown_signal());

    println!("Now listening!");

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
