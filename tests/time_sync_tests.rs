//! Tests for time synchronization module
//!
//! This module contains comprehensive tests for the time synchronization
//! system, including unit tests for individual components and integration
//! tests for the complete synchronization workflow.

#![cfg(test)]
#![cfg(feature = "network")]

use martos::time_sync::{
    SyncConfig, SyncError, SyncMessage, SyncMessageType, SyncPeer, TimeSyncManager,
};

/// Test basic TimeSyncManager creation and configuration
#[test]
fn test_time_sync_manager_creation() {
    let config = SyncConfig {
        node_id: 12345,
        sync_interval_ms: 1000,
        max_correction_threshold_us: 500,
        acceleration_factor: 0.1,
        deceleration_factor: 0.05,
        max_peers: 5,
        adaptive_frequency: true,
    };

    let mut manager = TimeSyncManager::new(config);

    // Test initial state
    assert!(!manager.is_sync_enabled());
    assert_eq!(manager.get_time_offset_us(), 0);
    assert_eq!(manager.get_sync_quality(), 1.0);
    assert_eq!(manager.get_peers().len(), 0);

    // Test enabling synchronization
    manager.enable_sync();
    assert!(manager.is_sync_enabled());

    // Test disabling synchronization
    manager.disable_sync();
    assert!(!manager.is_sync_enabled());
}

/// Test peer management functionality
#[test]
fn test_peer_management() {
    let config = SyncConfig::default();
    let mut manager = TimeSyncManager::new(config);

    // Add peers
    let peer1 = SyncPeer::new(1, [0x11, 0x11, 0x11, 0x11, 0x11, 0x11]);
    let peer2 = SyncPeer::new(2, [0x22, 0x22, 0x22, 0x22, 0x22, 0x22]);

    manager.add_peer(peer1.clone());
    manager.add_peer(peer2.clone());

    // Test peer retrieval
    assert_eq!(manager.get_peers().len(), 2);
    assert!(manager.get_peer(1).is_some());
    assert!(manager.get_peer(2).is_some());
    assert!(manager.get_peer(3).is_none());

    // Test peer removal
    manager.remove_peer(1);
    assert_eq!(manager.get_peers().len(), 1);
    assert!(manager.get_peer(1).is_none());
    assert!(manager.get_peer(2).is_some());
}

/// Test SyncMessage serialization and deserialization
#[test]
fn test_sync_message_serialization() {
    // Test sync request message
    let request = SyncMessage::new_sync_request(123, 456, 789012345);
    let data = request.to_bytes();
    let deserialized = SyncMessage::from_bytes(&data).unwrap();

    assert_eq!(request.msg_type as u8, deserialized.msg_type as u8);
    assert_eq!(request.source_node_id, deserialized.source_node_id);
    assert_eq!(request.target_node_id, deserialized.target_node_id);
    assert_eq!(request.timestamp_us, deserialized.timestamp_us);

    // Test sync response message
    let response = SyncMessage::new_sync_response(789, 101112, 131415161);
    let data = response.to_bytes();
    let deserialized = SyncMessage::from_bytes(&data).unwrap();

    assert_eq!(response.msg_type as u8, deserialized.msg_type as u8);
    assert_eq!(response.source_node_id, deserialized.source_node_id);
    assert_eq!(response.target_node_id, deserialized.target_node_id);
    assert_eq!(response.timestamp_us, deserialized.timestamp_us);
}

/// Test invalid message deserialization
#[test]
fn test_invalid_message_deserialization() {
    // Test with insufficient data
    let invalid_data = vec![0xFF; 5];
    assert!(SyncMessage::from_bytes(&invalid_data).is_none());

    // Test with invalid message type
    let mut invalid_data = vec![0xFF; 25]; // Valid length but invalid content
    invalid_data[0] = 0x99; // Invalid message type
    assert!(SyncMessage::from_bytes(&invalid_data).is_none());
}

