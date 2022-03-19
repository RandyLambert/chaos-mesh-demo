use tokio::net::UnixStream;
use std::os::unix::net::UnixStream as StdUnixStream;
#[cfg(unix)]
use std::os::unix::io::{FromRawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{FromRawFd};
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
    rx: Option<Receiver<()>>,
    sender: Option<Sender<()>>,
}

impl ConfigServer {
    pub fn new() -> Self {
        let (sender, rx) = channel();
        Self {
            task: None,
            rx: Some(rx),
            sender: Some(sender),
        }
    }
    pub fn serve_interactive(&mut self) {
        let rx = self.rx.take().unwrap();
        self.task = Some(tokio::spawn(async move {
            select! {
                _ = rx => {
                    tracing::trace!("catch signal in config server.");
                    return Ok(());
                },
                _ = async {
                    loop {
                        let std_unix_stream = unsafe { StdUnixStream::from_raw_fd(3) };
                        let unix_stream = UnixStream::from_std(std_unix_stream).unwrap();
                        let http = Http::new();
                        let conn = http.serve_connection(unix_stream, HelloWorld);
                        if let Err(e) = conn.await {
                            tracing::error!("{}",e);
                            return Err(anyhow::anyhow!("{}",e));
                        }
                    }
                    #[allow(unreachable_code)]
                    Ok::<_, anyhow::Error>(())
                } => {}
            };
            Ok(())
        }));
    }

}

// async fn test() {
//     let std_unix_stream = unsafe { StdUnixStream::from_raw_fd(3) };
//     let unix_stream = UnixStream::from_std(std_unix_stream).unwrap();
//     let http = Http::new();
//     let conn = http.serve_connection(unix_stream, HelloWorld);
// }
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config_server = ConfigServer::new();
    config_server.serve_interactive();
    
    let time = time::Duration::from_millis(9000000000);

    thread::sleep(time);

    Ok(())

}



