# ESP32-C6 Time Synchronization Example

This example demonstrates the time synchronization system implemented in Martos RTOS using ESP-NOW communication protocol. The system implements a **Local Voting Protocol** that allows time to accelerate but prevents it from going backwards, ensuring monotonic time progression.

## Features

- **Time Synchronization**: Synchronizes time across multiple ESP32-C6 nodes using broadcast messages
- **ESP-NOW Communication**: Uses ESP-NOW for low-latency peer-to-peer communication
- **Local Voting Protocol**: Implements dynamic time acceleration algorithm with monotonic time guarantee
- **Broadcast-based Sync**: Uses broadcast messages for efficient multi-node synchronization
- **Quality Monitoring**: Tracks synchronization quality and peer performance
- **Cross-platform**: Compatible with ESP32 (Xtensa) and ESP32-C6 (RISC-V) platforms

## Architecture

The example consists of several key components:

1. **TimeSyncManager**: Main synchronization controller
2. **ESP-NOW Protocol**: Communication layer for time data exchange
3. **Sync Algorithm**: Core synchronization algorithm with dynamic correction
4. **Timer Integration**: Integration with Martos timer system

## Configuration

The synchronization system is configured with the following parameters:

- **Node ID**: `0x12345678` (unique identifier for this node)
- **Sync Interval**: 2000ms (broadcast frequency)
- **Max Correction**: 100000μs (maximum time correction per cycle)
- **Acceleration Factor**: 0.8 (rate of time acceleration)
- **Deceleration Factor**: 0.6 (rate of time deceleration)
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

1. Flash the example to multiple ESP32-C6 devices (ESP32 and ESP32-C6)
2. Ensure devices are within ESP-NOW communication range
3. Monitor serial output to see synchronization progress
4. Watch the `diff` value decrease as synchronization improves

### Expected Output

```
ESP32-C6: Setup time synchronization!
ESP32-C6: Time synchronization setup complete!
ESP32-C6: Received timestamp: 30288013μs, corrected time: 936751μs, diff: 29351262μs
ESP32-C6: Current offset: 100000μs
ESP32-C6: Received timestamp: 32278010μs, corrected time: 3036532μs, diff: 29241478μs
ESP32-C6: Current offset: 200000μs
ESP32-C6: Received timestamp: 34268014μs, corrected time: 5136548μs, diff: 29131466μs
ESP32-C6: Current offset: 200000μs
ESP32-C6: Received timestamp: 36258013μs, corrected time: 7136562μs, diff: 29121451μs
ESP32-C6: Current offset: 300000μs
ESP32-C6: Received timestamp: 38248009μs, corrected time: 9236609μs, diff: 29011400μs
ESP32-C6: Current offset: 400000μs
ESP32-C6: Received timestamp: 40238008μs, corrected time: 11336665μs, diff: 28901343μs
ESP32-C6: Current offset: 400000μs
ESP32-C6: Received timestamp: 42228012μs, corrected time: 13336653μs, diff: 28891359μs
ESP32-C6: Current offset: 500000μs
ESP32-C6: Received timestamp: 44218013μs, corrected time: 15436722μs, diff: 28781291μs
ESP32-C6: Current offset: 600000μs
ESP32-C6: Received timestamp: 46208013μs, corrected time: 17538108μs, diff: 28669905μs
```

**Key observations:**
- `Received timestamp`: Time from the remote node
- `corrected time`: Local time adjusted by the synchronization offset
- `diff`: Time difference between remote and local (should decrease over time)
- `Current offset`: Virtual time offset applied to local time

## Synchronization Algorithm

The example implements a **Local Voting Protocol** algorithm with the following characteristics:

1. **Broadcast Communication**: Nodes send time broadcasts every 2 seconds
2. **Time Difference Calculation**: Compares received timestamps with local corrected time
3. **Monotonic Time**: Time can only accelerate, never go backwards
4. **Dynamic Correction**: Applies corrections based on weighted peer consensus
5. **Convergence Detection**: Algorithm detects when nodes are synchronized

### Algorithm Details

- **Acceleration Factor**: 0.8 (aggressive time acceleration)
- **Deceleration Factor**: 0.6 (moderate time deceleration)
- **Max Correction**: 100000μs (large corrections allowed for initial sync)
- **Convergence Threshold**: 50% of max correction threshold

## Customization

### Adding More Peers

```rust
let peer3 = SyncPeer::new(0x33333333, [0x33, 0x33, 0x33, 0x33, 0x33, 0x33]);
sync_manager.add_peer(peer3);
```

### Adjusting Synchronization Parameters

```rust
let sync_config = SyncConfig {
    sync_interval_ms: 1000,        // More frequent broadcasts
    max_correction_threshold_us: 50000, // Smaller corrections
    acceleration_factor: 0.9,      // More aggressive acceleration
    deceleration_factor: 0.7,      // More aggressive deceleration
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

## Cross-Platform Compatibility

This example is designed to work seamlessly between ESP32 (Xtensa) and ESP32-C6 (RISC-V) platforms:

- **ESP-NOW Compatibility**: ESP-NOW works between different ESP chipset architectures
- **Shared Protocol**: Both platforms use the same synchronization message format
- **Automatic Detection**: The system automatically detects and adapts to the target platform
- **Unified API**: Same Martos API works on both platforms

### Testing Cross-Platform Synchronization

1. Flash ESP32 example to an ESP32 device
2. Flash ESP32-C6 example to an ESP32-C6 device
3. Both devices will automatically discover and synchronize with each other
4. Monitor both serial outputs to see synchronization progress

## Future Enhancements

- **Encryption**: Add ESP-NOW encryption for secure time synchronization
- **Mesh Support**: Extend to multi-hop mesh networks
- **GPS Integration**: Use GPS for absolute time reference
- **Adaptive Algorithms**: Implement machine learning-based synchronization
