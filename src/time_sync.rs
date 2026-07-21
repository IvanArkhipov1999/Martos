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

    // --- Delay-resilient tracking (SOSP'26) parameters ---
    /// SoftMax temperature (microseconds) used by the tracking operator.
    pub softmax_temperature_us: f64,
    /// Input-correction (IC) sliding window size W (number of recent errors).
    pub ic_window: usize,
    /// IC gating gain κ in Eq. (10).
    pub ic_kappa: f64,
    /// IC gating epsilon (small positive) in Eq. (10) denominator.
    pub ic_epsilon: f64,
    /// IC gating calibration constant c in Eq. (10).
    pub ic_c: f64,
    /// Beta-consensus coupling strength λ_BC in Eq. (13)/(18).
    pub lambda_bc: f64,
    /// Dead-zone leak strength λ_L in Eq. (14)/(18).
    pub lambda_l: f64,
    /// Dead-zone half-width d in Eq. (14)/(18).
    pub beta_deadzone_d: f64,
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

            // SOSP'26-ish defaults (safe / mostly-off unless enabled)
            softmax_temperature_us: 1e6, // 1s in microseconds
            ic_window: 25,
            ic_kappa: 1.0,
            ic_epsilon: 1e-9,
            ic_c: 1.0,
            lambda_bc: 0.0, // start disabled; enable for experiments
            lambda_l: 0.0,  // start disabled; enable for experiments
            beta_deadzone_d: 0.3,
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
    /// Last accepted per-sender sequence number from this peer
    pub last_sequence: u32,
    /// Last accepted rate-correction state β from this peer (dimensionless)
    pub last_beta: f64,
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
            last_sequence: 0,
            last_beta: 0.0,
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
    /// Published logical time \tilde{x} when message was sent (microseconds)
    pub timestamp_us: u64,
    /// Per-sender sequence number for freshness under reordering/duplication
    pub sequence: u32,
    /// Node ID for debugging and identification
    pub node_id: u32,
    /// Sender's rate-correction state β (dimensionless) encoded as fixed-point
    /// beta_scaled = round(beta * BETA_SCALE)
    pub beta_scaled: i32,
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
    pub fn new_sync_request(timestamp_us: u64, node_id: u32, sequence: u32, beta: f64) -> Self {
        const BETA_SCALE: f64 = 1_000_000.0;
        let beta_scaled = (beta * BETA_SCALE) as i32;
        Self {
            msg_type: SyncMessageType::SyncRequest,
            timestamp_us,
            sequence,
            node_id,
            beta_scaled,
            payload: Vec::new(),
        }
    }

    /// Decode beta from fixed-point representation.
    pub fn beta(&self) -> f64 {
        const BETA_SCALE: f64 = 1_000_000.0;
        self.beta_scaled as f64 / BETA_SCALE
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
        // Format:
        // 1  byte  msg_type
        // 8  bytes timestamp_us (u64 LE)
        // 4  bytes sequence (u32 LE)
        // 4  bytes node_id (u32 LE)
        // 4  bytes beta_scaled (i32 LE)
        // 2  bytes payload_len (u16 LE)
        // N  bytes payload
        let mut data = Vec::with_capacity(1 + 8 + 4 + 4 + 4 + 2 + self.payload.len());

        // Message type (1 byte)
        data.push(self.msg_type as u8);

        // Timestamp (8 bytes)
        data.extend_from_slice(&self.timestamp_us.to_le_bytes());

        // Sequence number (4 bytes)
        data.extend_from_slice(&self.sequence.to_le_bytes());

        // Node ID (4 bytes)
        data.extend_from_slice(&self.node_id.to_le_bytes());

        // Beta (4 bytes)
        data.extend_from_slice(&self.beta_scaled.to_le_bytes());

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
            // Minimum message size:
            // 1 (type) + 8 (timestamp) + 4 (sequence) + 4 (node_id) + 4 (beta) + 2 (payload_len) = 23
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

        // Timestamp (u64)
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

        // Node ID
        let node_id = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        // Beta
        let beta_scaled = i32::from_le_bytes([
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
            beta_scaled,
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
    /// Map of synchronized peers (single anonymous peer in broadcast mode)
    peers: BTreeMap<u32, SyncPeer>,
    /// Current synchronization quality score (0.0-1.0 * 1000, atomic)
    sync_quality: AtomicU32,
    /// PI controller phase correction γ_i(t) in microseconds (applied to increments)
    pi_phase_correction_us: f64,
    /// PI controller rate correction β_i(t) (dimensionless, Eq. (30))
    pi_rate_correction: f64,
    /// Input-correction (IC) recent error window e_i(t) (microseconds)
    ic_error_window: Vec<i64>,
    /// Last error e_i(t-1) (microseconds) for Δe statistics
    ic_last_error: Option<i64>,
    /// Logical clock value \tilde{x}_i(t) in microseconds (incremental model)
    logical_time_us: u64,
    /// Last hardware clock sample x_i(t) used for logical-clock increments
    last_hw_for_clock_us: u64,
    /// Outgoing per-sender sequence number (incremented on each publish)
    outgoing_sequence: u32,
    /// Last hardware clock sample x_i(t) used for PI update (microseconds)
    /// Used to approximate x_i(t+1) − x_i(t) in Eq. (30).
    last_hw_time_us: u64,
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
            pi_phase_correction_us: 0.0,
            pi_rate_correction: 0.0,
            ic_error_window: Vec::new(),
            ic_last_error: None,
            logical_time_us: 0,
            last_hw_for_clock_us: 0,
            outgoing_sequence: 0,
            last_hw_time_us: 0,
            #[cfg(feature = "network")]
            esp_now_protocol: None,
            #[cfg(feature = "network")]
            sync_algorithm,
        }
    }

    fn ic_push_error(&mut self, e_us: i64) {
        if self.config.ic_window == 0 {
            return;
        }
        self.ic_error_window.push(e_us);
        if self.ic_error_window.len() > self.config.ic_window {
            // Remove oldest (small window sizes; O(W) is fine)
            self.ic_error_window.remove(0);
        }
    }

    /// Compute IC bias estimate \hat{b}_i and gating Ω_i (Eqs. (9)–(11)).
    fn ic_bias_and_gate(&self) -> (f64, f64) {
        let w = self.ic_error_window.len();
        if w < 4 {
            return (0.0, 0.0);
        }

        // Bias estimate: half-sample range estimator (Eq. (9))
        let mut sorted = self.ic_error_window.clone();
        sorted.sort_unstable();
        let half = w / 2;
        let lower = &sorted[..half];
        let upper = &sorted[half..];
        let mean_lower = lower.iter().map(|&x| x as f64).sum::<f64>() / lower.len() as f64;
        let mean_upper = upper.iter().map(|&x| x as f64).sum::<f64>() / upper.len() as f64;
        let b_hat = (mean_upper - mean_lower) / 2.0;

        // Gating (Eq. (10)): mean(e)^2 / (var(Δe) + ε) then sigmoid
        let mean_e =
            self.ic_error_window.iter().map(|&x| x as f64).sum::<f64>() / w as f64;

        // Δe(t) = e(t) - e(t-1) (Eq. (11))
        let mut deltas = Vec::with_capacity(w.saturating_sub(1));
        for k in 1..w {
            deltas.push(self.ic_error_window[k] as f64 - self.ic_error_window[k - 1] as f64);
        }
        let mean_d = deltas.iter().sum::<f64>() / deltas.len() as f64;
        let var_d = deltas
            .iter()
            .map(|d| {
                let v = d - mean_d;
                v * v
            })
            .sum::<f64>()
            / deltas.len() as f64;

        let score = (mean_e * mean_e) / (var_d + self.config.ic_epsilon) - self.config.ic_c;
        let omega = 1.0 / (1.0 + libm::exp(-self.config.ic_kappa * score));

        (b_hat, omega)
    }

    /// Allocate the next outgoing per-sender sequence number.
    pub fn next_outgoing_sequence(&mut self) -> u32 {
        let seq = self.outgoing_sequence;
        self.outgoing_sequence = self.outgoing_sequence.wrapping_add(1);
        seq
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

    /// Get PI rate correction β_i(t) from Eq. (30).
    ///
    /// Dimensionless skew correction; in [-β_max, β_max] (typically ±0.9).
    pub fn get_pi_rate_correction(&self) -> f64 {
        self.pi_rate_correction
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
        // Treat sync request as time broadcast for synchronization.
        // Use current logical time for consensus (softmax) error computation.
        let corrected_time_us = self.get_corrected_time_us();
        let time_diff_us = message.timestamp_us as i64 - corrected_time_us as i64;

        // Track per-sender freshness by node_id + sequence (ignore stale/duplicate)
        let peer_id: u32 = message.node_id;
        let msg_beta = message.beta();
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            if message.sequence <= peer.last_sequence {
                return;
            }
            peer.last_sequence = message.sequence;
            peer.last_beta = msg_beta;
            peer.last_timestamp = message.timestamp_us;
            peer.time_diff_us = time_diff_us;
            peer.sync_count += 1;

            // Update quality score based on consistency
            if time_diff_us.abs() < 1000 {
                peer.quality_score = (peer.quality_score * 0.9 + 1.0 * 0.1).min(1.0);
            } else {
                peer.quality_score = (peer.quality_score * 0.95 + 0.5 * 0.05).max(0.1);
            }
        } else {
            // Create peer slot
            let mut new_peer = SyncPeer::new([0; 6]);
            new_peer.last_sequence = message.sequence;
            new_peer.last_beta = msg_beta;
            new_peer.last_timestamp = message.timestamp_us;
            new_peer.time_diff_us = time_diff_us;
            new_peer.sync_count = 1;
            new_peer.quality_score = 0.5;
            self.peers.insert(peer_id, new_peer);
        }

        // --- Delay-resilient tracking dynamics (IC + SoftMax error) ---
        // IC uses a node-local bias estimate and gate to correct all incoming readings.
        let (b_hat, omega) = self.ic_bias_and_gate();

        // Build corrected neighbor readings \hat{y}_{ij}(t) (Eq. (12))
        let mut corrected_neighbor_times: Vec<u64> = Vec::with_capacity(self.peers.len());
        for p in self.peers.values() {
            // \hat{y} = \tilde{y} + Ω * \hat{b}
            let corr = (omega * b_hat) as i64;
            let t = (p.last_timestamp as i64 + corr).max(0) as u64;
            corrected_neighbor_times.push(t);
        }

        // Compute tracking error e_i(t) = SoftMax(\tilde{x}_i, \hat{y}) - \tilde{x}_i (Eq. (6)/(16))
        let consensus_error = self.softmax_error(corrected_time_us, &corrected_neighbor_times);

        // Push error history for IC stats
        self.ic_push_error(consensus_error);

        // Apply tracking/control update: γ, β with BC + leak (Eqs. (17)–(18))
        self.apply_pi_control(consensus_error);
    }

    /// SoftMax tracking error using log-sum-exp (max-like; monotone).
    fn softmax_error(&self, local_time_us: u64, neighbor_times_us: &[u64]) -> i64 {
        let t = self.config.softmax_temperature_us;
        if t <= 0.0 {
            return 0;
        }

        let mut c = local_time_us;
        for &nt in neighbor_times_us {
            if nt > c {
                c = nt;
            }
        }

        let c_f = c as f64;
        let mut sum_exp = 0.0_f64;

        // include local
        sum_exp += libm::exp(((local_time_us as f64) - c_f) / t);
        for &nt in neighbor_times_us {
            sum_exp += libm::exp(((nt as f64) - c_f) / t);
        }
        if sum_exp <= 0.0 || !sum_exp.is_finite() {
            return 0;
        }

        // Normalized log-sum-exp (as used in our PODC/SOSP drafts):
        //   SoftMax_T(z) = c + T * (log(sum_exp) - log(m))
        // This keeps SoftMax in [max(z) - T log(m), max(z)] and avoids
        // permanently positive error for the leading node.
        let m = (neighbor_times_us.len() + 1) as f64;
        let target = c_f + t * (libm::log(sum_exp) - libm::log(m));
        (target - local_time_us as f64) as i64
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

    /// Apply PI control law to update logical clock corrections.
    ///
    /// This implements Eqs. (29)–(31) from the PODC article:
    ///
    ///   γ_i(t + 1) = k_p · e_i(t)
    ///   β_i(t + 1) = β_i(t) + k_i · e_i(t) / (x_i(t + 1) − x_i(t)),
    ///
    /// with saturation |β_i(t)| ≤ β_max to prevent unbounded rate corrections.
    ///
    /// # Arguments
    ///
    /// * `consensus_error_us` - Consensus error e_i(t) in microseconds
    fn apply_pi_control(&mut self, consensus_error_us: i64) {
        #[cfg(all(
            feature = "network",
            any(target_arch = "riscv32", target_arch = "xtensa")
        ))]
        {
            use libm::fabs;

            // Proportional and integral gains (k_p, k_i) taken from configuration.
            let kp = self.config.acceleration_factor as f64;
            let ki = self.config.deceleration_factor as f64;

            // Eq. (29): phase correction γ_i(t + 1) = k_p · e_i(t)
            self.pi_phase_correction_us = kp * consensus_error_us as f64;

            // Apply phase correction once per control iteration (Eq. (2)):
            //   \tilde{x}_i(t+1) = \tilde{x}_i(t) + ... + γ_i(t+1),
            // with monotone commitment.
            let old = self.logical_time_us;
            let gamma_i64 = self.pi_phase_correction_us as i64;
            let candidate = (old as i64).saturating_add(gamma_i64).max(0) as u64;
            if candidate > old {
                self.logical_time_us = candidate;
            }

            // Read current hardware clock x_i(t + 1)
            let hw_time_us = time::now().duration_since_epoch().to_micros() as u64;

            // Approximate hardware increment x_i(t + 1) − x_i(t)
            if self.last_hw_time_us != 0 && hw_time_us > self.last_hw_time_us {
                let delta_hw = (hw_time_us - self.last_hw_time_us) as f64;
                if delta_hw > 0.0 && ki > 0.0 && fabs(consensus_error_us as f64) > 0.0 {
                    // Base integral update (Eq. (8))
                    let delta_beta = ki * (consensus_error_us as f64) / delta_hw;
                    self.pi_rate_correction += delta_beta;
                }
            }

            self.last_hw_time_us = hw_time_us;

            // BC: beta consensus coupling (Eq. (13)/(18))
            if self.config.lambda_bc > 0.0 && !self.peers.is_empty() {
                let mut lap = 0.0_f64;
                for p in self.peers.values() {
                    lap += self.pi_rate_correction - p.last_beta;
                }
                self.pi_rate_correction -= self.config.lambda_bc * lap;
            }

            // Dead-zone leaky integrator (Eq. (14)/(18))
            if self.config.lambda_l > 0.0 {
                let absb = fabs(self.pi_rate_correction);
                let excess = (absb - self.config.beta_deadzone_d).max(0.0);
                if excess > 0.0 {
                    self.pi_rate_correction -= self.config.lambda_l * excess * self.pi_rate_correction;
                }
            }

            // Keep a hard bound as a final safety net (still matches typical β_max usage)
            const BETA_MAX: f64 = 0.9;
            if self.pi_rate_correction > BETA_MAX {
                self.pi_rate_correction = BETA_MAX;
            } else if self.pi_rate_correction < -BETA_MAX {
                self.pi_rate_correction = -BETA_MAX;
            }

            // Update last sync time for diagnostics
            self.last_sync_time
                .store(hw_time_us as u32, Ordering::Release);
        }
        #[cfg(not(all(
            feature = "network",
            any(target_arch = "riscv32", target_arch = "xtensa")
        )))]
        {
            let _ = consensus_error_us;
        }
    }

    /// Get corrected time (real time + offset)
    /// 
    /// This method ensures that corrected time never goes backwards by tracking
    /// the last corrected time value and ensuring monotonicity. If the calculated
    /// time would be less than the last time, it returns the last time to prevent
    /// time from going backwards.
    pub fn get_corrected_time_us(&mut self) -> u64 {
        #[cfg(all(
            feature = "network",
            any(target_arch = "riscv32", target_arch = "xtensa")
        ))]
        {
            // Hardware clock reading x_i(t)
            let real_time_us = time::now().duration_since_epoch().to_micros() as u64;

            // Initialize logical clock on first call: start aligned with hardware clock.
            if self.last_hw_for_clock_us == 0 {
                self.last_hw_for_clock_us = real_time_us;
                self.logical_time_us = real_time_us;
            } else {
                // Incremental hardware advance Δx = x_i(t) − x_i(t_prev)
                let delta_hw = real_time_us.saturating_sub(self.last_hw_for_clock_us);
                self.last_hw_for_clock_us = real_time_us;

                // Incremental logical advance:
                //   Δ\tilde{x}_i(t) = (1 + β_i(t)) · Δx_i(t) + γ_i(t)
                let beta = self.pi_rate_correction;
                // IMPORTANT: γ is applied once per control iteration in apply_pi_control()
                // (Eq. (2)), not on every read/update here.
                let delta_logical_f = (1.0 + beta) * delta_hw as f64;
                let delta_logical = if delta_logical_f <= 0.0 {
                    0_u64
                } else {
                    delta_logical_f as u64
                };

                let proposed = self
                    .logical_time_us
                    .saturating_add(delta_logical);

                // Enforce monotonicity: logical time must never go backwards.
                if proposed > self.logical_time_us {
                    self.logical_time_us = proposed;
                }
            }

            // Also expose the current logical offset via time_offset_us
            // for compatibility with existing APIs:
            //   offset = \tilde{x}_i(t) − x_i(t)
            let offset_i64 = self.logical_time_us as i64 - real_time_us as i64;
            let clamped = offset_i64
                .max(i32::MIN as i64)
                .min(i32::MAX as i64) as i32;
            self.time_offset_us.store(clamped, Ordering::Release);

            self.logical_time_us
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
