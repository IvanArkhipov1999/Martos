//! Time synchronization algorithm implementation.
//!
//! This module implements the core Local Voting Protocol algorithm based on
//! dynamic time acceleration/deceleration approach described in the paper
//! "Comparing time. A New Approach To The Problem Of Time Synchronization In a Multi-agent System".
//!
//! # Algorithm Overview
//!
//! The Local Voting Protocol works by having each node vote on the correct time
//! based on information from its peers. The algorithm uses weighted averaging
//! where peers with higher quality scores have more influence on the final decision.
//!
//! # Key Components
//!
//! - **SyncAlgorithm**: Main algorithm implementation
//! - **SyncEvent**: Tracks synchronization events for analysis
//! - **Weighted Averaging**: Calculates consensus time based on peer quality
//! - **Dynamic Correction**: Applies acceleration/deceleration factors
//!
//! # Correction Strategy
//!
//! The algorithm uses different correction factors based on the magnitude
//! of time differences:
//!
//! - **Large differences**: Use acceleration factor for rapid correction
//! - **Small differences**: Use deceleration factor for stable convergence
//! - **Convergence threshold**: Defines the boundary between acceleration/deceleration
//!
//! # Quality Assessment
//!
//! Peer quality scores are updated based on:
//! - Consistency of time differences
//! - Frequency of successful synchronizations
//! - Stability of communication patterns

use crate::time_sync::{SyncConfig, SyncError, SyncPeer, SyncResult};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

/// Core Local Voting Protocol synchronization algorithm implementation.
///
/// This algorithm implements the consensus-based time synchronization approach
/// where each node votes on the correct time based on peer information. The
/// algorithm uses weighted averaging with quality-based peer influence.
///
/// # Algorithm Flow
///
/// 1. **Peer Analysis**: Calculate time differences with all peers
/// 2. **Quality Weighting**: Weight peer contributions by their quality scores
/// 3. **Consensus Calculation**: Compute weighted average of time differences
/// 4. **Dynamic Correction**: Apply acceleration/deceleration based on magnitude
/// 5. **Quality Update**: Update peer quality scores based on consistency
///
/// # Thread Safety
///
/// The algorithm is designed to be called from a single thread context
/// and maintains internal state for peer tracking and history.
pub struct SyncAlgorithm {
    /// Configuration parameters for algorithm behavior
    config: SyncConfig,
    /// Map of tracked peers (node_id -> SyncPeer)
    peers: BTreeMap<u32, SyncPeer>,
    /// History of synchronization events for analysis
    sync_history: Vec<SyncEvent>,
    /// Current accumulated correction value
    current_correction: i64,
    /// Threshold for switching between acceleration/deceleration
    convergence_threshold: i64,
}

/// Represents a synchronization event for tracking algorithm performance.
///
/// This structure records each synchronization event for analysis and
/// debugging purposes. It contains all relevant information about
/// the synchronization process.
#[derive(Debug, Clone)]
pub struct SyncEvent {
    /// Timestamp when the event occurred (microseconds)
    pub timestamp: u64,
    /// ID of the peer involved in synchronization
    pub peer_id: u32,
    /// Time difference calculated for this peer (microseconds)
    pub time_diff: i64,
    /// Correction value applied as result of this event (microseconds)
    pub correction_applied: i64,
    /// Quality score of the peer at time of event (0.0-1.0)
    pub quality_score: f32,
}

impl SyncAlgorithm {
    /// Create a new synchronization algorithm instance.
    ///
    /// Initializes the algorithm with the provided configuration and sets up
    /// the convergence threshold for switching between acceleration and deceleration modes.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration parameters for the algorithm
    ///
    /// # Returns
    ///
    /// A new `SyncAlgorithm` instance ready for use.
    pub fn new(config: SyncConfig) -> Self {
        let convergence_threshold = config.max_correction_threshold_us as i64 / 2; // 50% of max threshold
        Self {
            config,
            peers: BTreeMap::new(),
            sync_history: Vec::new(),
            current_correction: 0,
            convergence_threshold,
        }
    }

