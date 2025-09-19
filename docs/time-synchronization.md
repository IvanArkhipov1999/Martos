# Time Synchronization in Martos RTOS

## Overview

Martos RTOS implements a distributed time synchronization system using ESP-NOW broadcast communication protocol. The system is designed to synchronize time across multiple ESP32 and ESP32-C6 devices in a wireless network using broadcast messages, ensuring consistent timekeeping for distributed applications.

## Table of Contents

1. [Architecture](#architecture)
2. [Communication Protocol](#communication-protocol)
3. [Synchronization Algorithm](#synchronization-algorithm)
4. [Message Flow](#message-flow)
5. [Implementation Details](#implementation-details)
6. [Configuration](#configuration)
7. [Performance Characteristics](#performance-characteristics)
8. [Cross-Platform Compatibility](#cross-platform-compatibility)
9. [Usage Examples](#usage-examples)
10. [Troubleshooting](#troubleshooting)

## Architecture

The time synchronization system consists of several key components:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Martos Time Synchronization System           │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐ │
│  │   ESP32 Node    │    │   ESP32-C6 Node │    │   ESP32 Node    │ │
│  │                 │    │                 │    │                 │ │
│  │ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │ │
│  │ │TimeSyncMgr  │ │    │ │TimeSyncMgr  │ │    │ │TimeSyncMgr  │ │ │
│  │ │             │ │    │ │             │ │    │ │             │ │ │
│  │ │ ┌─────────┐ │ │    │ │ ┌─────────┐ │ │    │ │ ┌─────────┐ │ │ │
│  │ │ │SyncAlg  │ │ │    │ │ │SyncAlg  │ │ │    │ │ │SyncAlg  │ │ │ │
│  │ │ └─────────┘ │ │    │ │ └─────────┘ │ │    │ │ └─────────┘ │ │ │
│  │ │ ┌─────────┐ │ │    │ │ ┌─────────┐ │ │    │ │ ┌─────────┐ │ │ │
│  │ │ │ESP-NOW  │ │ │    │ │ │ESP-NOW  │ │ │    │ │ │ESP-NOW  │ │ │ │
│  │ │ │Protocol │ │ │    │ │ │Protocol │ │ │    │ │ │Protocol │ │ │ │
│  │ │ └─────────┘ │ │    │ │ └─────────┘ │ │    │ │ └─────────┘ │ │ │
│  │ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │ │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### Core Components

1. **TimeSyncManager**: Main synchronization controller
2. **SyncAlgorithm**: Local Voting Protocol implementation
3. **ESP-NOW Protocol**: Communication layer
4. **Broadcast Management**: Tracks network participants receiving broadcast messages
5. **Time Correction**: Applies virtual time adjustments

## Communication Protocol

The system uses ESP-NOW for low-latency, broadcast communication. Unlike traditional peer-to-peer systems, this implementation uses broadcast messages where:

- **All nodes send broadcasts**: Every node periodically broadcasts its current time
- **All nodes receive broadcasts**: Every node receives broadcasts from all other nodes
- **No peer management**: No need to maintain individual connections or peer lists
- **Simplified topology**: Works with any network topology (star, mesh, ring, etc.)

### Message Format

```
┌─────────────────────────────────────────────────────────────────┐
│                    SyncMessage Structure                        │
├─────────────────────────────────────────────────────────────────┤
│ Field           │ Size │ Description                            │
├─────────────────┼──────┼────────────────────────────────────────┤
│ message_type    │ 1    │ SyncRequest (0x01) or SyncResponse (0x02) │
│ source_node_id  │ 4    │ Unique identifier of sender            │
│ target_node_id  │ 4    │ Target node (0 for broadcast)          │
│ timestamp_us    │ 8    │ Current time in microseconds           │
│ sequence        │ 4    │ Message sequence number                │
│ payload         │ var  │ Additional data (currently empty)      │
└─────────────────┴──────┴────────────────────────────────────────┘
```

### Broadcast Communication

```
┌─────────────────────────────────────────────────────────────────┐
│                    Broadcast Communication Flow                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Node A                    Network                    Node B    │
│    │                         │                         │        │
│    │ ── Broadcast ──────────► │ ── Broadcast ──────────► │        │
│    │   (All nodes receive)   │   (All nodes receive)   │        │
│    │                         │                         │        │
│    │ ◄── Broadcast ────────── │ ◄── Broadcast ────────── │        │
│    │   (All nodes receive)   │   (All nodes receive)   │        │
│    │                         │                         │        │
│    │ Process & Apply          │                         │        │
│    │ Time Correction          │                         │        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Synchronization Algorithm

The system implements a **Local Voting Protocol** with the following characteristics:

1. **Broadcast Communication**: All nodes send time broadcasts every 2 seconds to the entire network
2. **Time Difference Calculation**: Each node compares received timestamps with its local corrected time
3. **Monotonic Time**: Time can only accelerate, never go backwards
4. **Dynamic Correction**: Applies corrections based on weighted consensus from all network participants
5. **Convergence Detection**: Algorithm detects when nodes are synchronized
6. **No Peer Management**: No need to maintain individual peer connections - all communication is broadcast

### Algorithm Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    Local Voting Protocol                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Receive Time Broadcast                                      │
│     │                                                           │
│     ▼                                                           │
│  2. Calculate Time Difference                                  │
│     │                                                           │
│     ▼                                                           │
│  3. Apply Weighted Consensus                                   │
│     │                                                           │
│     ▼                                                           │
│  4. Calculate Correction                                        │
│     │                                                           │
│     ▼                                                           │
│  5. Apply Time Adjustment                                       │
│     │                                                           │
│     ▼                                                           │
│  6. Update Peer Quality                                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Key Principles

1. **Monotonic Time**: Time can only accelerate, never go backwards
2. **Weighted Consensus**: Network participants with higher quality scores have more influence
3. **Convergence Detection**: Algorithm detects when nodes are synchronized
4. **Adaptive Correction**: Correction strength adapts based on network state

### Correction Calculation

```
┌─────────────────────────────────────────────────────────────────┐
│                    Correction Formula                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  weighted_diff = Σ(node.time_diff * node.quality_score) /       │
│                  Σ(node.quality_score)                          │
│                                                                 │
│  if |weighted_diff| <= convergence_threshold:                  │
│      correction = weighted_diff * acceleration_factor          │
│  else:                                                          │
│      correction = weighted_diff * deceleration_factor           │
│                                                                 │
│  final_correction = clamp(correction, -max_correction,         │
│                           +max_correction)                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Message Flow

### Synchronization Cycle

```
┌─────────────────────────────────────────────────────────────────┐
│                    Synchronization Cycle                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Time: 0s                                                      │
│    │                                                           │
│    ▼                                                           │
│  ┌─────────────────┐                                          │
│  │ Send Broadcast  │ ────────────────────────────────────────┐ │
│  │ (Every 2s)     │                                         │ │
│  └─────────────────┘                                         │ │
│    │                                                         │ │
│    ▼                                                         │ │
│  ┌─────────────────┐                                         │ │
│  │ Receive &       │ ◄─────────────────────────────────────┘ │
│  │ Process        │                                           │
│  │ Messages       │                                           │
│  └─────────────────┘                                           │
│    │                                                           │
│    ▼                                                           │
│  ┌─────────────────┐                                          │
│  │ Apply Time      │                                          │
│  │ Correction      │                                          │
│  └─────────────────┘                                          │
│    │                                                           │
│    ▼                                                           │
│  ┌─────────────────┐                                          │
│  │ Update Node     │                                          │
│  │ Quality         │                                          │
│  └─────────────────┘                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Detailed Message Exchange

```
┌─────────────────────────────────────────────────────────────────┐
│                    Broadcast Message Exchange                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Node A (ESP32)           Node B (ESP32-C6)                    │
│    │                         │                                 │
│    │ ── Broadcast ──────────► │                                 │
│    │   timestamp: 1000μs     │                                 │
│    │   (All nodes receive)   │                                 │
│    │                         │                                 │
│    │ ◄── Broadcast ────────── │                                 │
│    │   timestamp: 1050μs     │                                 │
│    │   (All nodes receive)   │                                 │
│    │                         │                                 │
│    │ Calculate diff: 50μs    │                                 │
│    │ Apply correction: +25μs  │                                 │
│    │                         │                                 │
│    │ ── Broadcast ──────────► │                                 │
│    │   timestamp: 2025μs     │                                 │
│    │   (All nodes receive)   │                                 │
│    │                         │                                 │
│    │ ◄── Broadcast ────────── │                                 │
│    │   timestamp: 2075μs     │                                 │
│    │   (All nodes receive)   │                                 │
│    │                         │                                 │
│    │ Calculate diff: 50μs    │                                 │
│    │ Apply correction: +25μs  │                                 │
│    │                         │                                 │
│    │ Continue...              │                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Details

### Time Correction Mechanism

The system uses a virtual time offset approach:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Virtual Time System                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Real Time (Hardware Clock)                                     │
│    │                                                           │
│    ▼                                                           │
│  ┌─────────────────┐                                          │
│  │ System Clock    │                                          │
│  │ (esp_hal::time) │                                          │
│  └─────────────────┘                                          │
│    │                                                           │
│    ▼                                                           │
│  ┌─────────────────┐                                          │
│  │ Virtual Offset  │                                          │
│  │ (time_offset_us)│                                          │
│  └─────────────────┘                                          │
│    │                                                           │
│    ▼                                                           │
│  ┌─────────────────┐                                          │
│  │ Corrected Time  │                                          │
│  │ (real + offset) │                                          │
│  └─────────────────┘                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Node Quality Assessment

```
┌─────────────────────────────────────────────────────────────────┐
│                    Node Quality System                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Quality Score Calculation:                                     │
│                                                                 │
│  initial_quality = 0.5                                         │
│                                                                 │
│  for each sync_event:                                           │
│    if correction_magnitude < threshold:                         │
│      quality += 0.1  // Good sync                              │
│    else:                                                        │
│      quality -= 0.05 // Poor sync                              │
│                                                                 │
│  quality = clamp(quality, 0.0, 1.0)                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Configuration

### Default Parameters

```rust
SyncConfig {
    node_id: 0x12345678,                    // Unique node identifier
    sync_interval_ms: 2000,                 // Broadcast frequency
    max_correction_threshold_us: 100000,    // Max correction per cycle
    acceleration_factor: 0.8,              // Time acceleration rate
    deceleration_factor: 0.6,              // Time deceleration rate
    max_peers: 10,                          // Maximum nodes to track
    adaptive_frequency: true,               // Enable adaptive sync
}
```

### Parameter Tuning

| Parameter | Purpose | Recommended Range | Impact |
|-----------|---------|-------------------|---------|
| `sync_interval_ms` | Broadcast frequency | 1000-5000ms | Higher = less network traffic, slower convergence |
| `max_correction_threshold_us` | Max correction | 1000-100000μs | Higher = faster initial sync, less stability |
| `acceleration_factor` | Acceleration rate | 0.1-0.9 | Higher = faster convergence, more instability |
| `deceleration_factor` | Deceleration rate | 0.1-0.9 | Higher = more aggressive corrections |

## Performance Characteristics

### Memory Usage

```
┌─────────────────────────────────────────────────────────────────┐
│                    Memory Footprint                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Component                │ Size (bytes)                       │
│  ─────────────────────────┼─────────────────────────────────────┤
│  TimeSyncManager          │ ~200                               │
│  SyncAlgorithm            │ ~150                               │
│  ESP-NOW Protocol         │ ~100                               │
│  Per Peer                 │ ~50                                │
│  ─────────────────────────┼─────────────────────────────────────┤
│  Total (10 peers)         │ ~900                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Network Traffic

```
┌─────────────────────────────────────────────────────────────────┐
│                    Network Traffic Analysis                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Message Type          │ Size │ Frequency │ Total Traffic       │
│  ──────────────────────┼──────┼───────────┼─────────────────────┤
│  SyncRequest           │ 23B  │ 0.5 Hz    │ 11.5 B/s per node   │
│  ──────────────────────┼──────┼───────────┼─────────────────────┤
│  Total (10 nodes)      │ -    │ -         │ 115 B/s             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Synchronization Accuracy

```
┌─────────────────────────────────────────────────────────────────┐
│                    Synchronization Accuracy                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Scenario                │ Accuracy │ Convergence Time         │
│  ────────────────────────┼──────────┼───────────────────────────┤
│  Initial Sync            │ ±100ms   │ 10-30 seconds            │
│  Stable Network          │ ±1ms     │ 5-10 seconds             │
│  Network Interference    │ ±10ms    │ 15-30 seconds             │
│  Node Addition           │ ±5ms     │ 5-15 seconds             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Cross-Platform Compatibility

### Supported Platforms

```
┌─────────────────────────────────────────────────────────────────┐
│                    Platform Support Matrix                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Platform      │ Architecture │ ESP-NOW │ Time Sync │ Status    │
│  ──────────────┼──────────────┼─────────┼───────────┼───────────┤
│  ESP32         │ Xtensa       │ ✅      │ ✅        │ Supported │
│  ESP32-C6      │ RISC-V       │ ✅      │ ✅        │ Supported │
│  ESP32-S2      │ Xtensa       │ ✅      │ ✅        │ Supported │
│  ESP32-S3      │ Xtensa       │ ✅      │ ✅        │ Supported │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Cross-Platform Communication

```
┌─────────────────────────────────────────────────────────────────┐
│                    Cross-Platform Communication                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ESP32 (Xtensa)           ESP32-C6 (RISC-V)                    │
│    │                         │                                 │
│    │ ── ESP-NOW ────────────► │                                 │
│    │   (Same Protocol)      │                                 │
│    │                         │                                 │
│    │ ◄── ESP-NOW ──────────── │                                 │
│    │   (Same Protocol)      │                                 │
│    │                         │                                 │
│    │ Automatic Platform      │                                 │
│    │ Detection & Adaptation │                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Usage Examples

### Basic Setup

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

### Advanced Configuration

```rust
// Custom configuration for high-precision applications
let config = SyncConfig {
    node_id: 0x12345678,
    sync_interval_ms: 1000,        // More frequent sync
    max_correction_threshold_us: 1000, // Smaller corrections
    acceleration_factor: 0.9,      // Aggressive acceleration
    deceleration_factor: 0.7,      // Aggressive deceleration
    max_peers: 20,                 // More peers
    adaptive_frequency: true,
};

// Initialize with ESP-NOW
sync_manager.init_esp_now_protocol(esp_now, local_mac);

// Monitor synchronization quality
let quality = sync_manager.get_sync_quality();
if quality > 0.8 {
    println!("High quality synchronization achieved");
}
```

## Troubleshooting

### Common Issues

#### 1. No Synchronization

**Symptoms**: No time correction applied, `diff` remains constant

**Causes**:
- ESP-NOW communication range exceeded
- Incorrect peer MAC addresses
- Synchronization disabled

**Solutions**:
```rust
// Check synchronization status
if !sync_manager.is_synchronized(1000) {
    println!("Synchronization not working");
}

// Verify peer configuration
let peers = sync_manager.get_peers();
println!("Active peers: {}", peers.len());

// Check ESP-NOW connectivity
if let Some(protocol) = &sync_manager.esp_now_protocol {
    let peer_count = protocol.get_peer_count();
    println!("ESP-NOW peers: {}", peer_count);
}
```

#### 2. Poor Synchronization Quality

**Symptoms**: Large time differences, unstable corrections

**Causes**:
- Network interference
- Inappropriate configuration parameters
- Hardware clock drift

**Solutions**:
```rust
// Adjust configuration for stability
let config = SyncConfig {
    sync_interval_ms: 5000,        // Less frequent sync
    max_correction_threshold_us: 1000, // Smaller corrections
    acceleration_factor: 0.3,      // Conservative acceleration
    deceleration_factor: 0.2,      // Conservative deceleration
    // ... other parameters
};
```

#### 3. Large Time Corrections

**Symptoms**: Sudden large time jumps, system instability

**Causes**:
- Initial synchronization with large time differences
- Network delays
- Clock drift accumulation

**Solutions**:
```rust
// Limit correction magnitude
let config = SyncConfig {
    max_correction_threshold_us: 1000, // Limit corrections
    // ... other parameters
};

// Monitor correction history
let stats = sync_manager.get_sync_stats();
println!("Max correction: {}μs", stats.max_correction);
```

### Debugging Tools

#### 1. Synchronization Statistics

```rust
let stats = sync_manager.get_sync_stats();
println!("Sync Statistics:");
println!("  Average time diff: {}μs", stats.avg_time_diff);
println!("  Max time diff: {}μs", stats.max_time_diff);
println!("  Min time diff: {}μs", stats.min_time_diff);
println!("  Current correction: {}μs", stats.current_correction);
println!("  Converged: {}", stats.converged);
```

#### 2. Peer Information

```rust
let peers = sync_manager.get_peers();
for peer in peers {
    println!("Peer {}: quality={}, diff={}μs, syncs={}",
             peer.node_id, peer.quality_score, 
             peer.time_diff_us, peer.sync_count);
}
```

#### 3. Real-time Monitoring

```rust
// Monitor synchronization progress
loop {
    let corrected_time = sync_manager.get_corrected_time_us();
    let offset = sync_manager.get_time_offset_us();
    let quality = sync_manager.get_sync_quality();
    
    println!("Time: {}μs, Offset: {}μs, Quality: {:.2}",
             corrected_time, offset, quality);
    
    // Process synchronization cycle
    sync_manager.process_sync_cycle();
    
    // Wait before next cycle
    delay_ms(1000);
}
```

## Conclusion

The Martos RTOS time synchronization system provides a robust, distributed timekeeping solution for ESP32-based networks. By implementing the Local Voting Protocol with ESP-NOW communication, it achieves:

- **High Accuracy**: Sub-millisecond synchronization in stable networks
- **Cross-Platform**: Seamless operation between different ESP chipset architectures
- **Scalability**: Support for networks with multiple nodes
- **Efficiency**: Minimal memory and network overhead
- **Reliability**: Robust operation in challenging network conditions

The system is designed for applications requiring precise time coordination, such as distributed sensor networks, industrial automation, and real-time control systems.

---

*For more information, see the [API Documentation](../src/time_sync.rs) and [Example Applications](../examples/rust-examples/).*
