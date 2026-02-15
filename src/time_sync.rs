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

#[cfg(all(
    feature = "network",
    any(target_arch = "riscv32", target_arch = "xtensa")
))]
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
/// - `sync_interval_ms`: How often to send synchronization messages (milliseconds, default: 500ms)
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
///     sync_interval_ms: 500,         // Sync every 500ms
///     max_correction_threshold_us: 100000,  // Max 100ms correction per cycle
///     acceleration_factor: 0.8,       // Aggressive correction for large differences
///     deceleration_factor: 0.6,      // Conservative correction for small differences
///     max_peers: 10,                 // Track up to 10 peers
///     adaptive_frequency: true,      // Enable adaptive sync frequency
/// };
/// ```
#[derive(Debug, Clone)]
pub struct SyncConfig {
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
            sync_interval_ms: 500,             // 500ms
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
    /// MAC address of the peer for ESP-NOW communication (optional in broadcast mode)
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
    /// * `mac_address` - MAC address for ESP-NOW communication
    ///
    /// # Returns
    ///
    /// A new `SyncPeer` instance with default quality score and zero counters.
    pub fn new(mac_address: [u8; 6]) -> Self {
        Self {
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
/// `to_bytes()` and `from_bytes()` methods. On the wire, the timestamp is
/// sent as softmax: exp(timestamp_us / T) as f64; when receiving, timestamp_us
/// is recovered as T * ln(value).
#[derive(Debug, Clone)]
pub struct SyncMessage {
    /// Type of synchronization message
    pub msg_type: SyncMessageType,
    /// Timestamp when message was sent (microseconds); when sending, this is encoded as softmax in `to_bytes()`
    pub timestamp_us: u64,
    /// Message sequence number for ordering
    pub sequence: u32,
    /// Node ID for debugging and identification
    pub node_id: u32,
    /// Additional data payload (currently unused)
    pub payload: Vec<u8>,
}

impl SyncMessage {
    /// Create a new synchronization request message.
    ///
    /// # Arguments
    ///
    /// * `timestamp_us` - Timestamp when the message was created (microseconds)
    /// * `node_id` - Node ID for debugging and identification
    ///
    /// # Returns
    ///
    /// A new `SyncMessage` with `SyncRequest` type and empty payload.
    pub fn new_sync_request(timestamp_us: u64, node_id: u32) -> Self {
        Self {
            msg_type: SyncMessageType::SyncRequest,
            timestamp_us,
            sequence: 0,
            node_id,
            payload: Vec::new(),
        }
    }

    /// Serialize message to bytes for ESP-NOW transmission.
    ///
    /// Converts the synchronization message into a byte array suitable
    /// for transmission via ESP-NOW protocol. The format includes:
    /// - Message type (1 byte)
    /// - Timestamp (8 bytes)
    /// - Sequence number (4 bytes)
    /// - Node ID (4 bytes)
    /// - Payload length (2 bytes)
/// - Payload data (variable length)
///
/// The timestamp is sent as softmax representation: exp(timestamp_us / SOFTMAX_TEMPERATURE_US).
///
/// # Returns
///
/// A `Vec<u8>` containing the serialized message data.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(24);

        // Message type (1 byte)
        data.push(self.msg_type as u8);

        // Softmax of timestamp: exp(timestamp_us / T) as f64 (8 bytes)
        const SOFTMAX_TEMPERATURE_US: f64 = 1e14;
        let softmax_val = libm::exp(self.timestamp_us as f64 / SOFTMAX_TEMPERATURE_US);
        data.extend_from_slice(&softmax_val.to_le_bytes());

        // Sequence number (4 bytes)
        data.extend_from_slice(&self.sequence.to_le_bytes());

        // Node ID (4 bytes)
        data.extend_from_slice(&self.node_id.to_le_bytes());

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
        if data.len() < 19 {
            // Minimum message size: 1 (type) + 8 (timestamp) + 4 (sequence) + 4 (node_id) + 2 (payload_len) = 19
            return None;
        }

        let mut offset = 0;

        // Message type
        let msg_type = match data[offset] {
            0x01 => SyncMessageType::SyncRequest,
            0x03 => SyncMessageType::TimeBroadcast,
            _ => return None,
        };
        offset += 1;

        // Softmax of timestamp (f64): recover timestamp_us = T * ln(value)
        const SOFTMAX_TEMPERATURE_US: f64 = 1e14;
        let softmax_val = f64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]);
        let timestamp_us = if softmax_val > 1e-300_f64 && softmax_val.is_finite() {
            (SOFTMAX_TEMPERATURE_US * libm::log(softmax_val)) as u64
        } else {
            0
        };
        offset += 8;

        // Sequence number
        let sequence = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Node ID
        let node_id = u32::from_le_bytes([
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
            timestamp_us,
            sequence,
            node_id,
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
    /// Last corrected time to prevent time from going backwards (atomic for thread safety)
    /// Stored as two u32 values (high and low) since AtomicU64 is not available on 32-bit platforms
    last_corrected_time_us_high: AtomicU32,
    last_corrected_time_us_low: AtomicU32,
    /// Map of synchronized peers (single anonymous peer in broadcast mode)
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
            last_corrected_time_us_high: AtomicU32::new(0),
            last_corrected_time_us_low: AtomicU32::new(0),
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

    // Broadcast-only: peer management API removed

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
    #[cfg(all(
        feature = "network",
        any(target_arch = "riscv32", target_arch = "xtensa")
    ))]
    fn handle_sync_request(&mut self, message: SyncMessage) {
        // Treat sync request as time broadcast for synchronization
        let corrected_time_us = self.get_corrected_time_us();
        let time_diff_us = message.timestamp_us as i64 - corrected_time_us as i64;

        // Use single anonymous peer (broadcast-only mode)
        let anon_peer_id: u32 = 0;
        if let Some(peer) = self.peers.get_mut(&anon_peer_id) {
            peer.time_diff_us = time_diff_us;
            peer.sync_count += 1;

            // Update quality score based on consistency
            if time_diff_us.abs() < 1000 {
                peer.quality_score = (peer.quality_score * 0.9 + 1.0 * 0.1).min(1.0);
            } else {
                peer.quality_score = (peer.quality_score * 0.95 + 0.5 * 0.05).max(0.1);
            }
        } else {
            // Create anonymous peer if not exists
            let mut new_peer = SyncPeer::new([0; 6]);
            new_peer.time_diff_us = time_diff_us;
            new_peer.sync_count = 1;
            new_peer.quality_score = 0.5;
            self.peers.insert(anon_peer_id, new_peer);
        }

        // Use sync algorithm to calculate correction
        if let Some(ref mut algorithm) = self.sync_algorithm {
            if let Ok(correction) = algorithm.process_sync_message(
                anon_peer_id,
                message.timestamp_us,
                corrected_time_us,
            ) {
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
    #[cfg(not(all(
        feature = "network",
        any(target_arch = "riscv32", target_arch = "xtensa")
    )))]
    fn handle_sync_request(&mut self, _message: SyncMessage) {
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
        #[cfg(all(
            feature = "network",
            any(target_arch = "riscv32", target_arch = "xtensa")
        ))]
        {
            let current_time_us = time::now().duration_since_epoch().to_micros() as u32;
            self.last_sync_time
                .store(current_time_us, Ordering::Release);
        }
    }

    /// Get corrected time (real time + offset)
    /// 
    /// This method ensures that corrected time never goes backwards by tracking
    /// the last corrected time value and ensuring monotonicity. If the calculated
    /// time would be less than the last time, it returns the last time to prevent
    /// time from going backwards.
    pub fn get_corrected_time_us(&self) -> u64 {
        #[cfg(all(
            feature = "network",
            any(target_arch = "riscv32", target_arch = "xtensa")
        ))]
        {
            let real_time_us = time::now().duration_since_epoch().to_micros() as u64;
            let offset_us = self.time_offset_us.load(Ordering::Acquire) as i64;
            let calculated_time = (real_time_us as i64 + offset_us) as u64;
            
            // Ensure time never goes backwards - read last time from two u32 values
            // We need to read both values in a way that ensures consistency
            loop {
                // Read high word first, then low word, then verify high word hasn't changed
                let last_high1 = self.last_corrected_time_us_high.load(Ordering::Acquire);
                let last_low = self.last_corrected_time_us_low.load(Ordering::Acquire);
                let last_high2 = self.last_corrected_time_us_high.load(Ordering::Acquire);
                
                // If high word changed during read, retry
                if last_high1 != last_high2 {
                    continue;
                }
                
                // Reconstruct 64-bit value
                let last_time = ((last_high1 as u64) << 32) | (last_low as u64);
                
                let corrected_time = if calculated_time < last_time {
                    // Time would go backwards - use last time to maintain monotonicity
                    last_time
                } else {
                    // Time is moving forward - use calculated time
                    calculated_time
                };
                
                // Split into high and low words
                let corrected_high = (corrected_time >> 32) as u32;
                let corrected_low = corrected_time as u32;
                
                // Try to update atomically - update high word first, then low word
                // If high word changed, retry
                match self.last_corrected_time_us_high.compare_exchange_weak(
                    last_high1,
                    corrected_high,
                    Ordering::Release,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        // High word updated successfully, now update low word
                        self.last_corrected_time_us_low.store(corrected_low, Ordering::Release);
                        return corrected_time;
                    }
                    Err(_) => continue, // Retry if value changed
                }
            }
        }
        #[cfg(not(all(
            feature = "network",
            any(target_arch = "riscv32", target_arch = "xtensa")
        )))]
        {
            0
        }
    }

    // Broadcast-only: peer lookup API removed

    /// Initialize ESP-NOW protocol handler.
    ///
    /// Sets up the ESP-NOW communication layer for time synchronization.
    /// This method must be called before enabling synchronization.
    ///
    /// # Arguments
    ///
    /// * `esp_now` - ESP-NOW communication instance
    #[cfg(feature = "network")]
    pub fn init_esp_now_protocol(
        &mut self,
        esp_now: crate::time_sync::esp_now_protocol::EspNow<'static>,
    ) {
        self.esp_now_protocol =
            Some(crate::time_sync::esp_now_protocol::EspNowTimeSyncProtocol::new(esp_now));
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
