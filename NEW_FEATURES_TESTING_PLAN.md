# New Features Testing Plan

## Overview

This document outlines comprehensive testing strategies for the newly implemented features:
1. **GPU Detection** (Windows)
2. **GPU Metrics Provider** (Windows, Linux)
3. **CPU Temperature Detection** (Windows, Linux, macOS)
4. **Storage Detection** (Windows)
5. **Storage Metrics Provider** (Windows, Linux)

---

## Testing Strategy

### Test Categories
1. **Unit Tests** - Test individual functions in isolation
2. **Integration Tests** - Test component interactions
3. **Platform-Specific Tests** - Test platform-specific implementations
4. **Error Handling Tests** - Test graceful degradation
5. **Performance Tests** - Ensure metrics collection doesn't impact system
6. **Edge Case Tests** - Test boundary conditions and unusual scenarios

---

## 1. GPU Detection Testing (Windows)

### 1.1 Unit Tests

#### Test: `test_windows_gpu_detection_basic`
- **Purpose**: Verify basic GPU detection via WMI
- **Setup**: Mock WMI output or use real system
- **Assertions**:
  - Returns non-empty vector if GPUs present
  - Each GPU has valid model name
  - VRAM is parsed correctly (if available)
  - Driver version is parsed (if available)
  - PCI ID is parsed (if available)

#### Test: `test_windows_gpu_detection_vendor_detection`
- **Purpose**: Verify vendor detection from GPU name
- **Test Cases**:
  - NVIDIA GPUs detected correctly
  - AMD GPUs detected correctly
  - Intel GPUs detected correctly
  - Unknown vendors handled gracefully

#### Test: `test_windows_gpu_detection_wmi_failure`
- **Purpose**: Test graceful degradation when WMI fails
- **Setup**: Simulate WMI command failure
- **Assertions**:
  - Returns empty vector (graceful degradation)
  - No panic or crash
  - Error is handled appropriately

#### Test: `test_windows_gpu_detection_parse_edge_cases`
- **Purpose**: Test parsing edge cases
- **Test Cases**:
  - Empty WMI output
  - Malformed CSV output
  - Missing fields in WMI response
  - Very large VRAM values
  - Special characters in model names

### 1.2 Integration Tests

#### Test: `test_gpu_detection_integration`
- **Purpose**: Test GPU detection as part of full hardware config
- **Setup**: Call `get_hardware_config()` on Windows
- **Assertions**:
  - GPU detection doesn't block other hardware detection
  - GPUs are included in `HardwareConfig`
  - Multiple GPUs are detected correctly
  - GPU info is serializable (for Tauri commands)

---

## 2. GPU Metrics Provider Testing

### 2.1 Windows GPU Metrics Tests

#### Test: `test_windows_nvidia_metrics_basic`
- **Purpose**: Verify nvidia-smi metrics collection
- **Setup**: System with NVIDIA GPU and nvidia-smi installed
- **Assertions**:
  - Utilization is between 0.0 and 1.0
  - VRAM values are reasonable (if available)
  - Temperature is within valid range (0-150°C)
  - Clock speeds are reasonable (if available)
  - Power consumption is reasonable (if available)

#### Test: `test_windows_nvidia_metrics_parse_csv`
- **Purpose**: Test CSV parsing from nvidia-smi
- **Test Cases**:
  - Valid nvidia-smi output
  - Missing fields in output
  - Invalid numeric values
  - Empty output
  - Multiple GPUs (first GPU used)

#### Test: `test_windows_nvidia_metrics_fallback`
- **Purpose**: Test fallback to WMI when nvidia-smi fails
- **Setup**: Simulate nvidia-smi failure
- **Assertions**:
  - Falls back to WMI gracefully
  - Returns zero metrics (indicating unavailable)
  - No panic or crash

#### Test: `test_windows_gpu_metrics_no_gpu`
- **Purpose**: Test behavior when no GPU is present
- **Assertions**:
  - Returns zero metrics gracefully
  - No errors thrown

### 2.2 Linux GPU Metrics Tests

