//! Time synchronization module for Martos RTOS.
//!
//! This module implements time synchronization between nodes in a multi-agent system
//! using ESP-NOW communication protocol. The synchronization algorithm is based on
//! dynamic time acceleration/deceleration approach described in the paper
//! "Comparing time. A New Approach To The Problem Of Time Synchronization In a Multi-agent System"
//! from PROCEEDING OF THE 36TH CONFERENCE OF FRUCT ASSOCIATION.
//!
//! # Architecture Overview
//!
//! The time synchronization system consists of several key components:
//!
//! - **TimeSyncManager**: Main synchronization controller
//! - **SyncPeer**: Represents a synchronized peer node
//! - **SyncMessage**: Communication protocol for time data exchange
//! - **SyncAlgorithm**: Core synchronization algorithm implementation
//!
//! # Synchronization Algorithm
//!
//! The algorithm uses dynamic time adjustment based on:
//! 1. Time difference calculation between local and remote timestamps
//! 2. Gradual time correction using acceleration/deceleration factors
//! 3. Consensus-based synchronization with multiple peers
//! 4. Adaptive synchronization frequency based on network conditions
//!
//! # Usage
//!
//! ```rust
//! use martos::time_sync::{TimeSyncManager, SyncConfig};
//! use martos::timer::Timer;
//!
//! // Initialize synchronization manager
//! let mut sync_manager = TimeSyncManager::new(SyncConfig::default());
//!
//! // Enable synchronization
//! sync_manager.enable_sync();
//!
//! // In your main loop:
//! loop {
//!     sync_manager.process_sync_cycle();
//!     // Your application logic here
//! }
//! ```

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use core::time::Duration;

#[cfg(feature = "network")]
pub mod esp_now_protocol;
#[cfg(feature = "network")]
pub mod sync_algorithm;

/// Configuration parameters for time synchronization
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Node identifier for this device
    pub node_id: u32,
    /// Synchronization interval in milliseconds
    pub sync_interval_ms: u32,
    /// Maximum time difference threshold for correction (microseconds)
    pub max_correction_threshold_us: u64,
    /// Acceleration factor for time correction (0.0 to 1.0)
    pub acceleration_factor: f32,
    /// Deceleration factor for time correction (0.0 to 1.0)
    pub deceleration_factor: f32,
    /// Maximum number of peers to synchronize with
    pub max_peers: usize,
    /// Enable adaptive synchronization frequency
    pub adaptive_frequency: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            node_id: 0,
            sync_interval_ms: 1000,            // 1 second
            max_correction_threshold_us: 1000, // 1ms
            acceleration_factor: 0.1,
            deceleration_factor: 0.05,
            max_peers: 10,
            adaptive_frequency: true,
        }
    }
}

/// Represents a synchronized peer node
#[derive(Debug, Clone)]
pub struct SyncPeer {
    /// Peer node identifier
    pub node_id: u32,
    /// MAC address of the peer
    pub mac_address: [u8; 6],
    /// Last received timestamp from this peer
    pub last_timestamp: u64,
    /// Time difference with this peer (microseconds)
    pub time_diff_us: i64,
    /// Quality score for this peer (0.0 to 1.0)
    pub quality_score: f32,
    /// Number of successful synchronizations
    pub sync_count: u32,
    /// Last synchronization time
    pub last_sync_time: u64,
}

impl SyncPeer {
    pub fn new(node_id: u32, mac_address: [u8; 6]) -> Self {
        Self {
            node_id,
            mac_address,
            last_timestamp: 0,
            time_diff_us: 0,
            quality_score: 1.0,
            sync_count: 0,
            last_sync_time: 0,
        }
    }
}

/// Message types for ESP-NOW communication
#[derive(Debug, Clone, Copy)]
pub enum SyncMessageType {
    /// Request for time synchronization
    SyncRequest = 0x01,
    /// Response with current timestamp
    SyncResponse = 0x02,
    /// Broadcast time announcement
    TimeBroadcast = 0x03,
}