    /// Process a synchronization message and calculate time correction.
    ///
    /// This is the main entry point for the Local Voting Protocol algorithm.
    /// It processes incoming synchronization data, updates peer information,
    /// and calculates the appropriate time correction to apply.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Unique identifier of the peer node
    /// * `remote_timestamp` - Timestamp received from the peer (microseconds)
    /// * `local_timestamp` - Local timestamp when message was received (microseconds)
    ///
    /// # Returns
    ///
    /// * `Ok(correction)` - Time correction to apply in microseconds
    /// * `Err(SyncError)` - Error if processing fails
    ///
    /// # Algorithm Steps
    ///
    /// 1. Calculate time difference between local and remote timestamps
    /// 2. Update or create peer information
    /// 3. Calculate weighted average of all peer time differences
    /// 4. Apply Local Voting Protocol correction algorithm
    /// 5. Record synchronization event for analysis
    /// 6. Update peer quality scores
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

    /// Calculate time correction using Local Voting Protocol.
    ///
    /// Implements the core Local Voting Protocol algorithm by calculating
    /// weighted average of time differences from all peers.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - ID of the peer that triggered the calculation
    /// * `_time_diff` - Time difference (currently unused)
    ///
    /// # Returns
    ///
    /// * `Ok(correction)` - Calculated time correction in microseconds
    /// * `Err(SyncError)` - Error if peer not found
    fn calculate_dynamic_correction(&mut self, peer_id: u32, _time_diff: i64) -> SyncResult<i64> {
        let _peer = self.peers.get(&peer_id).ok_or(SyncError::PeerNotFound)?;

        // Local Voting Protocol: Calculate weighted average of time differences from all peers
        let weighted_diff = self.calculate_weighted_average_diff();

        // Apply Local Voting Protocol correction
        // If our time is ahead (positive diff), we should slow down
        // If our time is behind (negative diff), we should speed up
        let correction = self.calculate_local_voting_correction(weighted_diff);

        // Apply bounds checking
        let bounded_correction = self.apply_correction_bounds(correction);

        // Update current correction
        self.current_correction += bounded_correction;

        // Update peer quality based on correction success
        self.update_peer_quality(peer_id, bounded_correction);

        Ok(bounded_correction)
    }

    /// Calculate weighted average of time differences from all peers.
    ///
    /// Computes the consensus time difference by weighting each peer's
    /// contribution by their quality score.
    ///
    /// # Returns
    ///
    /// Weighted average time difference in microseconds
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

    /// Calculate Local Voting Protocol correction.
    ///
    /// Applies the Local Voting Protocol algorithm to determine the appropriate
    /// time correction based on the weighted difference and convergence state.
    ///
    /// # Arguments
    ///
    /// * `weighted_diff` - Weighted average time difference from all peers
    ///
    /// # Returns
    ///
    /// Time correction to apply in microseconds
    fn calculate_local_voting_correction(&self, weighted_diff: i64) -> i64 {
        let abs_diff = weighted_diff.abs() as f64;
        let max_threshold = self.config.max_correction_threshold_us as f64;

        // Local Voting Protocol: Apply correction based on weighted difference
        let correction_factor = if abs_diff <= self.convergence_threshold as f64 {
            // Close to convergence - use deceleration factor
            self.config.deceleration_factor as f64
        } else if abs_diff <= max_threshold {
            // Moderate difference - use acceleration factor
            self.config.acceleration_factor as f64
        } else {
            // Large difference - use reduced acceleration to prevent instability
            self.config.acceleration_factor as f64 * 0.5
        };

        // Apply correction: if we're ahead (positive), slow down (negative correction)
        // If we're behind (negative), speed up (positive correction)
        let correction = (weighted_diff as f64 * correction_factor) as i64;
        
        // For Local Voting Protocol, we want to slow down when ahead, speed up when behind
        // But we never want to go backwards in time, so we limit negative corrections
        if correction < 0 {
            // When slowing down, limit the slowdown to prevent time going backwards
            correction.max(-(max_threshold as i64 / 10))
        } else {
            correction
        }
    }

