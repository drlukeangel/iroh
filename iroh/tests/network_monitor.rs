//! The endpoint's `network_monitor` option: with it off, `Endpoint::bind` enumerates no host
//! network interfaces. The count is netwatch's own count of interface enumerations
//! (`netwatch::interfaces::interface_enumerations`), never a time.
#![cfg(not(wasm_browser))]

use iroh::{
    Endpoint, RelayMode, SecretKey,
    endpoint::{PortmapperConfig, presets},
};
use netwatch::interfaces::interface_enumerations;

/// The configuration a Node RPC endpoint binds with: no relay, no portmapper, one explicit
/// IP transport, and (when `monitor` is off) no network monitor.
fn exact_builder(monitor: bool) -> iroh::endpoint::Builder {
    Endpoint::builder(presets::Minimal)
        .secret_key(SecretKey::generate())
        .relay_mode(RelayMode::Disabled)
        .portmapper_config(PortmapperConfig::Disabled)
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")
        .unwrap()
        .network_monitor(monitor)
}

/// CONTRACT: binding in the exact-address configuration with the network monitor off makes zero
/// interface enumerations, at bind and for as long as the endpoint is up, and `network_change`
/// does nothing; the same bind with the monitor on (the default) enumerates. One test, because
/// the count is the process's.
#[tokio::test]
async fn bind_with_the_network_monitor_off_enumerates_no_interfaces_and_the_default_does() {
    let before = interface_enumerations();
    let off = exact_builder(false).bind().await.unwrap();
    assert_eq!(
        interface_enumerations() - before,
        0,
        "Endpoint::bind with the network monitor off enumerated the host's interfaces"
    );
    assert!(off.bound_sockets().iter().all(|a| a.ip().is_loopback()));
    off.network_change().await;
    // Everything the bind spawned has had a chance to run once the endpoint closes.
    off.close().await;
    assert_eq!(
        interface_enumerations() - before,
        0,
        "an endpoint with the network monitor off enumerated the host's interfaces while it ran"
    );

    let on = exact_builder(true).bind().await.unwrap();
    let after = interface_enumerations();
    eprintln!("interface enumerations: monitor off {}, monitor on {}", 0, after - before);
    assert!(
        after - before >= 1,
        "the default (network monitor on) bind enumerates the host's interfaces; count {}",
        after - before
    );
    on.close().await;
}

/// CONTRACT: a disabled network monitor has no interface list to expand an unspecified bind
/// address with, so the bind is refused by name.
#[tokio::test]
async fn bind_with_the_network_monitor_off_refuses_an_unspecified_ip_transport() {
    let err = Endpoint::builder(presets::Minimal)
        .secret_key(SecretKey::generate())
        .relay_mode(RelayMode::Disabled)
        .portmapper_config(PortmapperConfig::Disabled)
        .clear_ip_transports()
        .bind_addr("0.0.0.0:0")
        .unwrap()
        .network_monitor(false)
        .bind()
        .await
        .unwrap_err();
    let shown = format!("{err:#}");
    assert!(shown.contains("unspecified address 0.0.0.0:0"), "{shown}");
}
