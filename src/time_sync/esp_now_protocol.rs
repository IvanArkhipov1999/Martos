//! ESP-NOW protocol implementation for time synchronization.
//!
//! This module provides the communication layer for time synchronization
//! using ESP-NOW protocol. It handles message serialization, transmission,
//! and reception of synchronization data between network nodes.
//!
//! # Overview
//!
//! The ESP-NOW protocol layer abstracts the low-level ESP-NOW communication
//! and provides a high-level interface for time synchronization messages.
//! It handles both real ESP-NOW communication on ESP32/ESP32-C6 targets and
//! mock implementations for testing on other platforms.
//!
//! # Key Features
//!
//! - **Cross-platform Support**: Works on ESP32/ESP32-C6 and provides mocks for testing
//! - **Message Serialization**: Converts SyncMessage structures to/from byte arrays
//! - **Broadcast Communication**: Efficient multi-node synchronization via ESP-NOW broadcast
//! - **Peer Management**: Handles ESP-NOW peer addition and management
//! - **Error Handling**: Comprehensive error handling for communication failures
//!
//! # Conditional Compilation
//!
//! The module uses conditional compilation to provide different implementations:
//!
//! - **ESP Targets**: Uses real `esp-wifi` ESP-NOW implementation
//! - **Test/Other Targets**: Provides mock implementations for testing
//!
//! # Usage Example
//!
//! ```rust
//! use martos::time_sync::esp_now_protocol::EspNowTimeSyncProtocol;
//! use esp_wifi::esp_now::EspNow;
//!
//! // Initialize protocol with ESP-NOW instance
//! let mut protocol = EspNowTimeSyncProtocol::new(esp_now_instance);
//!
//! // Send synchronization message (broadcast)
//! let message = SyncMessage::new_sync_request(timestamp, node_id, seq, beta);
//! protocol.send_sync_request(&BROADCAST_ADDRESS, timestamp, node_id)?;
//!
//! // Receive messages
//! if let Some(received) = protocol.receive_message() {
//!     // Process received synchronization data
//! }
//! ```

use crate::time_sync::{SyncError, SyncMessage, SyncResult};
use alloc::vec::Vec;

#[cfg(all(
    feature = "network",
    feature = "esp-wifi",
    not(test),
    any(target_arch = "riscv32", target_arch = "xtensa")
))]
pub use esp_wifi::esp_now::{EspNow, EspNowReceiver, PeerInfo, ReceivedData, BROADCAST_ADDRESS};

#[cfg(any(not(feature = "network"), not(feature = "esp-wifi"), test))]
pub struct EspNow {}
#[cfg(any(not(feature = "network"), not(feature = "esp-wifi"), test))]
pub struct PeerInfo {
    pub peer_address: [u8; 6],
    pub lmk: Option<[u8; 16]>,
    pub channel: Option<u8>,
    pub encrypt: bool,
}
#[cfg(any(not(feature = "network"), not(feature = "esp-wifi"), test))]
pub const BROADCAST_ADDRESS: [u8; 6] = [0xFF; 6];
#[cfg(any(not(feature = "network"), not(feature = "esp-wifi"), test))]
pub struct EspNowReceive {
    pub data: Vec<u8>,
}
#[cfg(any(not(feature = "network"), not(feature = "esp-wifi"), test))]
pub struct EspNowReceiveInfo {
    pub src_address: [u8; 6],
    pub dst_address: [u8; 6],
}
#[cfg(any(not(feature = "network"), not(feature = "esp-wifi"), test))]
pub struct EspNowReceiver {}
#[cfg(any(not(feature = "network"), not(feature = "esp-wifi"), test))]
pub struct ReceivedData {
    pub info: EspNowReceiveInfo,
    pub data: Vec<u8>,
}

#[cfg(any(not(feature = "network"), not(feature = "esp-wifi"), test))]
impl EspNow {
    pub fn send(&self, _mac: &[u8; 6], _data: &[u8]) -> Result<(), ()> {
        Ok(())
    }
    pub fn receive(&self) -> Option<EspNowReceive> {
        None
    }
}

#[cfg(not(feature = "network"))]
pub struct EspNowReceiveInfo {
    pub src_address: [u8; 6],
    pub dst_address: [u8; 6],
}

