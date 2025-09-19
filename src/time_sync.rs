//! Time synchronization module for Martos RTOS.
//!
//! This module implements comprehensive time synchronization between nodes in a multi-agent system
//! using ESP-NOW communication protocol. The synchronization algorithm is based on
//! dynamic time acceleration/deceleration approach described in the paper
//! "Comparing time. A New Approach To The Problem Of Time Synchronization In a Multi-agent System"
//! from PROCEEDING OF THE 36TH CONFERENCE OF FRUCT ASSOCIATION.
//!
//! # Architecture Overview
//!
//! The time synchronization system consists of several key components:
//!
//! - **TimeSyncManager**: Main synchronization controller that coordinates the entire process
//! - **SyncPeer**: Represents a synchronized peer node with quality metrics
//! - **SyncMessage**: Communication protocol for time data exchange via ESP-NOW
//! - **SyncAlgorithm**: Core Local Voting Protocol implementation
//! - **EspNowTimeSyncProtocol**: ESP-NOW communication layer abstraction
//!
//! # Key Features
//!
//! - **Local Voting Protocol**: Each node votes on correct time based on peer consensus
//! - **Dynamic Time Correction**: Uses acceleration/deceleration factors for smooth convergence
//! - **Quality-based Weighting**: Peers with better sync quality have more influence
//! - **Broadcast Communication**: Efficient multi-node synchronization via ESP-NOW broadcast
//! - **Virtual Time Correction**: Provides corrected time without modifying system clock
//! - **Adaptive Synchronization**: Adjusts sync frequency based on network stability
//!
//! # Usage Example
//!
//! ```rust
//! use martos::time_sync::{TimeSyncManager, SyncConfig};
//! use esp_wifi::esp_now::EspNow;
//!
//! // Create configuration
//! let config = SyncConfig {
//!     node_id: 0x12345678,
//!     sync_interval_ms: 2000,
//!     max_correction_threshold_us: 100000,
//!     acceleration_factor: 0.8,
//!     deceleration_factor: 0.6,
//!     max_peers: 10,
//!     adaptive_frequency: true,
//! };
//!
//! // Initialize manager
//! let mut sync_manager = TimeSyncManager::new(config);
//! sync_manager.init_esp_now_protocol(esp_now_instance, local_mac);
//! sync_manager.enable_sync();
//!
//! // Get corrected time (real time + offset)
//! let corrected_time = sync_manager.get_corrected_time_us();
//! ```
//!
//! # Synchronization Algorithm
//!
//! The Local Voting Protocol works as follows:
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
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};

#[cfg(all(feature = "network", any(target_arch = "riscv32", target_arch = "xtensa")))]
use esp_hal::time;

#[cfg(feature = "network")]
pub mod esp_now_protocol;
#[cfg(feature = "network")]
pub mod sync_algorithm;

/// Configuration parameters for time synchronization system.
///
/// This structure defines all the tunable parameters that control the behavior
/// of the Local Voting Protocol synchronization algorithm.
///
/// # Parameters
///
/// - `node_id`: Unique identifier for this node in the network
/// - `sync_interval_ms`: How often to send synchronization messages (milliseconds)
/// - `max_correction_threshold_us`: Maximum time correction per cycle (microseconds)
/// - `acceleration_factor`: How aggressively to correct large time differences (0.0-1.0)
/// - `deceleration_factor`: How conservatively to correct small time differences (0.0-1.0)
/// - `max_peers`: Maximum number of peers to track simultaneously
/// - `adaptive_frequency`: Whether to adjust sync frequency based on network stability
///
/// # Example Configuration
///
/// ```rust
/// use martos::time_sync::SyncConfig;
///
/// let config = SyncConfig {
///     node_id: 0x12345678,
///     sync_interval_ms: 2000,        // Sync every 2 seconds
///     max_correction_threshold_us: 100000,  // Max 100ms correction per cycle
///     acceleration_factor: 0.8,       // Aggressive correction for large differences
///     deceleration_factor: 0.6,      // Conservative correction for small differences
///     max_peers: 10,                 // Track up to 10 peers
///     adaptive_frequency: true,      // Enable adaptive sync frequency
/// };
/// ```
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Unique node identifier for this device in the network
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
    /// Create default configuration for time synchronization.
    ///
    /// Provides sensible default values for all synchronization parameters
    /// suitable for most use cases.
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