/// Test SyncPeer functionality
#[test]
fn test_sync_peer() {
    let mut peer = SyncPeer::new(12345, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);

    // Test initial state
    assert_eq!(peer.node_id, 12345);
    assert_eq!(peer.mac_address, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    assert_eq!(peer.last_timestamp, 0);
    assert_eq!(peer.time_diff_us, 0);
    assert_eq!(peer.quality_score, 1.0);
    assert_eq!(peer.sync_count, 0);
    assert_eq!(peer.last_sync_time, 0);

    // Test updating peer information
    peer.last_timestamp = 1000000;
    peer.time_diff_us = 500;
    peer.quality_score = 0.8;
    peer.sync_count = 5;
    peer.last_sync_time = 2000000;

    assert_eq!(peer.last_timestamp, 1000000);
    assert_eq!(peer.time_diff_us, 500);
    assert_eq!(peer.quality_score, 0.8);
    assert_eq!(peer.sync_count, 5);
    assert_eq!(peer.last_sync_time, 2000000);
}

/// Test SyncConfig default values
#[test]
fn test_sync_config_default() {
    let config = SyncConfig::default();

    assert_eq!(config.node_id, 0);
    assert_eq!(config.sync_interval_ms, 1000);
    assert_eq!(config.max_correction_threshold_us, 1000);
    assert_eq!(config.acceleration_factor, 0.1);
    assert_eq!(config.deceleration_factor, 0.05);
    assert_eq!(config.max_peers, 10);
    assert!(config.adaptive_frequency);
}

/// Test SyncConfig cloning
#[test]
fn test_sync_config_clone() {
    let config = SyncConfig {
        node_id: 12345,
        sync_interval_ms: 2000,
        max_correction_threshold_us: 1500,
        acceleration_factor: 0.15,
        deceleration_factor: 0.08,
        max_peers: 15,
        adaptive_frequency: false,
    };

    let cloned_config = config.clone();

    assert_eq!(config.node_id, cloned_config.node_id);
    assert_eq!(config.sync_interval_ms, cloned_config.sync_interval_ms);
    assert_eq!(
        config.max_correction_threshold_us,
        cloned_config.max_correction_threshold_us
    );
    assert_eq!(
        config.acceleration_factor,
        cloned_config.acceleration_factor
    );
    assert_eq!(
        config.deceleration_factor,
        cloned_config.deceleration_factor
    );
    assert_eq!(config.max_peers, cloned_config.max_peers);
    assert_eq!(config.adaptive_frequency, cloned_config.adaptive_frequency);
}

/// Test SyncError variants
#[test]
fn test_sync_error() {
    let errors = vec![
        SyncError::InvalidMessage,
        SyncError::PeerNotFound,
        SyncError::SyncDisabled,
        SyncError::NetworkError,
        SyncError::CorrectionTooLarge,
    ];

    for error in errors {
        // Test that all error variants can be cloned and copied
        let cloned_error = error.clone();
        assert_eq!(format!("{:?}", error), format!("{:?}", cloned_error));
    }
}

/// Test message type variants
#[test]
fn test_sync_message_types() {
    let types = vec![
        SyncMessageType::SyncRequest,
        SyncMessageType::SyncResponse,
        SyncMessageType::TimeBroadcast,
    ];

    for msg_type in types {
        // Test that all message types can be cloned and copied
        let cloned_type = msg_type.clone();
        assert_eq!(format!("{:?}", msg_type), format!("{:?}", cloned_type));
    }
}

/// Test SyncMessage with payload
#[test]
fn test_sync_message_with_payload() {
    let payload = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    let message = SyncMessage {
        msg_type: SyncMessageType::TimeBroadcast,
        source_node_id: 12345,
        target_node_id: 0,
        timestamp_us: 987654321,
        sequence: 42,
        payload: payload.clone(),
    };

    let data = message.to_bytes();
    let deserialized = SyncMessage::from_bytes(&data).unwrap();

    assert_eq!(message.msg_type as u8, deserialized.msg_type as u8);
    assert_eq!(message.source_node_id, deserialized.source_node_id);
    assert_eq!(message.target_node_id, deserialized.target_node_id);
    assert_eq!(message.timestamp_us, deserialized.timestamp_us);
    assert_eq!(message.sequence, deserialized.sequence);
    assert_eq!(message.payload, deserialized.payload);
}

/// Test large message handling
#[test]
fn test_large_message() {
    let large_payload = vec![0xAA; 1000]; // 1KB payload
    let message = SyncMessage {
        msg_type: SyncMessageType::TimeBroadcast,
        source_node_id: 0x12345678,
        target_node_id: 0x87654321,
        timestamp_us: 0x123456789ABCDEF0,
        sequence: 0xDEADBEEF,
        payload: large_payload.clone(),
    };

    let data = message.to_bytes();
    let deserialized = SyncMessage::from_bytes(&data).unwrap();

    assert_eq!(message.payload.len(), deserialized.payload.len());
    assert_eq!(message.payload, deserialized.payload);
}

/// Test edge cases for message serialization
#[test]
fn test_message_edge_cases() {
    // Test with maximum values
    let max_message = SyncMessage {
        msg_type: SyncMessageType::SyncRequest,
        source_node_id: u32::MAX,
        target_node_id: u32::MAX,
        timestamp_us: u64::MAX,
        sequence: u32::MAX,
        payload: Vec::new(),
    };

    let data = max_message.to_bytes();
    let deserialized = SyncMessage::from_bytes(&data).unwrap();

    assert_eq!(max_message.source_node_id, deserialized.source_node_id);
    assert_eq!(max_message.target_node_id, deserialized.target_node_id);
    assert_eq!(max_message.timestamp_us, deserialized.timestamp_us);
    assert_eq!(max_message.sequence, deserialized.sequence);

    // Test with minimum values
    let min_message = SyncMessage {
        msg_type: SyncMessageType::SyncResponse,
        source_node_id: 0,
        target_node_id: 0,
        timestamp_us: 0,
        sequence: 0,
        payload: Vec::new(),
    };

    let data = min_message.to_bytes();
    let deserialized = SyncMessage::from_bytes(&data).unwrap();

    assert_eq!(min_message.source_node_id, deserialized.source_node_id);
    assert_eq!(min_message.target_node_id, deserialized.target_node_id);
    assert_eq!(min_message.timestamp_us, deserialized.timestamp_us);
    assert_eq!(min_message.sequence, deserialized.sequence);
}

/// Integration test for complete synchronization workflow
#[test]
fn test_synchronization_workflow() {
    let config = SyncConfig {
        node_id: 1,
        sync_interval_ms: 1000,
        max_correction_threshold_us: 1000,
        acceleration_factor: 0.1,
        deceleration_factor: 0.05,
        max_peers: 5,
        adaptive_frequency: true,
    };

    let mut manager = TimeSyncManager::new(config);

    // Add a peer
    let peer = SyncPeer::new(2, [0x22, 0x22, 0x22, 0x22, 0x22, 0x22]);
    manager.add_peer(peer);

    // Enable synchronization
    manager.enable_sync();
    assert!(manager.is_sync_enabled());

    // Simulate receiving a sync message
    let sync_message = SyncMessage::new_sync_response(2, 1, 1000000);
    manager.handle_sync_message(sync_message);

    // Check that peer information was updated
    let peer = manager.get_peer(2).unwrap();
    assert_eq!(peer.last_timestamp, 1000000);

    // Test time offset functionality
    assert_eq!(manager.get_time_offset_us(), 0); // Should still be 0 without algorithm processing
}

/// Test concurrent access simulation
#[test]
fn test_concurrent_access_simulation() {
    let config = SyncConfig::default();
    let manager = TimeSyncManager::new(config);

    // Test that atomic operations work correctly
    // (In a real multi-threaded environment, this would be more comprehensive)
    assert!(!manager.is_sync_enabled());
    assert_eq!(manager.get_time_offset_us(), 0);
    assert_eq!(manager.get_sync_quality(), 1.0);
}

/// Test configuration validation
#[test]
fn test_configuration_validation() {
    // Test valid configuration
    let valid_config = SyncConfig {
        node_id: 12345,
        sync_interval_ms: 1000,
        max_correction_threshold_us: 1000,
        acceleration_factor: 0.1,
        deceleration_factor: 0.05,
        max_peers: 10,
        adaptive_frequency: true,
    };

    let manager = TimeSyncManager::new(valid_config);
    assert_eq!(manager.get_peers().len(), 0);

    // Test edge case configuration
    let edge_config = SyncConfig {
        node_id: 0,
        sync_interval_ms: 1,
        max_correction_threshold_us: 1,
        acceleration_factor: 0.0,
        deceleration_factor: 0.0,
        max_peers: 1,
        adaptive_frequency: false,
    };

    let manager = TimeSyncManager::new(edge_config);
    assert_eq!(manager.get_peers().len(), 0);
}