/// ESP-NOW protocol handler for time synchronization communication.
///
/// This structure wraps the ESP-NOW communication layer and provides
/// high-level methods for sending and receiving time synchronization
/// messages. It handles message serialization, peer management, and
/// error handling for ESP-NOW communication.
///
/// # Key Responsibilities
///
/// - **Message Transmission**: Send synchronization messages via ESP-NOW
/// - **Message Reception**: Receive and deserialize synchronization messages
/// - **Peer Management**: Handle ESP-NOW peer addition and management
/// - **Error Handling**: Provide robust error handling for communication failures
/// - **Broadcast Support**: Efficient multi-node communication via broadcast
///
/// # Thread Safety
///
/// The protocol handler is designed for single-threaded use and maintains
/// internal state for peer management and message handling.
#[cfg(feature = "network")]
pub struct EspNowTimeSyncProtocol<'a> {
    /// ESP-NOW communication instance
    pub esp_now: EspNow<'a>,
}

#[cfg(feature = "network")]
impl<'a> EspNowTimeSyncProtocol<'a> {
    /// Create a new ESP-NOW time synchronization protocol handler.
    ///
    /// Initializes the protocol handler with ESP-NOW communication instance.
    ///
    /// # Arguments
    ///
    /// * `esp_now` - ESP-NOW communication instance
    ///
    /// # Returns
    ///
    /// A new `EspNowTimeSyncProtocol` instance ready for use.
    pub fn new(esp_now: EspNow<'a>) -> Self {
        Self { esp_now }
    }

    /// Send a time synchronization request to a specific peer.
    ///
    /// Sends a synchronization request message to the specified peer
    /// containing the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `target_mac` - MAC address of the target peer
    /// * `timestamp_us` - Current timestamp in microseconds
    /// * `node_id` - Node ID for debugging and identification
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(SyncError)` - Communication error occurred
    pub fn send_sync_request(&mut self, target_mac: &[u8; 6], timestamp_us: u64, node_id: u32) -> SyncResult<()> {
        let message = SyncMessage::new_sync_request(timestamp_us, node_id, 0, 0.0);
        // Note: Debug info would be added here in real implementation
        self.send_message(&message, target_mac)
    }

    /// Send a synchronization message to a specific MAC address.
    ///
    /// Serializes the message and sends it via ESP-NOW to the specified target.
    ///
    /// # Arguments
    ///
    /// * `message` - Synchronization message to send
    /// * `target_mac` - MAC address of the target device
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(SyncError)` - Communication error occurred
    fn send_message(&mut self, message: &SyncMessage, target_mac: &[u8; 6]) -> SyncResult<()> {
        let data = message.to_bytes();
        match self.esp_now.send(target_mac, &data) {
            Ok(_) => Ok(()),
            Err(_) => Err(SyncError::NetworkError),
        }
    }

    /// Add a peer to the ESP-NOW peer list.
    ///
    /// Registers a new peer with ESP-NOW for communication.
    ///
    /// # Arguments
    ///
    /// * `mac_address` - MAC address of the peer to add
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Peer added successfully
    /// * `Err(SyncError)` - Failed to add peer
    // Broadcast-only mode: peer management not required

    /// Receive and process incoming synchronization messages.
    ///
    /// Polls for incoming ESP-NOW messages and converts them to
    /// `SyncMessage` structures for processing.
    ///
    /// # Returns
    ///
    /// Vector of received synchronization messages
    pub fn receive_messages(&mut self) -> Vec<SyncMessage> {
        let mut messages = Vec::new();

        // Process all available messages
        while let Some(received) = self.esp_now.receive() {
            if let Some(message) = SyncMessage::from_bytes(&received.data) {
                messages.push(message);
            }
        }

        messages
    }

    // Broadcast-only: peer existence/removal/count APIs removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_serialization() {
        let message = SyncMessage::new_sync_request(789012345, 42, 7, 0.123);
        let data = message.to_bytes();
        let deserialized = SyncMessage::from_bytes(&data).unwrap();

        assert_eq!(message.msg_type as u8, deserialized.msg_type as u8);
        assert_eq!(message.timestamp_us, deserialized.timestamp_us);
        assert_eq!(message.sequence, deserialized.sequence);
    }

    #[test]
    fn test_invalid_message_deserialization() {
        let invalid_data = vec![0xFF; 10]; // Invalid data
        assert!(SyncMessage::from_bytes(&invalid_data).is_none());
    }
}
