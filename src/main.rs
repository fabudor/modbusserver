use std::env;
use std::future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use tokio_modbus::prelude::*;
use tokio_modbus::server::{self, Service};

#[derive(Debug)]
struct RegisterBlock {
    pub startaddress: u16,
    pub registers: Vec<u16>,
}

struct MbServer {
    register_blocks: Arc<Mutex<Vec<RegisterBlock>>>,
}

impl Service for MbServer {
    type Request = Request;
    type Response = Response;
    type Error = std::io::Error;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req {
            Request::ReadHoldingRegisters(addr, cnt) => {
                let mut blocks = self.register_blocks.lock().unwrap();
                let mut found = false;
                let mut index = 0;
                for block in blocks.iter() {
                    if block.startaddress == addr {
                        found = true;
                        break;
                    }
                    index += 1;
                }
                if !found {
                    // create a vector holding all registers for this block
                    blocks.push(RegisterBlock {
                        startaddress: addr,
                        registers: vec![0; cnt as usize],
                    });
                }

                // increment all registers in this block
                for i in 0..blocks[index].registers.len() {
                    blocks[index].registers[i] += 1;
                }
                // println!("ReadHoldingRegisters: {:?}", &blocks[index]); //unimplemented!(),
                future::ready(Ok(Response::ReadHoldingRegisters(
                    blocks[index].registers.clone(),
                )))
            }
            _ => unimplemented!(),
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("Argument missing");
    }
    let socket_addr = SocketAddr::new(
        // IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        args[1].parse::<u16>().unwrap(),
    );

    println!("Starting modbus simulation server on port {}", args[1]);
    let _server = server::tcp::Server::new(socket_addr).serve(move || {
        Ok(MbServer {
            register_blocks: Arc::new(Mutex::new(vec![])),
        })
    });
    Ok(())
}
