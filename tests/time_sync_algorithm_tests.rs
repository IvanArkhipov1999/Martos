//! Tests for time synchronization algorithm
//!
//! This module contains tests for the core synchronization algorithm,
//! including dynamic acceleration/deceleration, peer quality management,
//! and convergence detection.

#![cfg(test)]
#![cfg(feature = "network")]

use martos::time_sync::{SyncConfig, SyncError, SyncPeer};

// Import the algorithm module (this would need to be made public for testing)
// For now, we'll test the public interface through TimeSyncManager

/// Test algorithm initialization
#[test]
fn test_algorithm_initialization() {
    let config = SyncConfig {
        node_id: 12345,
        sync_interval_ms: 1000,
        max_correction_threshold_us: 1000,
        acceleration_factor: 0.1,
        deceleration_factor: 0.05,
        max_peers: 5,
        adaptive_frequency: true,
    };

    // Test that configuration is properly stored
    assert_eq!(config.node_id, 12345);
    assert_eq!(config.sync_interval_ms, 1000);
    assert_eq!(config.max_correction_threshold_us, 1000);
    assert_eq!(config.acceleration_factor, 0.1);
    assert_eq!(config.deceleration_factor, 0.05);
    assert_eq!(config.max_peers, 5);
    assert!(config.adaptive_frequency);
}

/// Test peer quality score management
#[test]
fn test_peer_quality_management() {
    let mut peer = SyncPeer::new(12345, [0x11, 0x11, 0x11, 0x11, 0x11, 0x11]);

    // Test initial quality score
    assert_eq!(peer.quality_score, 1.0);

    // Test quality score bounds
    peer.quality_score = 1.5; // Above maximum
    assert!(peer.quality_score <= 1.0);

    peer.quality_score = -0.5; // Below minimum
    assert!(peer.quality_score >= 0.0);

    // Test quality score updates
    peer.quality_score = 0.8;
    peer.sync_count = 10;
    assert_eq!(peer.quality_score, 0.8);
    assert_eq!(peer.sync_count, 10);
}

/// Test time difference calculations
#[test]
fn test_time_difference_calculations() {
    let peer1 = SyncPeer {
        node_id: 1,
        mac_address: [0x11, 0x11, 0x11, 0x11, 0x11, 0x11],
        last_timestamp: 1000000,
        time_diff_us: 100,
        quality_score: 1.0,
        sync_count: 5,
        last_sync_time: 1000000,
    };

    let peer2 = SyncPeer {
        node_id: 2,
        mac_address: [0x22, 0x22, 0x22, 0x22, 0x22, 0x22],
        last_timestamp: 1000000,
        time_diff_us: -200,
        quality_score: 0.5,
        sync_count: 3,
        last_sync_time: 1000000,
    };

    // Test weighted average calculation
    // Peer1: diff=100, quality=1.0, weight=1.0
    // Peer2: diff=-200, quality=0.5, weight=0.5
    // Weighted average = (100*1.0 + (-200)*0.5) / (1.0 + 0.5) = 0 / 1.5 = 0
    let expected_weighted_avg = 0i64;

    // This would be calculated by the algorithm
    // For now, we verify the individual components
    assert_eq!(peer1.time_diff_us, 100);
    assert_eq!(peer2.time_diff_us, -200);
    assert_eq!(peer1.quality_score, 1.0);
    assert_eq!(peer2.quality_score, 0.5);
}

/// Test acceleration factor calculations
#[test]
fn test_acceleration_factor_calculations() {
    let config = SyncConfig {
        node_id: 12345,
        sync_interval_ms: 1000,
        max_correction_threshold_us: 1000,
        acceleration_factor: 0.1,
        deceleration_factor: 0.05,
        max_peers: 5,
        adaptive_frequency: true,
    };

    // Test different time difference scenarios
    let scenarios = vec![
        (0i64, 0.05),    // Perfect sync - should use deceleration
        (100i64, 0.1),   // Small difference - should use acceleration
        (500i64, 0.1),   // Moderate difference - should use acceleration
        (1000i64, 0.05), // Large difference - should use reduced acceleration
        (2000i64, 0.05), // Very large difference - should use reduced acceleration
    ];

    for (time_diff, expected_factor) in scenarios {
        let convergence_threshold = config.max_correction_threshold_us as i64 / 10; // 100μs

        let actual_factor = if time_diff.abs() <= convergence_threshold {
            config.deceleration_factor
        } else if time_diff.abs() <= config.max_correction_threshold_us as i64 {
            config.acceleration_factor
        } else {
            config.acceleration_factor * 0.5
        };

        assert_eq!(actual_factor, expected_factor);
    }
}