    /// Apply bounds checking to correction value.
    ///
    /// Ensures that the correction value does not exceed the maximum
    /// threshold to prevent instability.
    ///
    /// # Arguments
    ///
    /// * `correction` - Correction value to bound
    ///
    /// # Returns
    ///
    /// Bounded correction value
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

    /// Update peer quality score based on synchronization results.
    ///
    /// Adjusts the quality score of a peer based on the consistency
    /// of synchronization results.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - ID of the peer to update
    /// * `correction_applied` - Correction value that was applied
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

    /// Record a synchronization event for analysis.
    ///
    /// Adds a synchronization event to the history for performance
    /// analysis and debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - When the event occurred
    /// * `peer_id` - ID of the peer involved
    /// * `time_diff` - Time difference calculated
    /// * `correction` - Correction value applied
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

    /// Check if the synchronization algorithm has converged.
    ///
    /// Determines whether the algorithm has reached a stable state where
    /// time corrections are within the convergence threshold.
    ///
    /// # Returns
    ///
    /// * `true` - Algorithm has converged (stable state)
    /// * `false` - Algorithm is still adjusting (unstable state)
    pub fn is_converged(&self) -> bool {
        self.current_correction.abs() <= self.convergence_threshold
    }

    /// Get the current synchronization quality score.
    ///
    /// Calculates the overall quality of the synchronization process based on
    /// peer consistency and algorithm convergence.
    ///
    /// # Returns
    ///
    /// Quality score between 0.0 (poor) and 1.0 (excellent)
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

    /// Get synchronization statistics.
    ///
    /// Returns detailed statistics about the algorithm's performance including
    /// convergence metrics, peer quality, and correction history.
    ///
    /// # Returns
    ///
    /// `SyncStats` structure containing performance metrics
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

    /// Add or update a peer.
    ///
    /// Adds a new peer to the algorithm or updates an existing peer's information.
    ///
    /// # Arguments
    ///
    /// * `peer` - Peer information to add or update
    pub fn add_peer(&mut self, peer: SyncPeer) {
        self.peers.insert(peer.node_id, peer);
    }

    /// Remove a peer.
    ///
    /// Removes a peer from the algorithm's tracking.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - ID of the peer to remove
    pub fn remove_peer(&mut self, peer_id: u32) {
        self.peers.remove(&peer_id);
    }

    /// Get all peers.
    ///
    /// Returns a copy of all currently tracked peers.
    ///
    /// # Returns
    ///
    /// Vector containing all active `SyncPeer` instances
    pub fn get_peers(&self) -> Vec<SyncPeer> {
        self.peers.values().cloned().collect()
    }

    /// Get peer by ID.
    ///
    /// Retrieves information about a specific peer.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - ID of the peer to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(peer)` - Reference to the peer if found
    /// * `None` - Peer not found
    pub fn get_peer(&self, peer_id: u32) -> Option<&SyncPeer> {
        self.peers.get(&peer_id)
    }

    /// Reset synchronization state.
    ///
    /// Clears all peer information, synchronization history, and resets
    /// the algorithm to its initial state. Useful for restarting synchronization
    /// or clearing accumulated state.
    pub fn reset(&mut self) {
        self.current_correction = 0;
        self.sync_history.clear();
        for peer in self.peers.values_mut() {
            peer.quality_score = 1.0;
            peer.sync_count = 0;
        }
    }
}

/// Synchronization statistics for algorithm performance analysis.
///
/// This structure contains comprehensive metrics about the synchronization
/// algorithm's performance, including peer statistics, convergence state,
/// and quality metrics.
#[derive(Debug, Clone)]
pub struct SyncStats {
    /// Number of active peers in the synchronization network
    pub peer_count: usize,
    /// Average time difference across all peers (microseconds)
    pub avg_time_diff_us: f32,
    /// Maximum time difference observed (microseconds)
    pub max_time_diff_us: i64,
    /// Minimum time difference observed (microseconds)
    pub min_time_diff_us: i64,
    /// Current correction value being applied (microseconds)
    pub current_correction_us: i64,
    /// Overall synchronization quality score (0.0-1.0)
    pub sync_quality: f32,
    /// Whether the algorithm has converged to a stable state
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
