use super::setup_url;
use std::net::{IpAddr, Ipv4Addr};

#[test]
fn setup_url_uses_lan_ip_and_port() {
    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42));
    assert_eq!(setup_url(Some(ip), 8080), "http://192.168.1.42:8080");
}

#[test]
fn setup_url_is_empty_without_network() {
    assert_eq!(setup_url(None, 8080), "");
}