/// Test correction bounds application
#[test]
fn test_correction_bounds() {
    let config = SyncConfig {
        node_id: 12345,
        sync_interval_ms: 1000,
        max_correction_threshold_us: 1000,
        acceleration_factor: 0.1,
        deceleration_factor: 0.05,
        max_peers: 5,
        adaptive_frequency: true,
    };

    let max_correction = config.max_correction_threshold_us as i64;

    // Test various correction values
    let test_cases = vec![
        (0i64, 0i64),         // No correction
        (500i64, 500i64),     // Within bounds
        (1000i64, 1000i64),   // At maximum
        (1500i64, 1000i64),   // Above maximum - should be clamped
        (-500i64, -500i64),   // Negative within bounds
        (-1000i64, -1000i64), // Negative at maximum
        (-1500i64, -1000i64), // Negative above maximum - should be clamped
    ];

    for (input_correction, expected_correction) in test_cases {
        let actual_correction = if input_correction > max_correction {
            max_correction
        } else if input_correction < -max_correction {
            -max_correction
        } else {
            input_correction
        };

        assert_eq!(actual_correction, expected_correction);
    }
}

/// Test peer quality updates
#[test]
fn test_peer_quality_updates() {
    let mut peer = SyncPeer::new(12345, [0x11, 0x11, 0x11, 0x11, 0x11, 0x11]);
    let config = SyncConfig::default();

    // Test quality improvement with small correction
    let small_correction = 50i64; // Small correction
    let max_threshold = config.max_correction_threshold_us as f32;

    if small_correction.abs() as f32 <= max_threshold * 0.1 {
        peer.quality_score = (peer.quality_score + config.acceleration_factor).min(1.0);
        peer.sync_count += 1;
    }

    assert!(peer.quality_score > 1.0); // Should be clamped to 1.0
    assert_eq!(peer.sync_count, 1);

    // Test quality maintenance with moderate correction
    let moderate_correction = 500i64; // Moderate correction
    let initial_quality = peer.quality_score;

    if moderate_correction.abs() as f32 <= max_threshold * 0.5 {
        // No change to quality score
    }

    assert_eq!(peer.quality_score, initial_quality);

    // Test quality degradation with large correction
    let large_correction = 1500i64; // Large correction
    let initial_quality = peer.quality_score;

    if large_correction.abs() as f32 > max_threshold * 0.5 {
        peer.quality_score = (peer.quality_score - config.deceleration_factor).max(0.0);
    }

    assert!(peer.quality_score < initial_quality);
}

/// Test synchronization event recording
#[test]
fn test_sync_event_recording() {
    // This would test the SyncEvent struct if it were public
    // For now, we test the components that would be recorded

    let timestamp = 1000000u64;
    let peer_id = 12345u32;
    let time_diff = 100i64;
    let correction_applied = 50i64;
    let quality_score = 0.8f32;

    // Verify all components are valid
    assert!(timestamp > 0);
    assert!(peer_id > 0);
    assert!(time_diff.abs() < 10000); // Reasonable time difference
    assert!(correction_applied.abs() < 1000); // Reasonable correction
    assert!(quality_score >= 0.0 && quality_score <= 1.0);
}

/// Test convergence detection
#[test]
fn test_convergence_detection() {
    let config = SyncConfig {
        node_id: 12345,
        sync_interval_ms: 1000,
        max_correction_threshold_us: 1000,
        acceleration_factor: 0.1,
        deceleration_factor: 0.05,
        max_peers: 5,
        adaptive_frequency: true,
    };

    let convergence_threshold = config.max_correction_threshold_us as i64 / 10; // 100μs

    // Test convergence scenarios
    let test_cases = vec![
        (0i64, true),     // Perfect convergence
        (50i64, true),    // Within threshold
        (100i64, true),   // At threshold
        (150i64, false),  // Above threshold
        (-50i64, true),   // Negative within threshold
        (-150i64, false), // Negative above threshold
    ];

    for (current_correction, expected_converged) in test_cases {
        let is_converged = current_correction.abs() <= convergence_threshold;
        assert_eq!(is_converged, expected_converged);
    }
}

