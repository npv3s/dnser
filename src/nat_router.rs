use std::collections::HashMap;
use std::error::Error;
use std::net::Ipv4Addr;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;

use ipnet::{IpAdd, Ipv4Net};
use tracing::warn;
use tracing::{error, info};

fn nft(command: &str) -> Result<(), Box<dyn Error>> {
    info!("nft {command}");

    if cfg!(windows) {
        panic!("this program doesn't work on Windows")
    }

    match Command::new("nft").arg(command).output() {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("failed to execute nft {}: {}", command, err);
            Err(Box::new(err))
        }
    }
}

#[derive(Debug)]
pub struct NatRouter {
    subnet: Ipv4Net,
    routes: Arc<Mutex<HashMap<Ipv4Addr, Ipv4Addr>>>,
    last_ip: Arc<Mutex<Ipv4Addr>>,
}

impl NatRouter {
    pub fn new(subnet: Ipv4Net) -> Self {
        if let Err(err) = nft("add table ip nat") {
            warn!("failed to add nat table: {}", err);
        }
        if let Err(err) = nft(
            "add chain ip nat dnsmap { type nat hook prerouting priority -100; policy accept; }",
        ) {
            warn!("failed to add dnsmap chain: {}", err);
        }
        if let Err(err) = nft("flush chain ip nat dnsmap") {
            warn!("failed to flush dnsmap chain: {}", err);
        }

        NatRouter {
            subnet,
            routes: Arc::new(Mutex::new(HashMap::new())),
            last_ip: Arc::new(Mutex::new(subnet.addr())),
        }
    }

    pub fn route(&self, addr: Ipv4Addr) -> Ipv4Addr {
        let routes_r = Arc::clone(&self.routes);
        let mut routes = routes_r.lock().unwrap();

        match routes.get(&addr) {
            Some(route_addr) => route_addr.to_owned(),
            None => {
                let previous_ip = Arc::clone(&self.last_ip);
                let mut previous_ip = previous_ip.lock().unwrap();

                if *previous_ip == self.subnet.broadcast() {
                    *previous_ip = self.subnet.addr();
                }

                let new_ip = (*previous_ip).saturating_add(1);
                if let Err(err) = nft(format!(
                    "add rule ip nat dnsmap ip daddr {} counter dnat to {}",
                    new_ip, addr
                )
                .as_str())
                {
                    error!(
                        "failed to add routing rule from {} to {}: {}",
                        new_ip, addr, err
                    )
                }
                *previous_ip = new_ip;
                routes.insert(addr, new_ip);
                new_ip
            }
        }
    }
}

impl Drop for NatRouter {
    fn drop(&mut self) {
        let _ = nft("delete chain ip nat dnsmap");
    }
}
