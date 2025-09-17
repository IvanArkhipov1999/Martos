//! Time synchronization algorithm implementation.
//!
//! This module implements the core time synchronization algorithm based on
//! dynamic time acceleration/deceleration approach described in the paper
//! "Comparing time. A New Approach To The Problem Of Time Synchronization In a Multi-agent System".

use crate::time_sync::{SyncConfig, SyncError, SyncPeer, SyncResult};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Core synchronization algorithm implementation
pub struct SyncAlgorithm {
    config: SyncConfig,
    peers: BTreeMap<u32, SyncPeer>,
    sync_history: Vec<SyncEvent>,
    current_correction: i64,
    convergence_threshold: i64,
}

/// Represents a synchronization event for tracking algorithm performance
#[derive(Debug, Clone)]
pub struct SyncEvent {
    pub timestamp: u64,
    pub peer_id: u32,
    pub time_diff: i64,
    pub correction_applied: i64,
    pub quality_score: f32,
}

impl SyncAlgorithm {
    /// Create a new synchronization algorithm instance
    pub fn new(config: SyncConfig) -> Self {
        let convergence_threshold = config.max_correction_threshold_us as i64 / 10; // 10% of max threshold
        Self {
            config,
            peers: BTreeMap::new(),
            sync_history: Vec::new(),
            current_correction: 0,
            convergence_threshold,
        }
    }

    /// Process a synchronization message and calculate time correction
    pub fn process_sync_message(
        &mut self,
        peer_id: u32,
        remote_timestamp: u64,
        local_timestamp: u64,
    ) -> SyncResult<i64> {
        // Calculate time difference
        let time_diff = remote_timestamp as i64 - local_timestamp as i64;

        // Update peer information
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            peer.last_timestamp = remote_timestamp;
            peer.time_diff_us = time_diff;
            peer.last_sync_time = local_timestamp;
        } else {
            // Add new peer if not exists
            let mut new_peer = SyncPeer::new(peer_id, [0; 6]); // MAC will be set separately
            new_peer.last_timestamp = remote_timestamp;
            new_peer.time_diff_us = time_diff;
            new_peer.last_sync_time = local_timestamp;
            self.peers.insert(peer_id, new_peer);
        }

        // Calculate correction using dynamic acceleration/deceleration
        let correction = self.calculate_dynamic_correction(peer_id, time_diff)?;

        // Record synchronization event
        self.record_sync_event(local_timestamp, peer_id, time_diff, correction);