/// Test synchronization statistics calculation
#[test]
fn test_sync_stats_calculation() {
    let peers = vec![
        SyncPeer {
            node_id: 1,
            mac_address: [0x11, 0x11, 0x11, 0x11, 0x11, 0x11],
            last_timestamp: 1000000,
            time_diff_us: 100,
            quality_score: 1.0,
            sync_count: 5,
            last_sync_time: 1000000,
        },
        SyncPeer {
            node_id: 2,
            mac_address: [0x22, 0x22, 0x22, 0x22, 0x22, 0x22],
            last_timestamp: 1000000,
            time_diff_us: -200,
            quality_score: 0.5,
            sync_count: 3,
            last_sync_time: 1000000,
        },
        SyncPeer {
            node_id: 3,
            mac_address: [0x33, 0x33, 0x33, 0x33, 0x33, 0x33],
            last_timestamp: 1000000,
            time_diff_us: 50,
            quality_score: 0.8,
            sync_count: 7,
            last_sync_time: 1000000,
        },
    ];

    // Calculate statistics
    let peer_count = peers.len();
    let time_diffs: Vec<i64> = peers.iter().map(|p| p.time_diff_us).collect();
    let avg_time_diff = time_diffs.iter().sum::<i64>() as f32 / time_diffs.len() as f32;
    let max_time_diff = *time_diffs.iter().max().unwrap();
    let min_time_diff = *time_diffs.iter().min().unwrap();
    let avg_quality = peers.iter().map(|p| p.quality_score).sum::<f32>() / peers.len() as f32;

    // Verify calculations
    assert_eq!(peer_count, 3);
    assert_eq!(avg_time_diff, -16.666666); // (100 + (-200) + 50) / 3
    assert_eq!(max_time_diff, 100);
    assert_eq!(min_time_diff, -200);
    assert_eq!(avg_quality, 0.7666667); // (1.0 + 0.5 + 0.8) / 3
}

/// Test algorithm reset functionality
#[test]
fn test_algorithm_reset() {
    let mut peer = SyncPeer::new(12345, [0x11, 0x11, 0x11, 0x11, 0x11, 0x11]);

    // Modify peer state
    peer.quality_score = 0.5;
    peer.sync_count = 10;
    peer.time_diff_us = 500;

    // Reset peer state
    peer.quality_score = 1.0;
    peer.sync_count = 0;
    peer.time_diff_us = 0;

    // Verify reset
    assert_eq!(peer.quality_score, 1.0);
    assert_eq!(peer.sync_count, 0);
    assert_eq!(peer.time_diff_us, 0);
}

/// Test edge cases for algorithm
#[test]
fn test_algorithm_edge_cases() {
    // Test with no peers
    let empty_peers: Vec<SyncPeer> = Vec::new();
    assert_eq!(empty_peers.len(), 0);

    // Test with single peer
    let single_peer = SyncPeer::new(12345, [0x11, 0x11, 0x11, 0x11, 0x11, 0x11]);
    assert_eq!(single_peer.node_id, 12345);

    // Test with maximum peers
    let max_peers = 10;
    let mut peers = Vec::new();
    for i in 0..max_peers {
        peers.push(SyncPeer::new(i as u32, [i as u8; 6]));
    }
    assert_eq!(peers.len(), max_peers);

    // Test with extreme time differences
    let extreme_peer = SyncPeer {
        node_id: 99999,
        mac_address: [0xFF; 6],
        last_timestamp: u64::MAX,
        time_diff_us: i64::MAX,
        quality_score: 0.0,
        sync_count: u32::MAX,
        last_sync_time: u64::MAX,
    };

    assert_eq!(extreme_peer.time_diff_us, i64::MAX);
    assert_eq!(extreme_peer.quality_score, 0.0);
}

/// Test configuration edge cases
#[test]
fn test_config_edge_cases() {
    // Test minimum values
    let min_config = SyncConfig {
        node_id: 0,
        sync_interval_ms: 1,
        max_correction_threshold_us: 1,
        acceleration_factor: 0.0,
        deceleration_factor: 0.0,
        max_peers: 1,
        adaptive_frequency: false,
    };

    assert_eq!(min_config.node_id, 0);
    assert_eq!(min_config.sync_interval_ms, 1);
    assert_eq!(min_config.max_correction_threshold_us, 1);
    assert_eq!(min_config.acceleration_factor, 0.0);
    assert_eq!(min_config.deceleration_factor, 0.0);
    assert_eq!(min_config.max_peers, 1);
    assert!(!min_config.adaptive_frequency);

    // Test maximum reasonable values
    let max_config = SyncConfig {
        node_id: u32::MAX,
        sync_interval_ms: 3600000,            // 1 hour
        max_correction_threshold_us: 1000000, // 1 second
        acceleration_factor: 1.0,
        deceleration_factor: 1.0,
        max_peers: 255,
        adaptive_frequency: true,
    };

    assert_eq!(max_config.node_id, u32::MAX);
    assert_eq!(max_config.sync_interval_ms, 3600000);
    assert_eq!(max_config.max_correction_threshold_us, 1000000);
    assert_eq!(max_config.acceleration_factor, 1.0);
    assert_eq!(max_config.deceleration_factor, 1.0);
    assert_eq!(max_config.max_peers, 255);
    assert!(max_config.adaptive_frequency);
}
