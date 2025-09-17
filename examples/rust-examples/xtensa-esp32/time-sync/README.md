# ESP32 Time Synchronization Example

This example demonstrates the time synchronization system implemented in Martos RTOS using ESP-NOW communication protocol.

## Features

- **Time Synchronization**: Synchronizes time across multiple ESP32 nodes
- **ESP-NOW Communication**: Uses ESP-NOW for low-latency peer-to-peer communication
- **Dynamic Algorithm**: Implements dynamic time acceleration/deceleration algorithm
- **Quality Monitoring**: Tracks synchronization quality and peer performance
- **Statistics**: Provides detailed synchronization statistics

## Architecture

The example consists of several key components:

1. **TimeSyncManager**: Main synchronization controller
2. **ESP-NOW Protocol**: Communication layer for time data exchange
3. **Sync Algorithm**: Core synchronization algorithm with dynamic correction
4. **Timer Integration**: Integration with Martos timer system

## Configuration

The synchronization system is configured with the following parameters:

- **Node ID**: `0x12345678` (unique identifier for this node)
- **Sync Interval**: 2000ms (synchronization frequency)
- **Max Correction**: 1000μs (maximum time correction per cycle)
- **Acceleration Factor**: 0.1 (rate of time acceleration)
- **Deceleration Factor**: 0.05 (rate of time deceleration)
- **Max Peers**: 10 (maximum number of synchronized peers)

## Usage

### Building and Flashing

```bash
# Build the example
cargo build --release --example time-sync

# Flash to ESP32 (adjust port as needed)
espflash flash --release --example time-sync /dev/ttyUSB0
```

### Running Multiple Nodes

To test synchronization between multiple nodes:

1. Flash the example to multiple ESP32 devices
2. Ensure devices are within ESP-NOW communication range
3. Monitor serial output to see synchronization statistics

### Expected Output

```
=== ESP32 Time Synchronization Example ===
Time synchronization setup complete!
Node ID: 0x12345678
Sync interval: 2000ms
Max correction: 1000μs

Tick: 1000, Local: 1.000s, Sync: 1.001s, Offset: 1000μs
Tick: 2000, Local: 2.000s, Sync: 2.001s, Offset: 1000μs

=== Synchronization Statistics ===
Sync enabled: true
Sync quality: 0.85
Time offset: 500μs
Active peers: 2
  Peer 0x11111111: quality=0.90, diff=200μs, syncs=5
  Peer 0x22222222: quality=0.80, diff=-300μs, syncs=3
Algorithm stats:
  Avg time diff: 50.0μs
  Max time diff: 200μs
  Min time diff: -300μs
  Current correction: 100μs
  Converged: true
=====================================
```

## Synchronization Algorithm

The example implements a dynamic time synchronization algorithm based on:

1. **Time Difference Calculation**: Compares local and remote timestamps
2. **Weighted Averaging**: Uses peer quality scores for weighted time difference calculation
3. **Dynamic Correction**: Applies acceleration/deceleration based on convergence state
4. **Quality Tracking**: Monitors peer performance and adjusts synchronization accordingly

## Customization

### Adding More Peers

```rust
let peer3 = SyncPeer::new(0x33333333, [0x33, 0x33, 0x33, 0x33, 0x33, 0x33]);
sync_manager.add_peer(peer3);
```

### Adjusting Synchronization Parameters

```rust
let sync_config = SyncConfig {
    sync_interval_ms: 1000,        // More frequent sync
    max_correction_threshold_us: 500, // Smaller corrections
    acceleration_factor: 0.2,       // Faster convergence
    // ... other parameters
};
```

### Monitoring Synchronization Quality

```rust
if sync_manager.is_synchronized(100) { // Within 100μs tolerance
    println!("Time is well synchronized");
} else {
    println!("Time synchronization needs improvement");
}
```

## Troubleshooting

### No Synchronization

- Check ESP-NOW communication range
- Verify peer MAC addresses are correct
- Ensure synchronization is enabled

### Poor Synchronization Quality

- Increase sync frequency
- Adjust acceleration/deceleration factors
- Check network conditions

### Large Time Corrections

- Reduce max correction threshold
- Increase sync interval
- Check for network delays

## Performance Considerations

- **Memory Usage**: Each peer consumes ~100 bytes of RAM
- **CPU Usage**: Synchronization processing is lightweight
- **Network Traffic**: Minimal ESP-NOW traffic (few bytes per sync cycle)
- **Power Consumption**: ESP-NOW is power-efficient for IoT applications

## Future Enhancements

- **Encryption**: Add ESP-NOW encryption for secure time synchronization
- **Mesh Support**: Extend to multi-hop mesh networks
- **GPS Integration**: Use GPS for absolute time reference
- **Adaptive Algorithms**: Implement machine learning-based synchronization