#### Test: `test_linux_nvidia_metrics_basic`
- **Purpose**: Verify nvidia-smi on Linux
- **Setup**: Linux system with NVIDIA GPU
- **Assertions**: Same as Windows nvidia-smi tests

#### Test: `test_linux_gpu_metrics_no_nvidia`
- **Purpose**: Test when nvidia-smi is not available
- **Assertions**:
  - Returns zero metrics
  - No errors thrown

### 2.3 Integration Tests

#### Test: `test_gpu_metrics_collection_integration`
- **Purpose**: Test GPU metrics in metrics collector
- **Setup**: Start metrics collection
- **Assertions**:
  - GPU metrics are collected at regular intervals
  - Metrics are added to buffer
  - Metrics are broadcast via events
  - No performance degradation

#### Test: `test_gpu_metrics_serialization`
- **Purpose**: Verify GPU metrics can be serialized for Tauri
- **Assertions**:
  - `GpuMetrics` struct is serializable
  - All fields are included in serialization
  - None values are handled correctly

---

## 3. CPU Temperature Detection Testing

### 3.1 Windows CPU Temperature Tests

#### Test: `test_windows_cpu_temperature_wmi_success`
- **Purpose**: Verify WMI temperature query
- **Setup**: System with thermal sensors
- **Assertions**:
  - Temperature is returned in Celsius
  - Temperature is within valid range (0-150°C)
  - Temperature conversion from Kelvin is correct

#### Test: `test_windows_cpu_temperature_wmi_failure`
- **Purpose**: Test when WMI query fails
- **Assertions**:
  - Returns `None` gracefully
  - No panic or crash

#### Test: `test_windows_cpu_temperature_parse_kelvin`
- **Purpose**: Test Kelvin to Celsius conversion
- **Test Cases**:
  - Valid temperature conversion
  - Edge cases (very high/low temps)
  - Invalid temperature values
  - Temperature out of valid range (0-150°C)

### 3.2 Linux CPU Temperature Tests

#### Test: `test_linux_cpu_temperature_thermal_zones`
- **Purpose**: Verify reading from `/sys/class/thermal`
- **Setup**: Linux system with thermal zones
- **Assertions**:
  - Temperature is read from thermal zone files
  - Temperature is in Celsius (millidegrees converted)
  - Temperature is within valid range
  - Multiple thermal zones handled correctly

#### Test: `test_linux_cpu_temperature_no_zones`
- **Purpose**: Test when thermal zones don't exist
- **Assertions**:
  - Returns `None` gracefully
  - No errors thrown

#### Test: `test_linux_cpu_temperature_parse_millidegrees`
- **Purpose**: Test millidegrees to Celsius conversion
- **Test Cases**:
  - Valid conversion (e.g., 45000 -> 45.0°C)
  - Edge cases
  - Invalid file contents

### 3.3 Integration Tests

#### Test: `test_cpu_temperature_in_cpu_metrics`
- **Purpose**: Verify temperature is included in CPU metrics
- **Setup**: Collect CPU metrics
- **Assertions**:
  - Temperature field is populated (if available)
  - Temperature is `None` if unavailable (graceful)
  - Temperature doesn't affect other CPU metrics

#### Test: `test_cpu_temperature_thermal_bottleneck_detection`
- **Purpose**: Verify temperature is used in bottleneck analysis
- **Setup**: High CPU temperature scenario
- **Assertions**:
  - Thermal bottlenecks are detected
  - Temperature values are used in analysis
  - Severity calculation uses temperature

---

## 4. Storage Detection Testing (Windows)

### 4.1 Unit Tests

#### Test: `test_windows_storage_detection_wmi_success`
- **Purpose**: Verify WMI storage detection
- **Setup**: System with storage devices
- **Assertions**:
  - Returns non-empty vector if storage present
  - Each device has valid model name
  - Capacity is parsed correctly (bytes to MB)
  - Interface type is parsed (if available)
  - Storage type is determined correctly (NVMe/SSD/HDD)