/// Represents a synchronized peer node in the time synchronization network.
///
/// This structure tracks all relevant information about a peer node including
/// its synchronization quality, timing information, and communication history.
/// The quality score is used to weight the peer's influence in the Local Voting Protocol.
///
/// # Quality Score
///
/// The quality score (0.0 to 1.0) indicates how reliable this peer's time
/// synchronization is. Higher scores mean the peer has more influence in
/// determining the correct time. Quality is updated based on:
///
/// - Consistency of time differences
/// - Frequency of successful synchronizations
/// - Stability of communication
///
/// # Time Difference
///
/// `time_diff_us` represents the difference between this peer's time and
/// our local time in microseconds. Positive values mean the peer is ahead,
/// negative values mean the peer is behind.
#[derive(Debug, Clone)]
pub struct SyncPeer {
    /// Unique peer node identifier
    pub node_id: u32,
    /// MAC address of the peer for ESP-NOW communication
    pub mac_address: [u8; 6],
    /// Last received timestamp from this peer (microseconds)
    pub last_timestamp: u64,
    /// Time difference with this peer (microseconds, positive = peer ahead)
    pub time_diff_us: i64,
    /// Quality score for this peer (0.0 to 1.0, higher = more reliable)
    pub quality_score: f32,
    /// Number of successful synchronizations with this peer
    pub sync_count: u32,
    /// Last synchronization time (microseconds)
    pub last_sync_time: u64,
}

impl SyncPeer {
    /// Create a new peer with default values.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique identifier for the peer node
    /// * `mac_address` - MAC address for ESP-NOW communication
    ///
    /// # Returns
    ///
    /// A new `SyncPeer` instance with default quality score and zero counters.
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

/// Synchronization message structure for ESP-NOW communication.
///
/// This structure represents a time synchronization message that is exchanged
/// between nodes via ESP-NOW protocol. It contains all necessary information
/// for the Local Voting Protocol to calculate time corrections.
///
/// # Message Types
///
/// - `SyncRequest`: Request for time synchronization (typically sent as broadcast)
/// - `SyncResponse`: Response with current timestamp (peer-to-peer)
/// - `TimeBroadcast`: Broadcast time announcement (used in our implementation)
///
/// # Serialization
///
/// Messages can be serialized to/from bytes for ESP-NOW transmission using
/// `to_bytes()` and `from_bytes()` methods.
#[derive(Debug, Clone)]
pub struct SyncMessage {
    /// Type of synchronization message
    pub msg_type: SyncMessageType,
    /// Source node identifier
    pub source_node_id: u32,
    /// Target node identifier (0 for broadcast)
    pub target_node_id: u32,
    /// Timestamp when message was sent (microseconds)
    pub timestamp_us: u64,
    /// Message sequence number for ordering
    pub sequence: u32,
    /// Additional data payload (currently unused)
    pub payload: Vec<u8>,
}

impl SyncMessage {
    /// Create a new synchronization request message.
    ///
    /// # Arguments
    ///
    /// * `source_node_id` - ID of the node sending the request
    /// * `target_node_id` - ID of the target node (0 for broadcast)
    /// * `timestamp_us` - Timestamp when the message was created (microseconds)
    ///
    /// # Returns
    ///
    /// A new `SyncMessage` with `SyncRequest` type and empty payload.
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

    /// Create a new synchronization response message.
    ///
    /// # Arguments
    ///
    /// * `source_node_id` - ID of the node sending the response
    /// * `target_node_id` - ID of the target node
    /// * `timestamp_us` - Timestamp when the response was created (microseconds)
    ///
    /// # Returns
    ///
    /// A new `SyncMessage` with `SyncResponse` type and empty payload.
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

    /// Serialize message to bytes for ESP-NOW transmission.
    ///
    /// Converts the synchronization message into a byte array suitable
    /// for transmission via ESP-NOW protocol. The format includes:
    /// - Message type (1 byte)
    /// - Source node ID (4 bytes)
    /// - Target node ID (4 bytes)
    /// - Timestamp (8 bytes)
    /// - Sequence number (4 bytes)
    /// - Payload length (4 bytes)
    /// - Payload data (variable length)
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the serialized message data.
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

