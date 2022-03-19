extern crate jsonrpc_ipc_server;
use jsonrpc_ipc_server::ServerBuilder;
use jsonrpc_ipc_server::jsonrpc_core::*;

fn main() {
	let mut io = IoHandler::new();
	io.add_method("say_hello", |_params| {
		Ok(Value::String("hello".into()))
	});

	let builder = ServerBuilder::new(io);
	let server = builder.start("/tmp/json-ipc-test.ipc").expect("Couldn't open socket");
	server.wait();
}