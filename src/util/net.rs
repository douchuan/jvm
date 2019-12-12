#![allow(unused)]

use nix::{
    ifaddrs::{self, InterfaceAddress},
    sys::socket::{AddressFamily, InetAddr, SockAddr},
};

pub fn get_ip() -> String {
    let ifs = ifaddrs::getifaddrs().unwrap();
    for it in ifs {
        if it.interface_name != "lo" {
            match it.address {
                Some(address) if address.family() == AddressFamily::Inet => {
                    if let SockAddr::Inet(ref inet) = address {
                        return format!("{}", inet.ip());
                    }
                }
                _ => {}
            }
        }
    }

    "".to_string()
}