    /// Deserialize message from bytes received via ESP-NOW.
    ///
    /// Parses a byte array received via ESP-NOW into a `SyncMessage` structure.
    /// Returns `None` if the data is invalid or too short.
    ///
    /// # Arguments
    ///
    /// * `data` - Byte array containing the serialized message
    ///
    /// # Returns
    ///
    /// * `Some(message)` - Successfully parsed `SyncMessage`
    /// * `None` - Invalid or incomplete data
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

/// Main time synchronization manager for coordinating Local Voting Protocol.
///
/// This is the central component that manages the entire time synchronization process.
/// It coordinates between the ESP-NOW communication layer, the synchronization algorithm,
/// and peer management to achieve consensus-based time synchronization.
///
/// # Key Responsibilities
///
/// - **Peer Management**: Track and maintain information about synchronized peers
/// - **Message Handling**: Process incoming synchronization messages
/// - **Time Correction**: Apply calculated corrections to virtual time offset
/// - **Quality Assessment**: Monitor and update peer synchronization quality
/// - **Protocol Coordination**: Manage ESP-NOW communication and algorithm execution
///
/// # Virtual Time Correction
///
/// The manager maintains a virtual time offset that represents the difference
/// between real system time and synchronized network time. This allows for
/// time correction without modifying the actual system clock.
///
/// # Thread Safety
///
/// All internal state is protected by atomic operations, making the manager
/// safe for use in multi-threaded environments.
pub struct TimeSyncManager<'a> {
    /// Configuration parameters for synchronization behavior
    config: SyncConfig,
    /// Synchronization enabled flag (atomic for thread safety)
    sync_enabled: AtomicBool,
    /// Current time offset in microseconds (atomic for thread safety)
    time_offset_us: AtomicI32,
    /// Last synchronization time in microseconds (atomic for thread safety)
    last_sync_time: AtomicU32,
    /// Map of synchronized peers (node_id -> SyncPeer)
    peers: BTreeMap<u32, SyncPeer>,
    /// Current synchronization quality score (0.0-1.0 * 1000, atomic)
    sync_quality: AtomicU32,
    /// ESP-NOW protocol handler (only available with network feature)
    #[cfg(feature = "network")]
    pub esp_now_protocol: Option<crate::time_sync::esp_now_protocol::EspNowTimeSyncProtocol<'a>>,
    /// Synchronization algorithm instance (only available with network feature)
    #[cfg(feature = "network")]
    sync_algorithm: Option<crate::time_sync::sync_algorithm::SyncAlgorithm>,
}

impl<'a> TimeSyncManager<'a> {
    /// Create a new time synchronization manager.
    ///
    /// Initializes a new `TimeSyncManager` with the provided configuration.
    /// The manager starts with synchronization disabled and must be explicitly
    /// enabled using `enable_sync()`.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration parameters for synchronization behavior
    ///
    /// # Returns
    ///
    /// A new `TimeSyncManager` instance ready for initialization.
    pub fn new(config: SyncConfig) -> Self {
        #[cfg(feature = "network")]
        let sync_algorithm = Some(crate::time_sync::sync_algorithm::SyncAlgorithm::new(
            config.clone(),
        ));
        #[cfg(not(feature = "network"))]
        let sync_algorithm = None;

        Self {
            config,
            sync_enabled: AtomicBool::new(false),
            time_offset_us: AtomicI32::new(0),
            last_sync_time: AtomicU32::new(0),
            peers: BTreeMap::new(),
            sync_quality: AtomicU32::new(1000), // Start with perfect quality
            #[cfg(feature = "network")]
            esp_now_protocol: None,
            #[cfg(feature = "network")]
            sync_algorithm,
        }
    }

    /// Enable time synchronization.
    ///
    /// Starts the time synchronization process. The manager will begin
    /// processing incoming messages and applying corrections.
    pub fn enable_sync(&mut self) {
        self.sync_enabled.store(true, Ordering::Release);
    }

    /// Disable time synchronization.
    ///
    /// Stops the time synchronization process. The manager will no longer
    /// process incoming messages or apply corrections.
    pub fn disable_sync(&mut self) {
        self.sync_enabled.store(false, Ordering::Release);
    }

    /// Check if synchronization is enabled.
    ///
    /// # Returns
    ///
    /// * `true` - Synchronization is active
    /// * `false` - Synchronization is disabled
    pub fn is_sync_enabled(&self) -> bool {
        self.sync_enabled.load(Ordering::Acquire)
    }

