//! ESP-NOW protocol implementation for time synchronization.
//!
//! This module provides the communication layer for time synchronization
//! using ESP-NOW protocol. It handles message serialization, transmission,
//! and reception of synchronization data between network nodes.

use crate::time_sync::{SyncMessage, SyncMessageType, SyncError, SyncResult};
use esp_wifi::esp_now::{EspNow, PeerInfo, BROADCAST_ADDRESS};

/// ESP-NOW protocol handler for time synchronization
pub struct EspNowTimeSyncProtocol {
    esp_now: EspNow<'static>,
    local_node_id: u32,
    local_mac: [u8; 6],
}

impl EspNowTimeSyncProtocol {
    /// Create a new ESP-NOW time synchronization protocol handler
    pub fn new(esp_now: EspNow<'static>, local_node_id: u32, local_mac: [u8; 6]) -> Self {
        Self {
            esp_now,
            local_node_id,
            local_mac,
        }
    }

    /// Send a time synchronization request to a specific peer
    pub fn send_sync_request(&mut self, target_mac: &[u8; 6], timestamp_us: u64) -> SyncResult<()> {
        let message = SyncMessage::new_sync_request(self.local_node_id, 0, timestamp_us);
        self.send_message(&message, target_mac)
    }

    /// Send a time synchronization response to a specific peer
    pub fn send_sync_response(&mut self, target_mac: &[u8; 6], target_node_id: u32, timestamp_us: u64) -> SyncResult<()> {
        let message = SyncMessage::new_sync_response(self.local_node_id, target_node_id, timestamp_us);
        self.send_message(&message, target_mac)
    }

    /// Broadcast time announcement to all peers
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

    /// Send a synchronization message to a specific MAC address
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

    /// Add a peer to the ESP-NOW peer list
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

    /// Receive and process incoming synchronization messages
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

    /// Get the local node ID
    pub fn get_local_node_id(&self) -> u32 {
        self.local_node_id
    }

    /// Get the local MAC address
    pub fn get_local_mac(&self) -> [u8; 6] {
        self.local_mac
    }

    /// Check if a peer exists in the ESP-NOW peer list
    pub fn peer_exists(&self, mac_address: &[u8; 6]) -> bool {
        self.esp_now.peer_exists(mac_address)
    }

    /// Remove a peer from the ESP-NOW peer list
    pub fn remove_peer(&mut self, mac_address: &[u8; 6]) -> SyncResult<()> {
        match self.esp_now.remove_peer(mac_address) {
            Ok(_) => Ok(()),
            Err(_) => Err(SyncError::NetworkError),
        }
    }

    /// Get the number of registered peers
    pub fn get_peer_count(&self) -> usize {
        // Note: ESP-NOW doesn't provide a direct way to count peers
        // This would need to be tracked separately if needed
        0 // Placeholder implementation
    }
}

/// Utility functions for ESP-NOW time synchronization
pub mod utils {
    use super::*;

    /// Extract MAC address from ESP-NOW received data
    pub fn extract_sender_mac(received: &esp_wifi::esp_now::EspNowReceive) -> [u8; 6] {
        received.info.src_address
    }

    /// Extract destination MAC address from ESP-NOW received data
    pub fn extract_dest_mac(received: &esp_wifi::esp_now::EspNowReceive) -> [u8; 6] {
        received.info.dst_address
    }

    /// Check if a received message is a broadcast
    pub fn is_broadcast(received: &esp_wifi::esp_now::EspNowReceive) -> bool {
        received.info.dst_address == BROADCAST_ADDRESS
    }

    /// Calculate network delay estimation based on message timestamps
    pub fn estimate_network_delay(send_time: u64, receive_time: u64, remote_timestamp: u64) -> u64 {
        // Simple delay estimation: half of round-trip time
        let round_trip_time = receive_time - send_time;
        round_trip_time / 2
    }

    /// Validate synchronization message integrity
    pub fn validate_message(message: &SyncMessage, max_age_us: u64, current_time: u64) -> bool {
        // Check if message is not too old
        if current_time - message.timestamp_us > max_age_us {
            return false;
        }

        // Check if sequence number is reasonable (basic validation)
        if message.sequence > 0xFFFF_FFFF {
            return false;
        }

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
