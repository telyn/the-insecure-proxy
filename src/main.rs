mod https_url_rewriter;
mod proxy_error;
mod the_insecure_proxy;

use the_insecure_proxy::the_insecure_proxy;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::{env, net};
use tokio::net::TcpListener;


fn listen_addr() -> Result<net::SocketAddr, Box<dyn std::error::Error>> {
    let default_address = "0.0.0.0";
    let default_port = 3080;

    let address = match env::var("BIND_ADDRESS") {
        Ok(addr) => Ok(addr),
        Err(env::VarError::NotPresent) => Ok(String::from(default_address)),
        Err(_) => Err("BIND_ADDRESS was not valid"),
    }?;

    let port: u16 = match env::var("PORT") {
        Ok(port) => str::parse(&port).or(Err("PORT was not a valid integer")),
        Err(env::VarError::NotPresent) => Ok(default_port),
        Err(env::VarError::NotUnicode(_)) => Err("PORT was not valid"),
    }?;

    let ip_addr: net::IpAddr = address.parse()?;

    Ok(net::SocketAddr::from((ip_addr, port)))
}

async fn accept_connection(stream: tokio::net::TcpStream) {
    let io = TokioIo::new(stream);

    tokio::task::spawn(async move {
        if let Err(err) = http1::Builder::new()
            .serve_connection(io, service_fn(the_insecure_proxy))
            .await
        {
            eprintln!("Error serving connection: {:?}", err);
        }
    });
}

async fn graceful_shutdown() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
    println!("\nReceived SIGINT, shutting down gracefully...");
}

#[tokio::main]
async fn main() {
    let addr = listen_addr();
    let addr = match addr {
        Ok(addr) => {
            println!("Booting server on {}", addr);
            addr
        }
        Err(x) => {
            println!("{}", x);
            std::process::exit(1);
        }
    };

    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Now listening!");

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => {
                        accept_connection(stream).await;
                    }
                    Err(e) => {
                        eprintln!("accept error: {}", e);
                    }
                }
            }
            _ = graceful_shutdown() => {
                break;
            }
        }
    }

    println!("Server stopped.");
}