    /// Add a peer for synchronization.
    ///
    /// Adds a new peer to the synchronization network. The peer will be
    /// included in Local Voting Protocol calculations if the maximum
    /// peer limit hasn't been reached.
    ///
    /// # Arguments
    ///
    /// * `peer` - Peer information to add
    pub fn add_peer(&mut self, peer: SyncPeer) {
        if self.peers.len() < self.config.max_peers {
            self.peers.insert(peer.node_id, peer);
        }
    }

    /// Remove a peer from synchronization.
    ///
    /// Removes a peer from the synchronization network. The peer will no
    /// longer be included in Local Voting Protocol calculations.
    ///
    /// # Arguments
    ///
    /// * `node_id` - ID of the peer to remove
    pub fn remove_peer(&mut self, node_id: u32) {
        self.peers.remove(&node_id);
    }

    /// Get current time offset in microseconds.
    ///
    /// Returns the current virtual time offset that represents the difference
    /// between real system time and synchronized network time.
    ///
    /// # Returns
    ///
    /// Current time offset in microseconds (positive = ahead, negative = behind)
    pub fn get_time_offset_us(&self) -> i32 {
        self.time_offset_us.load(Ordering::Acquire)
    }

    /// Get synchronization quality score (0.0 to 1.0).
    ///
    /// Returns the overall quality of the synchronization process based on
    /// peer consistency and stability.
    ///
    /// # Returns
    ///
    /// Quality score between 0.0 (poor) and 1.0 (excellent)
    pub fn get_sync_quality(&self) -> f32 {
        self.sync_quality.load(Ordering::Acquire) as f32 / 1000.0
    }

    /// Process one synchronization cycle.
    ///
    /// This method should be called periodically from the main application loop
    /// to perform synchronization operations. It handles peer management,
    /// quality assessment, and time correction calculations.
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