#### Test: `test_windows_storage_detection_storage_type_detection`
- **Purpose**: Verify storage type detection logic
- **Test Cases**:
  - NVMe detection (model name, interface)
  - SSD detection (model name, interface, media type)
  - HDD detection (media type)
  - Unknown fallback for unrecognized types

#### Test: `test_windows_storage_detection_wmi_failure`
- **Purpose**: Test graceful degradation when WMI fails
- **Assertions**:
  - Falls back to sysinfo (or returns empty)
  - No panic or crash

#### Test: `test_windows_storage_detection_parse_edge_cases`
- **Purpose**: Test parsing edge cases
- **Test Cases**:
  - Empty WMI output
  - Malformed CSV
  - Missing fields
  - Very large capacity values
  - Special characters in model names

### 4.2 Integration Tests

#### Test: `test_storage_detection_integration`
- **Purpose**: Test storage detection in full hardware config
- **Assertions**:
  - Storage devices are included in `HardwareConfig`
  - Multiple storage devices detected
  - Storage info is serializable

---

## 5. Storage Metrics Provider Testing

### 5.1 Windows Storage Metrics Tests

#### Test: `test_windows_storage_metrics_typeperf_success`
- **Purpose**: Verify Performance Counter metrics
- **Setup**: System with storage I/O activity
- **Assertions**:
  - Read throughput is non-negative
  - Write throughput is non-negative
  - Queue depth is reasonable (if available)
  - Values are in correct units (MB/s)

#### Test: `test_windows_storage_metrics_parse_typeperf_output`
- **Purpose**: Test typeperf CSV parsing
- **Test Cases**:
  - Valid typeperf output
  - Missing fields
  - Invalid numeric values
  - Quoted values handling
  - Multiple data lines

#### Test: `test_windows_storage_metrics_typeperf_failure`
- **Purpose**: Test when typeperf fails
- **Assertions**:
  - Returns zero metrics gracefully
  - No panic or crash

#### Test: `test_windows_storage_metrics_bytes_to_mb_conversion`
- **Purpose**: Verify bytes/sec to MB/sec conversion
- **Test Cases**:
  - Correct conversion (bytes / (1024 * 1024))
  - Edge cases (very high/low values)
  - Zero values

### 5.2 Linux Storage Metrics Tests

#### Test: `test_linux_storage_metrics_diskstats_parsing`
- **Purpose**: Verify `/proc/diskstats` parsing
- **Setup**: Linux system with storage devices
- **Assertions**:
  - Diskstats file is read successfully
  - Physical disks are identified correctly
  - Loop devices and ramdisks are skipped
  - Sectors are parsed correctly
  - I/O in progress is parsed correctly

#### Test: `test_linux_storage_metrics_queue_depth`
- **Purpose**: Verify queue depth calculation
- **Assertions**:
  - Queue depth is sum of I/O in progress
  - Multiple disks are aggregated
  - Queue depth is reasonable value

#### Test: `test_linux_storage_metrics_no_diskstats`
- **Purpose**: Test when `/proc/diskstats` doesn't exist
- **Assertions**:
  - Returns error gracefully
  - No panic

#### Test: `test_linux_storage_metrics_throughput_delta_calculation`
- **Note**: This requires state tracking between samples
- **Purpose**: Verify throughput calculation from sector deltas
- **Future Enhancement**: Implement delta tracking

### 5.3 Integration Tests

#### Test: `test_storage_metrics_collection_integration`
- **Purpose**: Test storage metrics in metrics collector
- **Assertions**:
  - Storage metrics are collected at intervals
  - Metrics are added to buffer
  - Metrics don't block other metric collection
  - Performance is acceptable

#### Test: `test_storage_metrics_storage_bottleneck_detection`
- **Purpose**: Verify storage metrics are used in bottleneck analysis
- **Setup**: High storage I/O scenario
- **Assertions**:
  - Storage bottlenecks are detected
  - Throughput values are used in analysis
  - Queue depth affects bottleneck severity

---

## 6. Cross-Platform Testing

### 6.1 Platform Availability Tests