/// Synchronization message structure
#[derive(Debug, Clone)]
pub struct SyncMessage {
    /// Message type
    pub msg_type: SyncMessageType,
    /// Source node ID
    pub source_node_id: u32,
    /// Target node ID (0 for broadcast)
    pub target_node_id: u32,
    /// Timestamp when message was sent (microseconds)
    pub timestamp_us: u64,
    /// Message sequence number
    pub sequence: u32,
    /// Additional data payload
    pub payload: Vec<u8>,
}

impl SyncMessage {
    /// Create a new synchronization request message
    pub fn new_sync_request(source_node_id: u32, target_node_id: u32, timestamp_us: u64) -> Self {
        Self {
            msg_type: SyncMessageType::SyncRequest,
            source_node_id,
            target_node_id,
            timestamp_us,
            sequence: 0,
            payload: Vec::new(),
        }
    }

    /// Create a new synchronization response message
    pub fn new_sync_response(source_node_id: u32, target_node_id: u32, timestamp_us: u64) -> Self {
        Self {
            msg_type: SyncMessageType::SyncResponse,
            source_node_id,
            target_node_id,
            timestamp_us,
            sequence: 0,
            payload: Vec::new(),
        }
    }

    /// Serialize message to bytes for ESP-NOW transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(32);

        // Message type (1 byte)
        data.push(self.msg_type as u8);

        // Source node ID (4 bytes)
        data.extend_from_slice(&self.source_node_id.to_le_bytes());

        // Target node ID (4 bytes)
        data.extend_from_slice(&self.target_node_id.to_le_bytes());

        // Timestamp (8 bytes)
        data.extend_from_slice(&self.timestamp_us.to_le_bytes());

        // Sequence number (4 bytes)
        data.extend_from_slice(&self.sequence.to_le_bytes());

        // Payload length (2 bytes)
        data.extend_from_slice(&(self.payload.len() as u16).to_le_bytes());

        // Payload data
        data.extend_from_slice(&self.payload);

        data
    }

    /// Deserialize message from bytes received via ESP-NOW
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 23 {
            // Minimum message size
            return None;
        }

        let mut offset = 0;

        // Message type
        let msg_type = match data[offset] {
            0x01 => SyncMessageType::SyncRequest,
            0x02 => SyncMessageType::SyncResponse,
            0x03 => SyncMessageType::TimeBroadcast,
            _ => return None,
        };
        offset += 1;

        // Source node ID
        let source_node_id = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Target node ID
        let target_node_id = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Timestamp
        let timestamp_us = u64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        offset += 8;

        // Sequence number
        let sequence = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Payload length
        let payload_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        // Check if we have enough data for payload
        if data.len() < offset + payload_len {
            return None;
        }

        // Payload
        let payload = data[offset..offset + payload_len].to_vec();

        Some(Self {
            msg_type,
            source_node_id,
            target_node_id,
            timestamp_us,
            sequence,
            payload,
        })
    }
}

/// Main time synchronization manager
pub struct TimeSyncManager {
    /// Configuration parameters
    config: SyncConfig,
    /// Synchronization enabled flag
    sync_enabled: AtomicBool,
    /// Current time offset in microseconds
    time_offset_us: AtomicI64,
    /// Last synchronization time
    last_sync_time: AtomicU64,
    /// Synchronized peers
    peers: BTreeMap<u32, SyncPeer>,
    /// Message sequence counter
    sequence_counter: AtomicU64,
    /// Current synchronization quality score
    sync_quality: AtomicU64, // Stored as fixed-point (0.0-1.0 * 1000)
    /// ESP-NOW protocol handler (only available with network feature)
    #[cfg(feature = "network")]
    esp_now_protocol: Option<crate::time_sync::esp_now_protocol::EspNowTimeSyncProtocol>,
    /// Synchronization algorithm instance (only available with network feature)
    #[cfg(feature = "network")]
    sync_algorithm: Option<crate::time_sync::sync_algorithm::SyncAlgorithm>,
}