    /// Handle incoming synchronization message.
    ///
    /// Processes a synchronization message received from a peer and applies
    /// the Local Voting Protocol algorithm to calculate time corrections.
    ///
    /// # Arguments
    ///
    /// * `message` - Synchronization message to process
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
                self.handle_sync_request(message);
            }
        }
    }

    /// Handle synchronization request from a peer.
    ///
    /// Processes incoming synchronization requests and applies Local Voting Protocol
    /// corrections based on the received timestamp.
    ///
    /// # Arguments
    ///
    /// * `message` - Synchronization request message to process
    #[cfg(all(feature = "network", any(target_arch = "riscv32", target_arch = "xtensa")))]
    fn handle_sync_request(&mut self, message: SyncMessage) {
        // Treat sync request as time broadcast for synchronization
        let corrected_time_us = self.get_corrected_time_us();
        let time_diff_us = message.timestamp_us as i64 - corrected_time_us as i64;
        
        // Update peer information
        if let Some(peer) = self.peers.get_mut(&message.source_node_id) {
            peer.time_diff_us = time_diff_us;
            peer.sync_count += 1;
            
            // Update quality score based on consistency
            if time_diff_us.abs() < 1000 {
                peer.quality_score = (peer.quality_score * 0.9 + 1.0 * 0.1).min(1.0);
            } else {
                peer.quality_score = (peer.quality_score * 0.95 + 0.5 * 0.05).max(0.1);
            }
        } else {
            // Add new peer if not exists
            let mut new_peer = SyncPeer::new(message.source_node_id, [0; 6]);
            new_peer.time_diff_us = time_diff_us;
            new_peer.sync_count = 1;
            new_peer.quality_score = 0.5;
            self.peers.insert(message.source_node_id, new_peer);
        }
        
        // Use sync algorithm to calculate correction
        if let Some(ref mut algorithm) = self.sync_algorithm {
            if let Ok(correction) = algorithm.process_sync_message(message.source_node_id, message.timestamp_us, corrected_time_us) {
                // Apply correction to time offset
                self.apply_time_correction(correction as i32);
            } else {
                // esp_println::println!("Sync algorithm failed to process message");
            }
        } else {
            // esp_println::println!("Sync algorithm is None!");
        }
    }

    /// Handle synchronization request from a peer (mock implementation).
    ///
    /// Mock implementation for non-ESP targets that does nothing.
    ///
    /// # Arguments
    ///
    /// * `_message` - Synchronization request message (ignored)
    #[cfg(not(all(feature = "network", any(target_arch = "riscv32", target_arch = "xtensa"))))]
    fn handle_sync_request(&mut self, _message: SyncMessage) {
        // Mock implementation for non-ESP targets
    }

    /// Handle synchronization response from a peer.
    ///
    /// Processes incoming synchronization responses and applies Local Voting Protocol
    /// corrections based on the received timestamp.
    ///
    /// # Arguments
    ///
    /// * `message` - Synchronization response message to process
    #[cfg(all(feature = "network", any(target_arch = "riscv32", target_arch = "xtensa")))]
    fn handle_sync_response(&mut self, message: SyncMessage) {
        // Calculate time difference and update peer info
        let corrected_time_us = self.get_corrected_time_us();
        let time_diff_us = message.timestamp_us as i64 - corrected_time_us as i64;
        
        // Update peer information
        if let Some(peer) = self.peers.get_mut(&message.source_node_id) {
            peer.time_diff_us = time_diff_us;
            peer.sync_count += 1;
            
            // Update quality score based on consistency
            if time_diff_us.abs() < 1000 {
                peer.quality_score = (peer.quality_score * 0.9 + 1.0 * 0.1).min(1.0);
            } else {
                peer.quality_score = (peer.quality_score * 0.95 + 0.5 * 0.05).max(0.1);
            }
        } else {
            // Add new peer if not exists
            let mut new_peer = SyncPeer::new(message.source_node_id, [0; 6]);
            new_peer.time_diff_us = time_diff_us;
            new_peer.sync_count = 1;
            new_peer.quality_score = 0.5;
            self.peers.insert(message.source_node_id, new_peer);
        }
        
        // Use sync algorithm to calculate correction
        if let Some(ref mut algorithm) = self.sync_algorithm {
            if let Ok(correction) = algorithm.process_sync_message(message.source_node_id, message.timestamp_us, corrected_time_us) {
                // Apply correction to time offset
                self.apply_time_correction(correction as i32);
            } else {
                // esp_println::println!("Sync algorithm failed to process message");
            }
        } else {
            // esp_println::println!("Sync algorithm is None!");
        }
    }

    /// Handle synchronization response from a peer (mock implementation).
    ///
    /// Mock implementation for non-ESP targets that does nothing.
    ///
    /// # Arguments
    ///
    /// * `_message` - Synchronization response message (ignored)
    #[cfg(not(all(feature = "network", any(target_arch = "riscv32", target_arch = "xtensa"))))]
    fn handle_sync_response(&mut self, _message: SyncMessage) {
        // Mock implementation for non-ESP targets
    }

    /// Apply time correction to the system.
    ///
    /// Updates the virtual time offset based on the calculated correction.
    /// Corrections are bounded by the maximum threshold to prevent instability.
    ///
    /// # Arguments
    ///
    /// * `correction_us` - Time correction to apply in microseconds
    fn apply_time_correction(&mut self, correction_us: i32) {
        if correction_us.abs() > self.config.max_correction_threshold_us as i32 {
            return; // Skip correction if too large
        }

        // For Local Voting Protocol, we apply correction directly to offset
        // This represents how much we need to adjust our time perception
        let current_offset = self.time_offset_us.load(Ordering::Acquire);
        let new_offset = current_offset + correction_us;
        self.time_offset_us.store(new_offset, Ordering::Release);
        
        // Update last sync time
        #[cfg(all(feature = "network", any(target_arch = "riscv32", target_arch = "xtensa")))]
        {
            let current_time_us = time::now().duration_since_epoch().to_micros() as u32;
            self.last_sync_time.store(current_time_us, Ordering::Release);
        }
    }

    /// Get corrected time (real time + offset)
    pub fn get_corrected_time_us(&self) -> u64 {
        #[cfg(all(feature = "network", any(target_arch = "riscv32", target_arch = "xtensa")))]
        {
            let real_time_us = time::now().duration_since_epoch().to_micros() as u64;
            let offset_us = self.time_offset_us.load(Ordering::Acquire) as i64;
            (real_time_us as i64 + offset_us) as u64
        }
        #[cfg(not(all(feature = "network", any(target_arch = "riscv32", target_arch = "xtensa"))))]
        {
            0
        }
    }

    /// Get list of active peers.
    ///
    /// Returns a copy of all currently tracked peers in the synchronization network.
    ///
    /// # Returns
    ///
    /// Vector containing all active `SyncPeer` instances
    pub fn get_peers(&self) -> Vec<SyncPeer> {
        self.peers.values().cloned().collect()
    }

    /// Get peer by node ID.
    ///
    /// Retrieves information about a specific peer in the synchronization network.
    ///
    /// # Arguments
    ///
    /// * `node_id` - Unique identifier of the peer to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(peer)` - Reference to the peer if found
    /// * `None` - Peer not found in the network
    pub fn get_peer(&self, node_id: u32) -> Option<&SyncPeer> {
        self.peers.get(&node_id)
    }

    /// Initialize ESP-NOW protocol handler.
    ///
    /// Sets up the ESP-NOW communication layer for time synchronization.
    /// This method must be called before enabling synchronization.
    ///
    /// # Arguments
    ///
    /// * `esp_now` - ESP-NOW communication instance
    /// * `local_mac` - Local MAC address for ESP-NOW communication
    #[cfg(feature = "network")]
    pub fn init_esp_now_protocol(
        &mut self,
        esp_now: crate::time_sync::esp_now_protocol::EspNow<'static>,
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

    /// Process one synchronization cycle with ESP-NOW.
    ///
    /// Handles periodic synchronization operations including sending
    /// synchronization requests to peers.
    ///
    /// # Arguments
    ///
    /// * `current_time_us` - Current time in microseconds
    #[cfg(feature = "network")]
    pub fn process_sync_cycle_with_esp_now(&mut self, current_time_us: u32) {
        if !self.is_sync_enabled() {
            return;
        }

        // Receive and process incoming messages
        let messages = if let Some(ref mut protocol) = self.esp_now_protocol {
            protocol.receive_messages()
        } else {
            Vec::new()
        };

        for message in messages {
            self.handle_sync_message(message);
        }

        // Send periodic sync requests
        if current_time_us - self.last_sync_time.load(Ordering::Acquire)
            >= self.config.sync_interval_ms as u32 * 1000
        {
            self.send_periodic_sync_requests(current_time_us);
            self.last_sync_time
                .store(current_time_us, Ordering::Release);
        }
    }

    /// Send periodic synchronization requests to all peers.
    ///
    /// Sends synchronization requests to all tracked peers based on
    /// their quality scores and synchronization intervals.
    ///
    /// # Arguments
    ///
    /// * `current_time_us` - Current time in microseconds
    #[cfg(feature = "network")]
    fn send_periodic_sync_requests(&mut self, current_time_us: u32) {
        if let Some(ref mut protocol) = self.esp_now_protocol {
            for peer in self.peers.values() {
                if peer.quality_score > 0.1 {
                    // Only sync with good quality peers
                    let _ = protocol.send_sync_request(&peer.mac_address, current_time_us as u64);
                }
            }
        }
    }

    /// Get synchronization statistics.
    ///
    /// Returns detailed statistics about the synchronization algorithm performance
    /// including convergence metrics, peer quality, and correction history.
    ///
    /// # Returns
    ///
    /// * `Some(stats)` - Synchronization statistics if available
    /// * `None` - Statistics not available (network feature disabled)
    #[cfg(feature = "network")]
    pub fn get_sync_stats(&self) -> Option<crate::time_sync::sync_algorithm::SyncStats> {
        self.sync_algorithm.as_ref().map(|alg| alg.get_sync_stats())
    }
}

/// Time synchronization error types.
///
/// Defines the various error conditions that can occur during
/// time synchronization operations.
#[derive(Debug, Clone, Copy)]
pub enum SyncError {
    /// Invalid message format received
    InvalidMessage,
    /// Requested peer not found in network
    PeerNotFound,
    /// Synchronization is currently disabled
    SyncDisabled,
    /// Network communication error occurred
    NetworkError,
    /// Time correction exceeds maximum threshold
    CorrectionTooLarge,
}

/// Result type for synchronization operations.
///
/// Convenience type alias for `Result<T, SyncError>` used throughout
/// the time synchronization system.
pub type SyncResult<T> = Result<T, SyncError>;
