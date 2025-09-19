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
//! // Send synchronization message
//! let message = SyncMessage::new_sync_request(node_id, 0, timestamp);
//! protocol.send_sync_request(&BROADCAST_ADDRESS, node_id, timestamp)?;
//!
//! // Receive messages
//! if let Some(received) = protocol.receive_message() {
//!     // Process received synchronization data
//! }
//! ```

use crate::time_sync::{SyncError, SyncMessage, SyncMessageType, SyncResult};
use alloc::vec::Vec;

#[cfg(all(feature = "network", feature = "esp-wifi", not(test), any(target_arch = "riscv32", target_arch = "xtensa")))]
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
    pub fn peer_exists(&self, _mac: &[u8; 6]) -> bool {
        false
    }

    pub fn send(&self, _mac: &[u8; 6], _data: &[u8]) -> Result<(), ()> {
        Ok(())
    }

    pub fn add_peer(&self, _peer: PeerInfo) -> Result<(), ()> {
        Ok(())
    }

    pub fn receive(&self) -> Option<EspNowReceive> {
        None
    }

    pub fn remove_peer(&self, _mac: &[u8; 6]) -> Result<(), ()> {
        Ok(())
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
    /// Local node identifier for this device
    local_node_id: u32,
    /// Local MAC address for ESP-NOW communication
    local_mac: [u8; 6],
}