impl TimeSyncManager {
    /// Create a new time synchronization manager
    pub fn new(config: SyncConfig) -> Self {
        Self {
            config,
            sync_enabled: AtomicBool::new(false),
            time_offset_us: AtomicI64::new(0),
            last_sync_time: AtomicU64::new(0),
            peers: BTreeMap::new(),
            sequence_counter: AtomicU64::new(0),
            sync_quality: AtomicU64::new(1000), // Start with perfect quality
            #[cfg(feature = "network")]
            esp_now_protocol: None,
            #[cfg(feature = "network")]
            sync_algorithm: Some(crate::time_sync::sync_algorithm::SyncAlgorithm::new(
                config.clone(),
            )),
        }
    }

    /// Enable time synchronization
    pub fn enable_sync(&mut self) {
        self.sync_enabled.store(true, Ordering::Release);
    }

    /// Disable time synchronization
    pub fn disable_sync(&mut self) {
        self.sync_enabled.store(false, Ordering::Release);
    }

    /// Check if synchronization is enabled
    pub fn is_sync_enabled(&self) -> bool {
        self.sync_enabled.load(Ordering::Acquire)
    }

    /// Add a peer for synchronization
    pub fn add_peer(&mut self, peer: SyncPeer) {
        if self.peers.len() < self.config.max_peers {
            self.peers.insert(peer.node_id, peer);
        }
    }

    /// Remove a peer from synchronization
    pub fn remove_peer(&mut self, node_id: u32) {
        self.peers.remove(&node_id);
    }

    /// Get current time offset in microseconds
    pub fn get_time_offset_us(&self) -> i64 {
        self.time_offset_us.load(Ordering::Acquire)
    }

    /// Get synchronization quality score (0.0 to 1.0)
    pub fn get_sync_quality(&self) -> f32 {
        self.sync_quality.load(Ordering::Acquire) as f32 / 1000.0
    }

    /// Process one synchronization cycle
    /// This should be called periodically from the main application loop
    pub fn process_sync_cycle(&mut self) {
        if !self.is_sync_enabled() {
            return;
        }

        // TODO: Implement actual synchronization logic
        // This will include:
        // 1. Sending sync requests to peers
        // 2. Processing received sync messages
        // 3. Calculating time differences
        // 4. Applying time corrections
        // 5. Updating peer quality scores
    }

    /// Handle incoming synchronization message
    pub fn handle_sync_message(&mut self, message: SyncMessage) {
        if !self.is_sync_enabled() {
            return;
        }

        match message.msg_type {
            SyncMessageType::SyncRequest => {
                self.handle_sync_request(message);
            }
            SyncMessageType::SyncResponse => {
                self.handle_sync_response(message);
            }
            SyncMessageType::TimeBroadcast => {
                self.handle_time_broadcast(message);
            }
        }
    }

    /// Handle synchronization request from a peer
    fn handle_sync_request(&mut self, message: SyncMessage) {
        // TODO: Implement sync request handling
        // This should send a response with current timestamp
    }

    /// Handle synchronization response from a peer
    fn handle_sync_response(&mut self, message: SyncMessage) {
        // TODO: Implement sync response handling
        // This should calculate time difference and update peer info
    }

    /// Handle time broadcast from a peer
    fn handle_time_broadcast(&mut self, message: SyncMessage) {
        // TODO: Implement time broadcast handling
        // This should update peer time information
    }

    /// Calculate time correction based on peer data
    fn calculate_time_correction(&self, peer: &SyncPeer) -> i64 {
        // TODO: Implement time correction calculation
        // This should use the dynamic acceleration/deceleration algorithm
        0
    }

    /// Apply time correction to the system
    fn apply_time_correction(&mut self, correction_us: i64) {
        if correction_us.abs() > self.config.max_correction_threshold_us as i64 {
            return; // Skip correction if too large
        }

        let current_offset = self.time_offset_us.load(Ordering::Acquire);
        let new_offset = current_offset + correction_us;
        self.time_offset_us.store(new_offset, Ordering::Release);
    }

    /// Update peer quality score based on synchronization results
    fn update_peer_quality(&mut self, node_id: u32, success: bool) {
        if let Some(peer) = self.peers.get_mut(&node_id) {
            if success {
                peer.quality_score =
                    (peer.quality_score + self.config.acceleration_factor).min(1.0);
                peer.sync_count += 1;
            } else {
                peer.quality_score =
                    (peer.quality_score - self.config.deceleration_factor).max(0.0);
            }
        }
    }