#### Test: `test_platform_specific_features_availability`
- **Purpose**: Verify features work on correct platforms
- **Test Cases**:
  - Windows: GPU detection, GPU metrics, CPU temp, storage detection, storage metrics
  - Linux: GPU metrics (nvidia-smi), CPU temp, storage metrics
  - macOS: CPU temp (placeholder), GPU metrics (placeholder), storage metrics (placeholder)

### 6.2 Error Handling Tests

#### Test: `test_command_execution_failures`
- **Purpose**: Test behavior when external commands fail
- **Commands to Test**:
  - `wmic` (Windows)
  - `nvidia-smi` (Windows/Linux)
  - `typeperf` (Windows)
  - File reading (Linux thermal zones, diskstats)
- **Assertions**:
  - All failures handled gracefully
  - Appropriate fallbacks used
  - No panics or crashes

#### Test: `test_permission_denied_scenarios`
- **Purpose**: Test when commands require elevated permissions
- **Assertions**:
  - Graceful degradation
  - Clear error messages (if applicable)
  - Application continues to function

---

## 7. Performance Testing

### 7.1 Metrics Collection Performance

#### Test: `test_metrics_collection_overhead`
- **Purpose**: Ensure metrics collection doesn't impact system
- **Metrics**:
  - CPU usage during collection
  - Memory usage
  - Collection latency
- **Assertions**:
  - Collection overhead < 5% CPU
  - Memory usage is reasonable
  - Collection completes within sampling interval

#### Test: `test_concurrent_metrics_collection`
- **Purpose**: Test multiple metric providers running simultaneously
- **Assertions**:
  - No race conditions
  - All metrics collected successfully
  - No performance degradation

### 7.2 Hardware Detection Performance

#### Test: `test_hardware_detection_speed`
- **Purpose**: Ensure hardware detection completes quickly
- **Assertions**:
  - Full hardware detection < 5 seconds
  - GPU detection < 2 seconds
  - Storage detection < 2 seconds

---

## 8. Edge Cases and Boundary Conditions

### 8.1 Value Range Tests

#### Test: `test_temperature_value_ranges`
- **Test Cases**:
  - Very low temperatures (< 0°C) - should be filtered
  - Very high temperatures (> 150°C) - should be filtered
  - Normal range temperatures - should be accepted
  - Missing temperature - should return None

#### Test: `test_gpu_utilization_ranges`
- **Test Cases**:
  - 0% utilization
  - 100% utilization
  - Values > 100% (should be clamped or filtered)
  - Negative values (should be filtered)

#### Test: `test_storage_capacity_ranges`
- **Test Cases**:
  - Very small drives (< 1GB)
  - Very large drives (> 10TB)
  - Zero capacity (should be filtered)
  - Invalid capacity values

### 8.2 Multi-Component Tests

#### Test: `test_multiple_gpus_detection`
- **Purpose**: Test systems with multiple GPUs
- **Assertions**:
  - All GPUs are detected
  - Each GPU has unique identification
  - Metrics are collected for primary GPU (or all)

#### Test: `test_multiple_storage_devices`
- **Purpose**: Test systems with multiple storage devices
- **Assertions**:
  - All devices are detected
  - Each device has correct information
  - Storage metrics aggregate all devices

---

## 9. Real-World Scenario Tests

### 9.1 Gaming Workload Test

#### Test: `test_gaming_workload_metrics`
- **Purpose**: Simulate gaming workload
- **Setup**: Run a game or benchmark
- **Assertions**:
  - GPU utilization increases
  - GPU temperature increases
  - VRAM usage increases
  - Storage I/O is minimal
  - CPU temperature may increase
  - Bottlenecks are detected correctly

### 9.2 Rendering Workload Test

#### Test: `test_rendering_workload_metrics`
- **Purpose**: Simulate rendering workload
- **Setup**: Run rendering software
- **Assertions**:
  - GPU utilization is high
  - CPU utilization is high
  - Storage I/O for file operations
  - Thermal bottlenecks may be detected

### 9.3 Idle System Test

#### Test: `test_idle_system_metrics`
- **Purpose**: Test metrics collection on idle system
- **Assertions**:
  - All metrics are low/zero
  - No false bottleneck detections
  - Temperature is stable
  - No errors in collection