        Ok(correction)
    }

    /// Calculate time correction using dynamic acceleration/deceleration algorithm
    fn calculate_dynamic_correction(&mut self, peer_id: u32, _time_diff: i64) -> SyncResult<i64> {
        let _peer = self.peers.get(&peer_id).ok_or(SyncError::PeerNotFound)?;

        // Calculate weighted average of time differences from all peers
        let weighted_diff = self.calculate_weighted_average_diff();

        // Apply dynamic acceleration/deceleration based on convergence
        let acceleration_factor = self.calculate_acceleration_factor(weighted_diff);
        let correction = (weighted_diff as f64 * acceleration_factor) as i64;

        // Apply bounds checking
        let bounded_correction = self.apply_correction_bounds(correction);

        // Update current correction
        self.current_correction += bounded_correction;

        // Update peer quality based on correction success
        self.update_peer_quality(peer_id, bounded_correction);

        Ok(bounded_correction)
    }

    /// Calculate weighted average of time differences from all peers
    fn calculate_weighted_average_diff(&self) -> i64 {
        if self.peers.is_empty() {
            return 0;
        }

        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for peer in self.peers.values() {
            let weight = peer.quality_score;
            weighted_sum += peer.time_diff_us as f32 * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            (weighted_sum / total_weight) as i64
        } else {
            0
        }
    }

    /// Calculate acceleration factor based on convergence state
    fn calculate_acceleration_factor(&self, time_diff: i64) -> f64 {
        let abs_diff = time_diff.abs() as f64;
        let max_threshold = self.config.max_correction_threshold_us as f64;

        if abs_diff <= self.convergence_threshold as f64 {
            // Close to convergence - use deceleration factor
            self.config.deceleration_factor as f64
        } else if abs_diff <= max_threshold {
            // Moderate difference - use acceleration factor
            self.config.acceleration_factor as f64
        } else {
            // Large difference - use reduced acceleration to prevent instability
            self.config.acceleration_factor as f64 * 0.5
        }
    }

    /// Apply bounds checking to correction value
    fn apply_correction_bounds(&self, correction: i64) -> i64 {
        let max_correction = self.config.max_correction_threshold_us as i64;

        if correction > max_correction {
            max_correction
        } else if correction < -max_correction {
            -max_correction
        } else {
            correction
        }
    }

    /// Update peer quality score based on synchronization results
    fn update_peer_quality(&mut self, peer_id: u32, correction_applied: i64) {
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            // Quality improves if correction is small and consistent
            let correction_magnitude = correction_applied.abs() as f32;
            let max_threshold = self.config.max_correction_threshold_us as f32;

            if correction_magnitude <= max_threshold * 0.1 {
                // Small correction - good quality
                peer.quality_score =
                    (peer.quality_score + self.config.acceleration_factor).min(1.0);
            } else if correction_magnitude <= max_threshold * 0.5 {
                // Moderate correction - maintain quality
                // No change to quality score
            } else {
                // Large correction - reduce quality
                peer.quality_score =
                    (peer.quality_score - self.config.deceleration_factor).max(0.0);
            }

            peer.sync_count += 1;
        }
    }

    /// Record a synchronization event for analysis
    fn record_sync_event(&mut self, timestamp: u64, peer_id: u32, time_diff: i64, correction: i64) {
        let quality_score = self
            .peers
            .get(&peer_id)
            .map(|p| p.quality_score)
            .unwrap_or(0.0);

        let event = SyncEvent {
            timestamp,
            peer_id,
            time_diff,
            correction_applied: correction,
            quality_score,
        };

        self.sync_history.push(event);

        // Keep only recent history (last 100 events)
        if self.sync_history.len() > 100 {
            self.sync_history.remove(0);
        }
    }

    /// Check if the synchronization algorithm has converged
    pub fn is_converged(&self) -> bool {
        self.current_correction.abs() <= self.convergence_threshold
    }

    /// Get the current synchronization quality score
    pub fn get_sync_quality(&self) -> f32 {
        if self.peers.is_empty() {
            return 0.0;
        }

        let mut total_quality = 0.0;
        for peer in self.peers.values() {
            total_quality += peer.quality_score;
        }

        total_quality / self.peers.len() as f32
    }

    /// Get synchronization statistics
    pub fn get_sync_stats(&self) -> SyncStats {
        let mut avg_time_diff = 0.0;
        let mut max_time_diff = 0i64;
        let mut min_time_diff = 0i64;

        if !self.peers.is_empty() {
            let mut time_diffs: Vec<i64> = self.peers.values().map(|p| p.time_diff_us).collect();
            time_diffs.sort();

            avg_time_diff = time_diffs.iter().sum::<i64>() as f32 / time_diffs.len() as f32;
            max_time_diff = *time_diffs.last().unwrap_or(&0);
            min_time_diff = *time_diffs.first().unwrap_or(&0);
        }

        SyncStats {
            peer_count: self.peers.len(),
            avg_time_diff_us: avg_time_diff,
            max_time_diff_us: max_time_diff,
            min_time_diff_us: min_time_diff,
            current_correction_us: self.current_correction,
            sync_quality: self.get_sync_quality(),
            is_converged: self.is_converged(),
        }
    }

    /// Add or update a peer
    pub fn add_peer(&mut self, peer: SyncPeer) {
        self.peers.insert(peer.node_id, peer);
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: u32) {
        self.peers.remove(&peer_id);
    }

    /// Get all peers
    pub fn get_peers(&self) -> Vec<SyncPeer> {
        self.peers.values().cloned().collect()
    }

    /// Get peer by ID
    pub fn get_peer(&self, peer_id: u32) -> Option<&SyncPeer> {
        self.peers.get(&peer_id)
    }

    /// Reset synchronization state
    pub fn reset(&mut self) {
        self.current_correction = 0;
        self.sync_history.clear();
        for peer in self.peers.values_mut() {
            peer.quality_score = 1.0;
            peer.sync_count = 0;
        }
    }
}

/// Synchronization statistics
#[derive(Debug, Clone)]
pub struct SyncStats {
    pub peer_count: usize,
    pub avg_time_diff_us: f32,
    pub max_time_diff_us: i64,
    pub min_time_diff_us: i64,
    pub current_correction_us: i64,
    pub sync_quality: f32,
    pub is_converged: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_algorithm_creation() {
        let config = SyncConfig::default();
        let algorithm = SyncAlgorithm::new(config);

        assert_eq!(algorithm.peers.len(), 0);
        assert_eq!(algorithm.current_correction, 0);
    }

    #[test]
    fn test_process_sync_message() {
        let config = SyncConfig::default();
        let mut algorithm = SyncAlgorithm::new(config);

        let correction = algorithm.process_sync_message(123, 1000, 1100).unwrap();

        // Should calculate correction based on time difference
        assert!(correction != 0);
        assert!(algorithm.peers.contains_key(&123));
    }

    #[test]
    fn test_weighted_average_calculation() {
        let config = SyncConfig::default();
        let mut algorithm = SyncAlgorithm::new(config);

        // Add peers with different quality scores
        let mut peer1 = SyncPeer::new(1, [0; 6]);
        peer1.time_diff_us = 100;
        peer1.quality_score = 1.0;

        let mut peer2 = SyncPeer::new(2, [0; 6]);
        peer2.time_diff_us = 200;
        peer2.quality_score = 0.5;

        algorithm.add_peer(peer1);
        algorithm.add_peer(peer2);

        let weighted_avg = algorithm.calculate_weighted_average_diff();

        // Should be closer to peer1's value due to higher quality
        assert!(weighted_avg < 150); // Less than simple average
    }
}