    /// Get list of active peers
    pub fn get_peers(&self) -> Vec<SyncPeer> {
        self.peers.values().cloned().collect()
    }

    /// Get peer by node ID
    pub fn get_peer(&self, node_id: u32) -> Option<&SyncPeer> {
        self.peers.get(&node_id)
    }

    /// Initialize ESP-NOW protocol handler
    #[cfg(feature = "network")]
    pub fn init_esp_now_protocol(
        &mut self,
        esp_now: esp_wifi::esp_now::EspNow<'static>,
        local_mac: [u8; 6],
    ) {
        self.esp_now_protocol = Some(
            crate::time_sync::esp_now_protocol::EspNowTimeSyncProtocol::new(
                esp_now,
                self.config.node_id,
                local_mac,
            ),
        );
    }

    /// Process one synchronization cycle with ESP-NOW
    #[cfg(feature = "network")]
    pub fn process_sync_cycle_with_esp_now(&mut self, current_time_us: u64) {
        if !self.is_sync_enabled() {
            return;
        }

        if let Some(ref mut protocol) = self.esp_now_protocol {
            // Receive and process incoming messages
            let messages = protocol.receive_messages();
            for message in messages {
                self.handle_sync_message(message);
            }

            // Send periodic sync requests
            if current_time_us - self.last_sync_time.load(Ordering::Acquire)
                >= self.config.sync_interval_ms as u64 * 1000
            {
                self.send_periodic_sync_requests(protocol, current_time_us);
                self.last_sync_time
                    .store(current_time_us, Ordering::Release);
            }
        }
    }

    /// Send periodic synchronization requests to all peers
    #[cfg(feature = "network")]
    fn send_periodic_sync_requests(
        &mut self,
        protocol: &mut crate::time_sync::esp_now_protocol::EspNowTimeSyncProtocol,
        current_time_us: u64,
    ) {
        for peer in self.peers.values() {
            if peer.quality_score > 0.1 {
                // Only sync with good quality peers
                let _ = protocol.send_sync_request(&peer.mac_address, current_time_us);
            }
        }
    }

    /// Handle incoming synchronization message with algorithm integration
    #[cfg(feature = "network")]
    fn handle_sync_message_with_algorithm(&mut self, message: SyncMessage, current_time_us: u64) {
        if let Some(ref mut algorithm) = self.sync_algorithm {
            match message.msg_type {
                SyncMessageType::SyncRequest => {
                    // Send response
                    if let Some(ref mut protocol) = self.esp_now_protocol {
                        let _ = protocol.send_sync_response(
                            &message.source_node_id.to_le_bytes(),
                            message.source_node_id,
                            current_time_us,
                        );
                    }
                }
                SyncMessageType::SyncResponse => {
                    // Process response and calculate correction
                    if let Ok(correction) = algorithm.process_sync_message(
                        message.source_node_id,
                        message.timestamp_us,
                        current_time_us,
                    ) {
                        self.apply_time_correction(correction);
                    }
                }
                SyncMessageType::TimeBroadcast => {
                    // Process broadcast and calculate correction
                    if let Ok(correction) = algorithm.process_sync_message(
                        message.source_node_id,
                        message.timestamp_us,
                        current_time_us,
                    ) {
                        self.apply_time_correction(correction);
                    }
                }
            }
        }
    }

    /// Get synchronization statistics
    #[cfg(feature = "network")]
    pub fn get_sync_stats(&self) -> Option<crate::time_sync::sync_algorithm::SyncStats> {
        self.sync_algorithm.as_ref().map(|alg| alg.get_sync_stats())
    }
}

/// Time synchronization error types
#[derive(Debug, Clone, Copy)]
pub enum SyncError {
    /// Invalid message format
    InvalidMessage,
    /// Peer not found
    PeerNotFound,
    /// Synchronization disabled
    SyncDisabled,
    /// Network communication error
    NetworkError,
    /// Time correction too large
    CorrectionTooLarge,
}

/// Result type for synchronization operations
pub type SyncResult<T> = Result<T, SyncError>;