---

## 10. Test Implementation Plan

### Phase 1: Unit Tests (Priority: High)
1. GPU Detection unit tests
2. GPU Metrics Provider unit tests
3. CPU Temperature detection unit tests
4. Storage Detection unit tests
5. Storage Metrics Provider unit tests

### Phase 2: Integration Tests (Priority: High)
1. Hardware detection integration
2. Metrics collection integration
3. Bottleneck analysis integration

### Phase 3: Error Handling Tests (Priority: Medium)
1. Command execution failures
2. Permission denied scenarios
3. Missing dependencies (nvidia-smi, etc.)

### Phase 4: Performance Tests (Priority: Medium)
1. Metrics collection overhead
2. Hardware detection speed
3. Concurrent operations

### Phase 5: Edge Cases (Priority: Low)
1. Value range tests
2. Multi-component tests
3. Boundary conditions

### Phase 6: Real-World Scenarios (Priority: Low)
1. Gaming workload
2. Rendering workload
3. Idle system

---

## 11. Test Data and Fixtures

### 11.1 Mock Data

Create mock responses for:
- WMI output (GPU, Storage)
- nvidia-smi output
- typeperf output
- `/proc/diskstats` content
- `/sys/class/thermal` files

### 11.2 Test Fixtures

Create test fixtures for:
- `HardwareConfig` with various GPU configurations
- `GpuMetrics` with different utilization levels
- `StorageMetrics` with various I/O patterns
- `CpuMetrics` with different temperatures

---

## 12. Test Execution Strategy

### 12.1 Automated Tests

- **Unit Tests**: Run on every commit
- **Integration Tests**: Run on pull requests
- **Platform-Specific Tests**: Run on CI for each platform

### 12.2 Manual Tests

- **Real Hardware Testing**: Test on actual systems
- **Performance Testing**: Measure actual overhead
- **User Acceptance**: Verify features work as expected

---

## 13. Success Criteria

### 13.1 Code Coverage
- **Target**: 80%+ coverage for new features
- **Critical Paths**: 100% coverage

### 13.2 Test Results
- **All Unit Tests**: Must pass
- **All Integration Tests**: Must pass
- **Error Handling**: All error paths tested
- **Performance**: Meets performance targets

### 13.3 Quality Metrics
- **No Panics**: All error cases handled gracefully
- **No Memory Leaks**: Proper resource cleanup
- **No Race Conditions**: Thread-safe implementations
- **Documentation**: All public APIs documented

---

## 14. Known Limitations and TODOs

### 14.1 Current Limitations
1. **Linux Storage Throughput**: Requires delta tracking (not yet implemented)
2. **macOS GPU Metrics**: Placeholder (requires IOKit)
3. **AMD GPU Support**: Not yet implemented (NVIDIA only)
4. **Storage Latency**: Not yet collected (requires additional metrics)

### 14.2 Future Enhancements
1. Implement delta tracking for Linux storage throughput
2. Add AMD GPU support (radeontop, rocm-smi)
3. Add Intel GPU support
4. Implement storage latency collection
5. Add per-disk metrics (currently aggregates all)

---

## 15. Test Files Structure

```
src-tauri/tests/
├── gpu_detection_test.rs          # GPU detection tests
├── gpu_metrics_test.rs            # GPU metrics provider tests
├── cpu_temperature_test.rs       # CPU temperature tests
├── storage_detection_test.rs      # Storage detection tests
├── storage_metrics_test.rs        # Storage metrics provider tests
├── new_features_integration_test.rs  # Integration tests
└── new_features_performance_test.rs   # Performance tests
```

---

## Conclusion

This comprehensive testing plan ensures all newly implemented features are thoroughly tested across unit, integration, platform-specific, error handling, performance, and real-world scenarios. The tests will verify that:

1. Features work correctly on their target platforms
2. Error cases are handled gracefully
3. Performance impact is minimal
4. Integration with existing systems is seamless
5. Real-world usage scenarios are supported

Implementation should proceed in phases, starting with high-priority unit and integration tests, followed by error handling and performance tests.

