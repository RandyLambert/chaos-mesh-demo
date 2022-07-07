use tokio::net::{UnixListener};
#[cfg(unix)]
use std::os::unix::io::{FromRawFd};
use hyper::server::conn::Http;
use hyper::{Body, Request, Response, StatusCode, Version};
use std::task::{Context, Poll};
use futures_util::future::{self, Either, FutureExt, TryFutureExt};
use tokio::select;
use tokio::sync::oneshot::{channel, Receiver, Sender};
use tokio::task::JoinHandle;
use anyhow::Error;
use std::{thread, time};
const HELLO: &str = "hello";

struct HelloWorld;

impl tower_service::Service<Request<Body>> for HelloWorld {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = future::Ready<Result<Response<Body>, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        let response = Response::new(HELLO.into());
        future::ok(response)
    }
}

#[derive(Debug)]
pub struct ConfigServer {
    task: Option<JoinHandle<Result<(), Error>>>,
}

impl ConfigServer {
    pub fn new() -> Self {
        Self {
            task: None,
        }
    }
    pub fn serve_interactive(&mut self) {

        self.task = Some(tokio::spawn(async move {
            let unix_listener = UnixListener::from_std(unsafe {std::os::unix::net::UnixListener::from_raw_fd(3)}).unwrap();

            loop {
                match unix_listener.accept().await {
                    Ok((mut stream, addr)) => {
                        println!("test");

                        let http = Http::new();
                        let conn = http.serve_connection(stream, HelloWorld);
                        if let Err(e) = conn.await {
                            tracing::error!("{}",e);
                            return Err(anyhow::anyhow!("{}",e));
                        }
                    }
                    Err(e) => {
                        tracing::error!("error : accept connection failed");
                        return Err(anyhow::anyhow!("{}", e));
                    }
                }
            }
        }));
    }

}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config_server = ConfigServer::new();
    config_server.serve_interactive();
    
    let time = time::Duration::from_millis(9000000000);

    thread::sleep(time);

    Ok(())

}