#[cfg(feature = "network")]
impl<'a> EspNowTimeSyncProtocol<'a> {
    /// Create a new ESP-NOW time synchronization protocol handler.
    ///
    /// Initializes the protocol handler with ESP-NOW communication instance
    /// and local device information.
    ///
    /// # Arguments
    ///
    /// * `esp_now` - ESP-NOW communication instance
    /// * `local_node_id` - Unique identifier for this device
    /// * `local_mac` - MAC address of this device
    ///
    /// # Returns
    ///
    /// A new `EspNowTimeSyncProtocol` instance ready for use.
    pub fn new(esp_now: EspNow<'a>, local_node_id: u32, local_mac: [u8; 6]) -> Self {
        Self {
            esp_now,
            local_node_id,
            local_mac,
        }
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
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(SyncError)` - Communication error occurred
    pub fn send_sync_request(&mut self, target_mac: &[u8; 6], timestamp_us: u64) -> SyncResult<()> {
        let message = SyncMessage::new_sync_request(self.local_node_id, 0, timestamp_us);
        // Note: Debug info would be added here in real implementation
        self.send_message(&message, target_mac)
    }

    /// Send a time synchronization response to a specific peer.
    ///
    /// Sends a synchronization response message to the specified peer
    /// containing the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `target_mac` - MAC address of the target peer
    /// * `target_node_id` - Node ID of the target peer
    /// * `timestamp_us` - Current timestamp in microseconds
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully
    /// * `Err(SyncError)` - Communication error occurred
    pub fn send_sync_response(
        &mut self,
        target_mac: &[u8; 6],
        target_node_id: u32,
        timestamp_us: u64,
    ) -> SyncResult<()> {
        let message =
            SyncMessage::new_sync_response(self.local_node_id, target_node_id, timestamp_us);
        self.send_message(&message, target_mac)
    }

    /// Broadcast time announcement to all peers.
    ///
    /// Sends a time broadcast message to all peers in the network
    /// for synchronization purposes.
    ///
    /// # Arguments
    ///
    /// * `timestamp_us` - Current timestamp in microseconds
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Broadcast sent successfully
    /// * `Err(SyncError)` - Communication error occurred
    pub fn broadcast_time(&mut self, timestamp_us: u64) -> SyncResult<()> {
        let message = SyncMessage {
            msg_type: SyncMessageType::TimeBroadcast,
            source_node_id: self.local_node_id,
            target_node_id: 0, // Broadcast
            timestamp_us,
            sequence: 0,
            payload: Vec::new(),
        };
        self.send_message(&message, &BROADCAST_ADDRESS)
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

        // Ensure peer exists
        if !self.esp_now.peer_exists(target_mac) {
            self.add_peer(target_mac)?;
        }

        // Send the message
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
    fn add_peer(&mut self, mac_address: &[u8; 6]) -> SyncResult<()> {
        let peer_info = PeerInfo {
            peer_address: *mac_address,
            lmk: None,
            channel: None,
            encrypt: false, // TODO: Add encryption support
        };

        match self.esp_now.add_peer(peer_info) {
            Ok(_) => Ok(()),
            Err(_) => Err(SyncError::NetworkError),
        }
    }

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
                // Filter out messages from ourselves
                if message.source_node_id != self.local_node_id {
                    messages.push(message);
                }
            }
        }

        messages
    }

    /// Get the local node ID.
    ///
    /// # Returns
    ///
    /// The unique identifier of this device
    pub fn get_local_node_id(&self) -> u32 {
        self.local_node_id
    }

    /// Get the local MAC address.
    ///
    /// # Returns
    ///
    /// The MAC address of this device
    pub fn get_local_mac(&self) -> [u8; 6] {
        self.local_mac
    }

    /// Check if a peer exists in the ESP-NOW peer list.
    ///
    /// # Arguments
    ///
    /// * `mac_address` - MAC address to check
    ///
    /// # Returns
    ///
    /// * `true` - Peer exists in the list
    /// * `false` - Peer not found
    pub fn peer_exists(&self, mac_address: &[u8; 6]) -> bool {
        self.esp_now.peer_exists(mac_address)
    }

    /// Remove a peer from the ESP-NOW peer list.
    ///
    /// # Arguments
    ///
    /// * `mac_address` - MAC address of the peer to remove
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Peer removed successfully
    /// * `Err(SyncError)` - Failed to remove peer
    pub fn remove_peer(&mut self, mac_address: &[u8; 6]) -> SyncResult<()> {
        match self.esp_now.remove_peer(mac_address) {
            Ok(_) => Ok(()),
            Err(_) => Err(SyncError::NetworkError),
        }
    }

    /// Get the number of registered peers.
    ///
    /// Note: ESP-NOW doesn't provide a direct way to count peers,
    /// so this returns 0 as a placeholder.
    ///
    /// # Returns
    ///
    /// Number of registered peers (currently always 0)
    pub fn get_peer_count(&self) -> usize {
        // Note: ESP-NOW doesn't provide a direct way to count peers
        // This would need to be tracked separately if needed
        0 // Placeholder implementation
    }
}

/// Utility functions for ESP-NOW time synchronization
#[cfg(all(feature = "network", not(test)))]
pub mod utils {
    use super::*;

    /// Extract MAC address from ESP-NOW received data.
    ///
    /// # Arguments
    ///
    /// * `received` - ESP-NOW received data structure
    ///
    /// # Returns
    ///
    /// MAC address of the sender
    pub fn extract_sender_mac(received: &ReceivedData) -> [u8; 6] {
        received.info.src_address
    }

    /// Extract destination MAC address from ESP-NOW received data.
    ///
    /// # Arguments
    ///
    /// * `received` - ESP-NOW received data structure
    ///
    /// # Returns
    ///
    /// MAC address of the destination
    pub fn extract_dest_mac(received: &ReceivedData) -> [u8; 6] {
        received.info.dst_address
    }

    /// Check if a received message is a broadcast.
    ///
    /// # Arguments
    ///
    /// * `received` - ESP-NOW received data structure
    ///
    /// # Returns
    ///
    /// * `true` - Message is a broadcast
    /// * `false` - Message is unicast
    pub fn is_broadcast(received: &ReceivedData) -> bool {
        received.info.dst_address == BROADCAST_ADDRESS
    }

    /// Calculate network delay estimation based on message timestamps.
    ///
    /// Estimates the network delay by comparing message timestamps.
    ///
    /// # Arguments
    ///
    /// * `send_time` - When the message was sent
    /// * `receive_time` - When the message was received
    /// * `_remote_timestamp` - Remote timestamp (currently unused)
    ///
    /// # Returns
    ///
    /// Estimated network delay in microseconds
    pub fn estimate_network_delay(
        send_time: u64,
        receive_time: u64,
        _remote_timestamp: u64,
    ) -> u64 {
        // Simple delay estimation: half of round-trip time
        let round_trip_time = receive_time - send_time;
        round_trip_time / 2
    }

    /// Validate synchronization message integrity.
    ///
    /// Checks if a synchronization message is valid and not too old.
    ///
    /// # Arguments
    ///
    /// * `message` - Message to validate
    /// * `max_age_us` - Maximum allowed message age in microseconds
    /// * `current_time` - Current time for age calculation
    ///
    /// # Returns
    ///
    /// * `true` - Message is valid
    /// * `false` - Message is invalid or too old
    pub fn validate_message(message: &SyncMessage, max_age_us: u64, current_time: u64) -> bool {
        // Check if message is not too old
        if current_time - message.timestamp_us > max_age_us {
            return false;
        }

        // Sequence number validation is not needed for u32 type

        // Additional validation can be added here
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_serialization() {
        let message = SyncMessage::new_sync_request(123, 456, 789012345);
        let data = message.to_bytes();
        let deserialized = SyncMessage::from_bytes(&data).unwrap();

        assert_eq!(message.msg_type as u8, deserialized.msg_type as u8);
        assert_eq!(message.source_node_id, deserialized.source_node_id);
        assert_eq!(message.target_node_id, deserialized.target_node_id);
        assert_eq!(message.timestamp_us, deserialized.timestamp_us);
    }

    #[test]
    fn test_invalid_message_deserialization() {
        let invalid_data = vec![0xFF; 10]; // Invalid data
        assert!(SyncMessage::from_bytes(&invalid_data).is_none());
    }

    #[test]
    fn test_message_validation() {
        let message = SyncMessage::new_sync_request(123, 456, 1000);

        // Valid message
        assert!(utils::validate_message(&message, 10000, 5000));

        // Message too old
        assert!(!utils::validate_message(&message, 1000, 5000));
    }
}

#[cfg(any(not(feature = "network"), test))]
pub mod utils {
    use super::*;

    /// Extract MAC address from ESP-NOW received data (mock)
    pub fn extract_sender_mac(_received: &[u8]) -> [u8; 6] {
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    }

    /// Extract destination MAC address from ESP-NOW received data (mock)
    pub fn extract_dest_mac(_received: &[u8]) -> [u8; 6] {
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
    }

    /// Check if a received message is a broadcast (mock)
    pub fn is_broadcast(_received: &[u8]) -> bool {
        false
    }

    /// Calculate network delay estimation based on message timestamps (mock)
    pub fn estimate_network_delay(_send_time: u64, _receive_time: u64, _local_time: u64) -> u64 {
        0
    }

    /// Validate synchronization message (mock)
    pub fn validate_message(_message: &SyncMessage, _current_time: u64, _max_age: u64) -> bool {
        true
    }
}
