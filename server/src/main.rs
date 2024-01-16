use std::net::SocketAddr;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use hyper::{server::conn::http1, service::service_fn};

mod handler;
pub use handler::on_req;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 4269));
    let listener = TcpListener::bind(addr).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(on_req))
                .await
            {
                println!("Error: {}", err);
            }  
        });
    }
}
