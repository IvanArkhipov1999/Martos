# Martos RTOS Documentation

Welcome to the Martos RTOS documentation. This directory contains comprehensive documentation for the Martos Real-Time Operating System.

## Table of Contents

1. [Time Synchronization System](./time-synchronization.md) - Complete guide to the distributed time synchronization system
2. [Time Synchronization Diagrams](./time-sync-diagrams.md) - Visual diagrams and flowcharts for the time synchronization system

## Time Synchronization System

The Martos RTOS implements a sophisticated distributed time synchronization system using ESP-NOW communication protocol. This system enables precise time coordination across multiple ESP32 and ESP32-C6 devices in wireless networks.

### Key Features

- **Local Voting Protocol**: Implements a distributed consensus algorithm for time synchronization
- **Cross-Platform Compatibility**: Works seamlessly between ESP32 (Xtensa) and ESP32-C6 (RISC-V) platforms
- **ESP-NOW Communication**: Uses low-latency, broadcast communication for efficient synchronization
- **Monotonic Time**: Ensures time can only accelerate, never go backwards
- **Adaptive Quality**: Automatically adjusts synchronization parameters based on network conditions

### Quick Start

```rust
use martos::time_sync::{TimeSyncManager, SyncConfig, SyncPeer};

// Create configuration
let config = SyncConfig {
    node_id: 0x12345678,
    sync_interval_ms: 2000,
    max_correction_threshold_us: 100000,
    acceleration_factor: 0.8,
    deceleration_factor: 0.6,
    max_peers: 10,
    adaptive_frequency: true,
};

// Initialize sync manager
let mut sync_manager = TimeSyncManager::new(config);

// Add peers
let peer = SyncPeer::new(0x87654321, [0x24, 0x6F, 0x28, 0x12, 0x34, 0x56]);
sync_manager.add_peer(peer);

// Enable synchronization
sync_manager.enable_sync();
```

### Documentation Structure

#### [Time Synchronization System](./time-synchronization.md)
Comprehensive documentation covering:
- System architecture and components
- Communication protocol details
- Local Voting Protocol algorithm
- Message flow and synchronization cycles
- Implementation details and virtual time system
- Configuration parameters and tuning
- Performance characteristics and metrics
- Cross-platform compatibility
- Usage examples and code samples
- Troubleshooting guide

#### [Time Synchronization Diagrams](./time-sync-diagrams.md)
Visual documentation including:
- System architecture overview
- Message flow sequences
- Local Voting Protocol algorithm flow
- Virtual time system operation
- Network topology examples
- Performance metrics visualization
- Cross-platform communication flow
- Troubleshooting diagnosis flow

## Examples

The time synchronization system includes working examples for both ESP32 and ESP32-C6 platforms:

- [`xtensa-esp32/time-sync`](../examples/rust-examples/xtensa-esp32/time-sync/) - ESP32 (Xtensa) example
- [`risc-v-esp32-c6/time-sync`](../examples/rust-examples/risc-v-esp32-c6/time-sync/) - ESP32-C6 (RISC-V) example

## API Reference

The complete API documentation is available in the source code:

- [`src/time_sync.rs`](../src/time_sync.rs) - Main synchronization manager
- [`src/time_sync/sync_algorithm.rs`](../src/time_sync/sync_algorithm.rs) - Local Voting Protocol implementation
- [`src/time_sync/esp_now_protocol.rs`](../src/time_sync/esp_now_protocol.rs) - ESP-NOW communication layer

## Getting Help

If you have questions or need assistance:

1. Check the [troubleshooting section](./time-synchronization.md#troubleshooting) in the main documentation
2. Review the [example applications](../examples/rust-examples/) for usage patterns
3. Examine the [API documentation](../src/time_sync.rs) for detailed function descriptions
4. Look at the [diagrams](./time-sync-diagrams.md) for visual understanding of the system

## Contributing

When contributing to the time synchronization system:

1. Follow the existing code style and documentation patterns
2. Add comprehensive documentation for new features
3. Include examples and usage patterns
4. Update diagrams when architectural changes are made
5. Test on both ESP32 and ESP32-C6 platforms

---

*This documentation is part of the Martos RTOS project. For more information, visit the [main project repository](../README.md).*
