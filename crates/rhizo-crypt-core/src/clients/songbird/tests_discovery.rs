// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024–2026 ecoPrimals Project

//! Discovery, cache, and registry population tests for Songbird client.

use super::{DiscoveryClient, DiscoveryConfig};
use crate::clients::discovery_types::{ClientState, ServiceInfo, ServiceInfoExt};
use crate::discovery::{Capability, DiscoveryRegistry};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_without_connection() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    let result = client.discover("signing").await;
    assert!(result.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_operations() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let services = vec![
        ServiceInfo::new("test-1", "test-beardog")
            .with_endpoint("127.0.0.1:9000")
            .with_capability("signing"),
    ];

    client.cache_discovery("signing", services.clone()).await;

    let cached = client.discover("signing").await.unwrap();
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].name, "test-beardog");

    client.clear_cache().await;
    let empty = client.discover("signing").await.unwrap();
    assert!(empty.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "signing",
            vec![
                ServiceInfo::new("crypto-1", "crypto-primary")
                    .with_endpoint("127.0.0.1:9500")
                    .with_capability("signing"),
            ],
        )
        .await;

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    client.populate_registry(&registry).await.unwrap();

    assert!(registry.is_available(&Capability::Signing).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_specific_primal() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "signing",
            vec![
                ServiceInfo::new("crypto-1", "crypto-primary")
                    .with_endpoint("127.0.0.1:9500")
                    .with_capability("signing")
                    .with_capability("did-verification"),
            ],
        )
        .await;

    let signer = client.discover_signing_provider().await;
    assert!(signer.is_ok());
    assert!(signer.unwrap().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_permanent_storage_provider() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "permanent-storage",
            vec![
                ServiceInfo::new("storage-1", "permanent-storage-main")
                    .with_endpoint("127.0.0.1:9600")
                    .with_capability("permanent-storage"),
            ],
        )
        .await;

    let storage = client.discover_permanent_storage_provider().await;
    assert!(storage.is_ok());
    assert!(storage.unwrap().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_payload_storage_provider() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "payload-storage",
            vec![
                ServiceInfo::new("payload-1", "payload-storage-main")
                    .with_endpoint("127.0.0.1:9700")
                    .with_capability("payload-storage"),
            ],
        )
        .await;

    let storage = client.discover_payload_storage_provider().await;
    assert!(storage.is_ok());
    assert!(storage.unwrap().is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_empty_results() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let result = client.discover("nonexistent-capability").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry_not_connected() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    let registry = DiscoveryRegistry::new("test-service");

    let result = client.populate_registry(&registry).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Not connected"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry_with_services() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "signing",
            vec![
                ServiceInfo::new("crypto-1", "crypto-provider")
                    .with_endpoint("127.0.0.1:9500")
                    .with_capability("signing"),
            ],
        )
        .await;

    client
        .cache_discovery(
            "permanent-storage",
            vec![
                ServiceInfo::new("ledger-1", "ledger-provider")
                    .with_endpoint("127.0.0.1:9600")
                    .with_capability("permanent-storage"),
            ],
        )
        .await;

    client
        .cache_discovery(
            "payload-storage",
            vec![
                ServiceInfo::new("storage-1", "storage-provider")
                    .with_endpoint("127.0.0.1:9700")
                    .with_capability("payload-storage"),
            ],
        )
        .await;

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    let result = client.populate_registry(&registry).await;
    assert!(result.is_ok());

    assert!(registry.is_available(&Capability::Signing).await);
    assert!(registry.is_available(&Capability::PermanentCommit).await);
    assert!(registry.is_available(&Capability::PayloadStorage).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_cache_invalidation() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "signing",
            vec![
                ServiceInfo::new("test", "test")
                    .with_endpoint("127.0.0.1:9000")
                    .with_capability("signing"),
            ],
        )
        .await;

    let result = client.discover("signing").await.unwrap();
    assert_eq!(result.len(), 1);

    client.clear_cache().await;

    let result = client.discover("signing").await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_multiple_capability_discovery() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "multi-capability",
            vec![
                ServiceInfo::new("multi-1", "multi-service")
                    .with_endpoint("127.0.0.1:9000")
                    .with_capability("signing")
                    .with_capability("did-verification")
                    .with_capability("key-management"),
            ],
        )
        .await;

    let services = client.discover("multi-capability").await.unwrap();
    assert_eq!(services.len(), 1);
    assert_eq!(services[0].capabilities.len(), 3);
    assert!(services[0].has_capability("signing"));
    assert!(services[0].has_capability("did-verification"));
    assert!(services[0].has_capability("key-management"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_populate_registry_skips_invalid_endpoint() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    client
        .cache_discovery(
            "signing",
            vec![
                ServiceInfo::new("bad-1", "bad-endpoint")
                    .with_endpoint("not-a-valid-address")
                    .with_capability("signing"),
            ],
        )
        .await;

    let registry = DiscoveryRegistry::new("rhizoCrypt");
    let result = client.populate_registry(&registry).await;
    assert!(result.is_ok());
    assert!(!registry.is_available(&Capability::Signing).await);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_signing_provider_returns_none_when_empty() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let signer = client.discover_signing_provider().await.unwrap();
    assert!(signer.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_permanent_storage_provider_returns_none_when_empty() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let storage = client.discover_permanent_storage_provider().await.unwrap();
    assert!(storage.is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_discover_payload_storage_provider_returns_none_when_empty() {
    let config = DiscoveryConfig::with_address("127.0.0.1:8091");
    let client = DiscoveryClient::new(config);
    *client.state.write().await = ClientState::Connected;

    let storage = client.discover_payload_storage_provider().await.unwrap();
    assert!(storage.is_none());
}
