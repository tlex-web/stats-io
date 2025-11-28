# Implementation Plan: PC Rig Hardware & Bottleneck Analyzer

This document outlines the phased implementation plan for the PC Rig Hardware & Bottleneck Analyzer application, strictly following the guidelines in `AGENT.md`.

## Overview

The implementation is divided into **4 main phases** (MVP + 3 iterations), with each phase building incrementally on the previous one. Each phase is designed to deliver a usable, testable product that can be validated before moving to the next phase.

---

## Phase 0: Foundation & Setup

**Goal**: Establish project structure, core architecture patterns, and development infrastructure.

### 0.1 Project Structure & Technology Stack Selection

**Technology Stack Decision**: **Rust + Tauri**

- **Backend**: Rust (core logic, hardware access, metrics collection, bottleneck analysis)
- **Frontend**: TypeScript/React (GUI, visualizations, user interactions)
- **Framework**: Tauri v2 (lightweight, secure, cross-platform desktop framework)
- **Rationale**: 
  - Low overhead for metrics collection (critical requirement)
  - Excellent cross-platform support (Windows, Linux, macOS)
  - Strong type safety and memory safety
  - Small binary size (~5-10MB vs Electron's ~100MB+)
  - Modern web technologies for rich GUI
  - Excellent async support (Tokio) for non-blocking operations

**Tasks**:
- Initialize Tauri project with `cargo tauri init`
- Set up Rust workspace structure for modular architecture
- Configure frontend build (Vite + React + TypeScript)
- Set up project structure following modular architecture:
  ```
  src-tauri/
    src/
      core/
        domain/          # Domain models (HardwareConfig, Session, Run, etc.)
        interfaces/      # Trait definitions (HardwareDetector, MetricsProvider, etc.)
        error/           # Error types and handling
      hardware/
        hal/             # Hardware Abstraction Layer (Rust structs)
        adapters/        # Platform-specific adapters (Windows, Linux, macOS)
          windows/       # Windows-specific implementation
          linux/         # Linux-specific implementation
          macos/         # macOS-specific implementation
      metrics/
        providers/       # Per-metric providers (CPU, GPU, RAM, etc.)
        collector/       # Central metrics collector and scheduler (Tokio-based)
        models/          # MetricSample, MetricsStream, etc.
      analysis/
        engine/          # Bottleneck analysis engine
        insights/        # Recommendation/insights engine
        rules/           # Workload-specific analysis rules
      persistence/
        models/          # Data models for storage
        storage/         # Storage implementations (JSON, SQLite via rusqlite)
        migration/       # Schema migration logic
      tauri/             # Tauri commands (exposed to frontend)
        hardware.rs      # Hardware detection commands
        metrics.rs       # Metrics collection commands
        analysis.rs      # Analysis commands
        sessions.rs      # Session management commands
    Cargo.toml           # Rust dependencies
    tauri.conf.json      # Tauri configuration
  
  src/                    # Frontend (TypeScript/React)
    components/          # React components
      dashboard/         # Overview dashboard
      charts/            # Chart components (recharts, chart.js)
      hardware/          # Hardware display components
      analysis/           # Analysis result components
    hooks/               # React hooks for Tauri commands
    stores/              # State management (Zustand or similar)
    types/               # TypeScript type definitions (mirror Rust types)
    utils/               # Utility functions
    App.tsx              # Main app component
    main.tsx             # Entry point
  
  tests/
    unit/                # Rust unit tests
    integration/         # Rust integration tests
    e2e/                 # End-to-end tests
    fixtures/            # Mock data and test fixtures
  ```

**Key Rust Dependencies**:
- `tauri` - Desktop framework
- `tokio` - Async runtime for metrics collection
- `serde` + `serde_json` - Serialization
- `sysinfo` - Cross-platform system information
- `windows` / `winapi` - Windows-specific APIs
- `rusqlite` - SQLite database (optional, Phase 4)
- `chrono` - Date/time handling
- `thiserror` / `anyhow` - Error handling
- `mockall` - Mocking for tests

**Key Frontend Dependencies**:
- `react` + `react-dom` - UI framework
- `@tauri-apps/api` - Tauri API bindings
- `recharts` or `chart.js` - Charting library
- `zustand` or `jotai` - State management
- `typescript` - Type safety
- `vite` - Build tool

**Deliverables**:
- Tauri project initialized with Rust workspace
- Frontend React app configured with TypeScript
- Project structure with empty modules
- `Cargo.toml` with initial dependencies
- `package.json` with frontend dependencies
- `tauri.conf.json` configured
- Basic build/run scripts
- README with setup instructions (Rust, Node.js, Tauri CLI)

### 0.2 Core Domain Models & Interfaces

**Tasks**:
- Define core domain models in Rust (Section 7.1):
  - `HardwareConfig` struct with nested types: `CPUInfo`, `GPUInfo`, `MemoryInfo`, `StorageInfo`, `MotherboardInfo`, `PSUInfo`, `CoolingInfo`, `DisplayInfo`
    - Use `serde` for serialization/deserialization
    - Use `Option<T>` for optional fields (e.g., PSU info may be unknown)
    - Use enums for types (e.g., `StorageType::SSD`, `StorageType::HDD`, `StorageType::NVMe`)
  - `WorkloadProfile` struct (id: String, name: String, type: WorkloadType enum, parameters: HashMap<String, Value>, threshold_overrides: Option<ThresholdOverrides>)
  - `Session` struct (id: Uuid, start_time: DateTime<Utc>, end_time: Option<DateTime<Utc>>, hardware_config_snapshot: HardwareConfig, profile: WorkloadProfile, runs: Vec<Run>)
  - `Run` struct (id: Uuid, name: String, metrics_streams: HashMap<String, Vec<MetricSample>>, analysis_result: Option<BottleneckAnalysisResult>, notes: Option<String>)
  - `BottleneckAnalysisResult` struct (bottlenecks: Vec<Bottleneck>, timestamp: DateTime<Utc>)
  - `Bottleneck` struct (type: BottleneckType enum, severity: u8 (0-100), evidence: Vec<EvidenceItem>, summary: String, details: String)
  - `MetricSample` struct (timestamp: DateTime<Utc>, metric_type: MetricType enum, value: f64, unit: String, source_component: String)
- Define trait contracts (Rust traits, Section 5.3):
  - `HardwareDetector` trait with `async fn get_hardware_config(&self) -> Result<HardwareConfig, HardwareError>`
  - `CpuMetricsProvider`, `GpuMetricsProvider`, `MemoryMetricsProvider`, `StorageMetricsProvider` traits
  - `WorkloadKPIProvider` trait (for FPS, render times, etc.)
  - All traits should be async (use `async-trait` crate if needed)
- Define error types using `thiserror`:
  - `HardwareError` enum with variants for different failure modes
  - `MetricsError` enum for metrics collection failures
  - `AnalysisError` enum for analysis failures
- Add schema versioning support (Section 7.3):
  - `SchemaVersion` enum or constant
  - Migration trait and implementations

**Deliverables**:
- Rust struct definitions with `serde` derives for all domain models
- Trait definitions with async methods
- Error types with `thiserror`
- TypeScript type definitions (generated or manually maintained) matching Rust types
- Unit tests for model validation and serialization (using `serde_test` or similar)
- Schema version constants and migration framework skeleton

### 0.3 Testing Infrastructure

**Tasks**:
- Set up Rust testing framework:
  - Use built-in `#[cfg(test)]` modules
  - Use `mockall` crate for trait mocking
  - Use `tokio-test` for async testing
- Create mock implementations using `mockall`:
  - `MockHardwareDetector` with `#[automock]` attribute
  - `MockCpuMetricsProvider`, `MockGpuMetricsProvider`, `MockMemoryMetricsProvider`, etc.
  - Configure mock expectations for test scenarios
- Create test fixtures:
  - Synthetic hardware configurations (Rust structs)
  - Recorded/synthetic metrics streams for known scenarios (CPU-bound, GPU-bound, etc.)
  - Helper functions to generate test data
- Set up frontend testing:
  - Jest + React Testing Library for component tests
  - Mock Tauri API calls
- Set up CI/CD pipeline skeleton (Section 9.7):
  - GitHub Actions workflow:
    - Rust: `cargo test`, `cargo clippy`, `cargo fmt --check`
    - Frontend: `npm test`, `npm run lint`
    - Build: `cargo tauri build` (for release artifacts)
    - Cross-platform builds (Windows, Linux, macOS runners)

**Deliverables**:
- Rust test framework configuration
- Mock implementations using `mockall` for all traits
- Test fixtures library with helper functions
- Frontend test setup (Jest, React Testing Library)
- CI configuration file (`.github/workflows/ci.yml`)
- Test utilities and helpers module

---

## Phase 1: MVP (Minimum Viable Product)

**Goal**: Deliver a working application with basic hardware detection, metrics collection, simple bottleneck analysis, and a functional GUI.

**Reference**: Section 13.1 MVP Scope

### 1.1 Hardware Detection Subsystem

**Tasks** (Section 6.2):
- Implement Hardware Abstraction Layer (HAL) in Rust:
  - Define domain models for all hardware components (structs with `serde` derives)
  - Create `HardwareDetector` trait with async methods
  - Use `sysinfo` crate as base for cross-platform system info
- Implement platform-specific adapters (start with primary platform):
  - **Windows**: 
    - Use `sysinfo` for basic info
    - Use `windows` crate for WMI queries (Win32_Processor, Win32_VideoController, etc.)
    - Use `winapi` for performance counters if needed
    - Parse registry for additional details (use `winreg` crate)
  - **Linux**: 
    - Use `sysinfo` for basic info
    - Parse `/proc/cpuinfo`, `/proc/meminfo`, `/sys/class/drm/` for GPU info
    - Use `lscpu` output parsing or direct `/proc`/`/sys` file reading
    - Parse `/proc/bus/pci/devices` or use `pciutils` bindings
  - **macOS**: 
    - Use `sysinfo` for basic info
    - Use `iokit-sys` crate for IOKit access
    - Parse `system_profiler` output or use native APIs
- Implement caching/snapshotting logic:
  - Use `Arc<HardwareConfig>` for shared, immutable hardware config
  - Cache in memory with `OnceCell` or `LazyLock` for one-time initialization
  - Optional refresh mechanism via Tauri command
  - Cache results to avoid repeated expensive queries
- Handle edge cases (Section 6.2.4):
  - Missing sensors/components → use `Option<T>` and mark as `None`
  - Permissions issues → return `HardwareError::PermissionDenied`, graceful degradation
  - VMs/cloud environments → detect and return partial data with warnings
  - Multi-GPU support (basic enumeration via `sysinfo` and platform-specific APIs)

**Rust Implementation Details**:
- Use `async fn` for all detection methods (non-blocking)
- Use `Result<T, HardwareError>` for error handling
- Use `tokio::fs` for async file I/O on Linux/macOS
- Use `serde` for serialization when passing to frontend
- Implement `Send + Sync` bounds for thread safety

**Deliverables**:
- `HardwareConfig` struct populated with detected hardware
- Platform adapter for at least one OS (Windows recommended first)
- Error handling with `HardwareError` enum
- Tauri command to expose hardware detection to frontend
- Unit tests with mocked platform APIs using `mockall`
- Integration tests with real hardware (optional, manual)

**Testing** (Section 9.1):
- Unit tests: Mock platform APIs using `mockall`, verify mapping to HAL models
- Integration tests: Verify detection on real hardware (manual validation)
- Test error cases: Permission denied, missing sensors, VM detection

### 1.2 Basic Metrics Collection

**Tasks** (Section 6.3):
- Implement per-metric providers as Rust traits:
  - `CpuMetricsProvider` trait: `async fn get_cpu_metrics(&self) -> Result<CpuMetrics, MetricsError>`
    - CPU utilization per core, overall utilization
    - Use `sysinfo` crate for CPU metrics
  - `GpuMetricsProvider` trait: `async fn get_gpu_metrics(&self) -> Result<GpuMetrics, MetricsError>`
    - GPU utilization, VRAM usage, temperature (basic)
    - Use platform-specific APIs (NVIDIA NVML via FFI, AMD ADL, Intel, or `sysinfo`)
  - `MemoryMetricsProvider` trait: `async fn get_memory_metrics(&self) -> Result<MemoryMetrics, MetricsError>`
    - RAM usage (used, cache, swap if relevant)
    - Use `sysinfo` crate
- Implement central metrics collector using Tokio:
  - Use `tokio::time::interval` for timer-based scheduler (default 1-second intervals, configurable)
  - Run in background Tokio task (spawned on Tauri app initialization)
  - Use `Arc<Mutex<RingBuffer<MetricSample>>>` or `Arc<RwLock<VecDeque<MetricSample>>>` for thread-safe ring buffer
  - Ring buffer for in-memory storage (last N minutes, e.g., 10 minutes = 600 samples at 1s interval)
  - Use `tokio::sync::broadcast` or `tokio::sync::watch` to notify frontend of new metrics
- Implement time-series data model:
  - `MetricSample` struct with `serde` derives
  - `MetricsStream` type alias or struct for time-series management
  - Basic aggregation functions: `min()`, `max()`, `avg()`, `percentile()` for analysis
- Handle errors gracefully:
  - Missing sensors → return `Ok(Metrics)` with `None` values, log warning
  - API failures → use `tokio::time::sleep` for exponential backoff retry
  - Use `tracing` or `log` crate for logging

**Rust Implementation Details**:
- Use `tokio::spawn` to run metrics collection in background task
- Use `Arc<dyn MetricsProvider>` for trait objects (or use enum dispatch for better performance)
- Use `tokio::sync::broadcast` channel to send metrics to multiple subscribers (GUI, analysis engine)
- Use `chrono::Utc::now()` for timestamps
- Implement `Send + Sync` for all types used across threads

**Tauri Integration**:
- Expose metrics via Tauri events: `app.emit("metrics-update", metrics_data)`
- Create Tauri command to get current metrics snapshot
- Create Tauri command to start/stop metrics collection

**Deliverables**:
- Working metrics collection for CPU, GPU, RAM using Rust traits
- Configurable sampling intervals (default: 1s, stored in config)
- In-memory time-series buffer with thread-safe access
- Background Tokio task that doesn't block UI
- Tauri events for real-time metrics updates
- Unit tests for sampling intervals and buffer behavior
- Performance tests to verify low overhead (< 1% CPU usage)

**Testing** (Section 9.1, 9.4):
- Unit tests: Verify sampling intervals, buffer behavior, error handling using `tokio-test`
- Performance tests: Measure overhead at different frequencies using `criterion` benchmark
- Integration tests: Verify metrics collection doesn't block main thread

### 1.3 Basic Bottleneck Analysis Engine

**Tasks** (Section 6.4):
- Implement rules-based analysis in Rust (Section 6.4.3):
  - Define thresholds as constants or configurable values:
    - `const CPU_HIGH_THRESHOLD: f64 = 0.85;` (85%)
    - `const GPU_HIGH_THRESHOLD: f64 = 0.90;` (90%)
    - `const RAM_HIGH_THRESHOLD: f64 = 0.90;` (90%)
  - Implement time-window analysis (e.g., last 30 seconds):
    - Filter `MetricSample` by timestamp
    - Use `chrono::Duration` for time calculations
  - Create scoring system (0-100 severity):
    - Use `u8` type for severity (0-100)
    - Implement scoring algorithm based on threshold violations
- Implement basic bottleneck detection for generic workloads:
  - **CPU-bound**: CPU utilization > 85% sustained, GPU < 70%
    - Check if CPU samples exceed threshold for sustained period
    - Verify GPU is below threshold
  - **GPU-bound**: GPU utilization > 90% sustained, CPU < 80%
    - Check if GPU samples exceed threshold for sustained period
    - Verify CPU is below threshold
  - **RAM-bound**: RAM usage > 90%, high paging/swapping activity
    - Check RAM usage percentage
    - Check swap usage (if available)
- Store evidence (Section 6.4.3):
  - `EvidenceItem` struct: metric_type, threshold, actual_value, time_range
  - Track which metrics triggered diagnosis
  - Store time ranges where thresholds were crossed
  - Store threshold values used
- Generate basic analysis result:
  - `BottleneckAnalysisResult` struct with detected bottlenecks
  - Severity scores (0-100)
  - Evidence items as `Vec<EvidenceItem>`

**Rust Implementation Details**:
- Use iterator methods for efficient metrics filtering and analysis
- Use `std::collections::HashMap` for grouping metrics by type
- Make analysis functions pure (no side effects) for testability
- Use `async fn` if analysis needs to be non-blocking
- Serialize results with `serde` for Tauri communication

**Deliverables**:
- Working bottleneck analysis for CPU/GPU/RAM-bound scenarios
- Explainable rules with evidence tracking
- Analysis engine as Rust module with clear separation of concerns
- Tauri command to trigger analysis
- Unit tests with synthetic metrics streams:
  - CPU-bound scenario test
  - GPU-bound scenario test
  - RAM-bound scenario test
- Integration tests: Full pipeline (hardware → metrics → analysis)

**Testing** (Section 9.1, 9.2):
- Unit tests: Synthetic metrics → validate diagnoses and severity scores
- Integration tests: Hardware detection → metrics → analysis → verify correct bottlenecks
- Test edge cases: No bottlenecks, multiple bottlenecks, threshold boundaries

### 1.4 Basic Recommendation/Insights Engine

**Tasks** (Section 6.4.4):
- Convert `BottleneckAnalysisResult` to human-readable insights:
  - Short labels: "Likely GPU bottleneck"
  - Summary explanations
  - Basic recommendations:
    - Hardware upgrade suggestions (generic, non-vendor-specific)
    - Configuration tips (e.g., "Close background applications to free RAM")
- Avoid vendor-specific or aggressive recommendations (Section 6.4.4)

**Deliverables**:
- `UserFacingInsights` generation from analysis results
- Human-readable bottleneck summaries
- Basic actionable recommendations
- Unit tests for insight generation

### 1.5 Data Model & Persistence Layer

**Tasks** (Section 5.1, 7.2):
- Implement persistence in Rust:
  - Use `serde_json` for JSON serialization/deserialization
  - Use Tauri's app data directory: `app.handle().path_resolver().app_data_dir()`
  - Implement file-based storage:
    - Hardware configurations: `hardware_config.json`
    - Sessions: `sessions/{session_id}.json`
    - User configuration: `config.json`
- Implement schema versioning (Section 7.3):
  - Include `schema_version: u32` field in persisted structs
  - Create `Migration` trait for version upgrades
  - Implement migration functions (v1 → v2, etc.)
- Implement session management:
  - `SessionManager` struct with async methods
  - Create new session: Generate UUID, save to file
  - Save session to file: Serialize with `serde_json::to_string_pretty`
  - Load session from file: Deserialize with `serde_json::from_str`
  - List saved sessions: Read directory, parse metadata
- Use `tokio::fs` for async file I/O
- Use `uuid` crate for session IDs

**Rust Implementation Details**:
- Use `serde` derives on all persistence models
- Use `Result<T, PersistenceError>` for error handling
- Implement `Send + Sync` for thread-safe access
- Use `Arc<RwLock<SessionManager>>` if shared across threads

**Deliverables**:
- JSON persistence implementation using `serde_json`
- Session save/load functionality with async file I/O
- Configuration file management
- Schema versioning support with migration framework
- Tauri commands for session management
- Unit tests for serialization/deserialization
- Integration tests for file I/O operations

### 1.6 Basic GUI - Overview Dashboard & Component Views

**Tasks** (Section 6.5.1):
- Implement Overview Dashboard (React + TypeScript):
  - Use Tauri commands to fetch hardware config on mount
  - Subscribe to Tauri events for real-time metrics: `listen('metrics-update', ...)`
  - Display hardware inventory summary (CPU, GPU, RAM specs)
  - Show key metrics: CPU/GPU utilization, RAM usage, temperatures
  - Display main bottleneck(s) with severity (from analysis result)
  - Use React hooks (`useState`, `useEffect`) for state management
- Implement Component Detail Views:
  - Per-component metrics (CPU, GPU, RAM)
  - Current values display with React components
  - Basic time-series charts using `recharts` or `chart.js`:
    - Line charts for utilization over time
    - Update charts on new metrics events
- Implement navigation between views:
  - Use React Router or simple state-based navigation
  - Create navigation component
- Ensure UI doesn't block:
  - All Tauri commands are async (handled by Rust backend)
  - Use React Suspense for loading states
  - Use `useMemo` and `useCallback` for performance

**Frontend Implementation Details**:
- Use TypeScript for type safety (mirror Rust types)
- Use Zustand or Jotai for global state management
- Use `@tauri-apps/api` for Tauri integration
- Use `recharts` for charting (lightweight, React-friendly)
- Use Tailwind CSS or similar for styling

**Deliverables**:
- Overview dashboard React component with hardware summary and key metrics
- Component detail views with basic charts
- Real-time metric updates via Tauri events
- Navigation between screens
- Responsive UI (non-blocking, async operations)
- TypeScript type definitions matching Rust types

**Testing** (Section 9.3):
- E2E tests: Launch app, verify hardware detection, verify metrics display
- Component tests: React Testing Library for UI components
- Manual UI testing for responsiveness

### 1.7 Session Recording & Simple Reports

**Tasks**:
- Implement session recording:
  - Start/stop recording button
  - Save metrics during recording
  - Associate analysis results with session
- Implement simple report generation:
  - Textual summary of findings
  - List of detected bottlenecks with severity
  - Basic evidence (which metrics triggered diagnosis)
  - Recommendations
- Export report as text or simple HTML

**Deliverables**:
- Session recording functionality
- Simple text/HTML report generation
- Save/load recorded sessions
- Unit tests for session management

### 1.8 MVP Integration & Testing

**Tasks**:
- Integrate all components:
  - Hardware detection → Metrics collection → Analysis → Insights → GUI
- End-to-end testing:
  - Full workflow: Launch app → Detect hardware → Start monitoring → Record session → Generate report
- Performance validation:
  - Verify low overhead of metrics collection
  - Verify UI responsiveness
- Bug fixes and polish

**Deliverables**:
- Fully integrated MVP application
- E2E test suite
- Performance benchmarks
- User documentation (basic)

---

## Phase 2: Workload Profiles & Enhanced Analysis

**Goal**: Add workload-aware analysis, more metrics, and improved recommendations.

**Reference**: Section 13.2 Iteration 2

### 2.1 Workload Profile System

**Tasks** (Section 3.5, 7.1):
- Implement `WorkloadProfile` system:
  - Profile types: Gaming, Rendering, AI/ML, Productivity
  - Profile parameters (e.g., target FPS, resolution, model size)
  - Threshold overrides per profile
- Create preset profiles:
  - "1080p 60 FPS Gaming"
  - "4K Video Editing"
  - "AI Model Inference (Small)"
  - "Productivity/General"
- Implement profile selection in GUI
- Save/load custom profiles

**Deliverables**:
- Workload profile data model and management
- Preset profiles
- Profile selection UI
- Unit tests for profile management

### 2.2 Enhanced Metrics Collection

**Tasks** (Section 3.2):
- Add more metrics:
  - **Disk I/O**: Throughput, queue depth, basic latency (`IStorageMetricsProvider`)
  - **Temperatures**: CPU, GPU, system temperatures (enhance existing providers)
  - **Fan speeds**: CPU/GPU/system fans (if available)
- Enhance GPU metrics:
  - GPU clocks (core, memory)
  - GPU power consumption (if available)
- Add workload-specific KPIs (Section 3.2):
  - **FPS tracking**: Implement `IWorkloadKPIProvider` for FPS (requires integration with games/benchmarks or manual input)
  - Frame time statistics (min/avg/max, percentiles)
- Improve time-series handling:
  - Better aggregation (percentiles: P50, P95, P99)
  - Longer retention options

**Deliverables**:
- Enhanced metrics providers (Disk I/O, temperatures, fan speeds, GPU clocks/power)
- FPS/workload KPI provider interface and basic implementation
- Improved time-series aggregation
- Unit tests for new metrics
- Performance tests for additional overhead

### 2.3 Workload-Aware Bottleneck Analysis

**Tasks** (Section 6.4.2):
- Implement workload-specific heuristics:

  **Gaming** (Section 6.4.2):
  - Key metrics: FPS, frame times, CPU/GPU utilization, VRAM usage
  - GPU-bound detection: High GPU utilization (~90-100%), moderate CPU, FPS plateaus
  - CPU-bound detection: High CPU utilization, few cores saturated, GPU underutilized, FPS fluctuates
  - VRAM-bound detection: VRAM near full, potential stuttering

  **Rendering/Content Creation** (Section 6.4.2):
  - Key metrics: Render time, CPU/GPU utilization over job, RAM/VRAM usage, disk I/O
  - CPU-bound render: CPU pegged, GPU idle, long render times
  - VRAM-limited: VRAM near full, potential fallback to RAM

  **AI/ML Workloads** (Section 6.4.2):
  - Key metrics: VRAM utilization, OOM events, GPU core utilization, CPU utilization, disk I/O
  - GPU-starved: GPU usage fluctuating low, CPU/disk pegged
  - VRAM-limited: Frequent OOM, reduced batch sizes

  **Productivity/General** (Section 6.4.2):
  - Key metrics: RAM usage, paging, disk I/O, CPU usage
  - Memory-bound: High RAM usage, frequent paging
  - Storage-bound: High disk usage, I/O queues, app slowdowns

- Implement thermal throttling detection (Section 3.3):
  - Monitor temperatures near limits
  - Detect clock speed drops
  - Identify throttling patterns

- Improve scoring system:
  - Weighted metrics per workload type
  - More nuanced severity calculation

**Deliverables**:
- Workload-specific bottleneck detection rules
- Thermal throttling detection
- Enhanced scoring system
- Unit tests for each workload profile:
  - Gaming: CPU-bound, GPU-bound, VRAM-bound scenarios
  - Rendering: CPU-bound, VRAM-limited scenarios
  - AI/ML: GPU-starved, VRAM-limited scenarios
  - Productivity: Memory-bound, storage-bound scenarios
- Integration tests with synthetic workloads

### 2.4 Enhanced Recommendations

**Tasks** (Section 6.4.4):
- Improve recommendation engine:
  - Workload-specific recommendations
  - More detailed explanations
  - Hardware upgrade suggestions with context (e.g., "For 4K gaming, consider GPU upgrade")
  - Configuration tips per workload (e.g., "Lower texture quality to reduce VRAM usage" for gaming)
- Add actionable tips:
  - In-game/workload settings tweaks
  - Resource allocation changes
  - Background app management suggestions

**Deliverables**:
- Enhanced recommendation engine
- Workload-specific recommendations
- More actionable insights
- Unit tests for recommendation generation

### 2.5 Enhanced GUI - Live Monitoring & Better Visualization

**Tasks** (Section 6.5.1, 6.5.2):
- Implement Live Monitoring Screen:
  - Real-time graphs for selected metrics
  - Session controls: start/stop recording, choose workload profile
  - Workload profile selector
- Improve visualizations:
  - Better time-series charts (smoother, more responsive)
  - Gauges/progress bars for utilization
  - Bar charts for comparisons
  - Color coding consistency (CPU = one color, GPU = another)
- Add tooltips and explanations for technical terms (Section 6.5.3)

**Deliverables**:
- Live monitoring screen with real-time graphs
- Enhanced visualizations
- Workload profile selection UI
- Improved UX with tooltips

### 2.6 Phase 2 Integration & Testing

**Tasks**:
- Integrate workload profiles into full pipeline
- Test all workload scenarios
- Performance validation with additional metrics
- Bug fixes and improvements

**Deliverables**:
- Fully integrated Phase 2 application
- Test suite covering all workload profiles
- Performance benchmarks
- Updated documentation

---

## Phase 3: Advanced UX & Comparisons

**Goal**: Add comparison features, better reporting, and user customization.

**Reference**: Section 13.3 Iteration 3

### 3.1 Comparison View

**Tasks** (Section 6.5.1):
- Implement comparison functionality:
  - Load multiple runs/sessions
  - Side-by-side metrics comparison
  - Before/after hardware changes comparison
  - Different settings/workloads comparison
- Visualize differences:
  - Side-by-side charts
  - Delta displays (changes between runs)
  - Bottleneck comparison (which bottlenecks changed)

**Deliverables**:
- Comparison view UI
- Side-by-side metrics display
- Delta calculations and visualization
- Unit tests for comparison logic

### 3.2 Enhanced Reporting & Export

**Tasks** (Section 3.4):
- Improve report generation:
  - More detailed narrative explanations
  - Better evidence presentation (graphs, stats)
  - Comparison reports (if comparing runs)
- Add export formats:
  - PDF export
  - HTML export (enhanced)
  - JSON export (for programmatic use)
- Add report customization:
  - Include/exclude sections
  - Custom branding/themes

**Deliverables**:
- Enhanced report generation
- Multiple export formats (PDF, HTML, JSON)
- Report customization options
- Unit tests for export functionality

### 3.3 User Customization

**Tasks** (Section 3.5):
- Implement user settings:
  - Adjustable thresholds (what counts as "high utilization")
  - Custom sampling rates (advanced users)
  - Preferred units (Celsius/Fahrenheit, MB/GB, etc.)
  - Theme preferences (light/dark, high contrast)
- Settings UI:
  - Settings screen
  - Profile-specific threshold overrides
  - Reset to defaults option

**Deliverables**:
- User settings system
- Settings UI
- Persistent configuration
- Unit tests for settings management

### 3.4 Advanced Visualizations

**Tasks** (Section 6.5.2):
- Add more chart types:
  - Heatmaps for multi-core CPU utilization
  - Stacked area charts for memory breakdown
  - Scatter plots for correlation analysis
- Improve chart interactivity:
  - Zoom/pan
  - Data point tooltips
  - Time range selection
- Add historical data views:
  - View past sessions
  - Trend analysis over time

**Deliverables**:
- Advanced visualization components
- Interactive charts
- Historical data views
- Unit tests for visualization components

### 3.5 Accessibility & UX Improvements

**Tasks** (Section 6.5.3):
- Implement accessibility features:
  - Keyboard navigation support
  - High-contrast themes
  - Screen reader support (if applicable)
  - Readable fonts and sizing
- Improve progressive disclosure:
  - Collapsible sections
  - Advanced metrics hidden by default
  - Expandable details
- Handle different resolutions and scaling

**Deliverables**:
- Accessibility improvements
- Better progressive disclosure
- Responsive layout improvements
- Accessibility testing

### 3.6 Phase 3 Integration & Testing

**Tasks**:
- Integrate all Phase 3 features
- E2E testing for comparison workflows
- Performance testing with large datasets
- User acceptance testing preparation

**Deliverables**:
- Fully integrated Phase 3 application
- Comprehensive test suite
- Performance optimizations
- User documentation updates

---

## Phase 4: Advanced Features & Future Enhancements

**Goal**: Add advanced features, platform expansion, and optional enhancements.

**Reference**: Section 13.4 Iteration 4+

### 4.1 Multi-Platform Expansion

**Tasks** (Section 3.6):
- Complete platform adapters for all target platforms:
  - Windows (if not primary)
  - Linux (complete implementation)
  - macOS (complete implementation)
- Handle platform-specific edge cases:
  - Hybrid graphics (laptops)
  - External GPU enclosures
  - Virtual machines
- Graceful degradation for unavailable features

**Deliverables**:
- Complete platform adapters
- Cross-platform testing
- Platform-specific documentation

### 4.2 Advanced Bottleneck Detection

**Tasks** (Section 3.3):
- Implement bandwidth/interconnect detection:
  - PCIe saturation detection
  - Memory bus bandwidth analysis
- Enhanced thermal analysis:
  - Predictive throttling warnings
  - Cooling efficiency analysis
- Multi-GPU scenarios:
  - SLI/CrossFire analysis
  - Workload distribution analysis

**Deliverables**:
- Advanced bottleneck detection rules
- Multi-GPU support
- Enhanced thermal analysis
- Unit tests for advanced scenarios

### 4.3 Data Persistence Enhancements

**Tasks**:
- Implement database option (SQLite) for large datasets:
  - Efficient querying of historical data
  - Indexed searches
  - Data compression
- Add data export/import:
  - Export sessions for sharing
  - Import external benchmark data
- Implement data retention policies:
  - Automatic cleanup of old sessions
  - Configurable retention periods

**Deliverables**:
- Database persistence option
- Export/import functionality
- Data retention policies
- Migration tools for existing data

### 4.4 Optional: Cloud Sync (Opt-In)

**Tasks** (Section 12.2, 13.4):
- Implement opt-in cloud sync:
  - Explicit user consent
  - Encrypted data transmission
  - Privacy-focused design
- Sync configurations and sessions:
  - Cross-device access
  - Backup and restore
- Clear opt-out mechanism

**Deliverables**:
- Cloud sync implementation (if approved)
- Privacy documentation
- Opt-in/opt-out UI
- Security audit

### 4.5 Optional: Community Features

**Tasks** (Section 13.4):
- Community benchmark sharing (with anonymization):
  - Anonymize hardware identifiers
  - Share bottleneck patterns
  - Compare with similar systems
- Privacy-first design:
  - No identifying information
  - User control over sharing

**Deliverables**:
- Community features (if approved)
- Anonymization implementation
- Privacy controls

### 4.6 Optional: ML-Assisted Recommendations

**Tasks** (Section 13.4):
- Implement ML-assisted recommendations:
  - Learn from historical data
  - Community data integration (if available)
  - Pattern recognition
- Keep explainability:
  - ML suggestions must be explainable
  - Fallback to rule-based analysis

**Deliverables**:
- ML recommendation engine (optional)
- Explainable AI integration
- Training data pipeline

---

## Testing Strategy Summary

Following Section 9, each phase includes:

### Unit Tests (Section 9.1)
- Hardware detection: Mock platform APIs, verify HAL mapping
- Metrics collection: Sampling intervals, buffer behavior, error handling
- Bottleneck analysis: Synthetic metrics → validate diagnoses
- GUI logic: View model reactions to data updates

### Integration Tests (Section 9.2)
- Full pipeline: Hardware detection → metrics → analysis → recommendations
- Use simulated metrics providers (never real hardware in CI)

### End-to-End Tests (Section 9.3)
- Launch app → simulate metrics → verify GUI displays correctly
- Use recorded sessions for deterministic testing

### Performance Tests (Section 9.4)
- Metrics sampling overhead at different frequencies
- Memory usage for large sessions
- UI responsiveness under load

### Regression Tests (Section 9.5)
- Golden sessions with expected analysis outcomes
- Bug reproduction tests

### Mocking & Simulation (Section 9.6)
- Mock implementations for all interfaces
- Synthetic metrics for known scenarios
- Never use real hardware APIs in automated tests

---

## Development Principles

Throughout all phases, adhere to:

1. **Modularity & Separation of Concerns** (Section 10.1)
   - Keep layers separate
   - Use dependency injection
   - Clear interfaces

2. **Safety & Read-Only by Default** (Section 4.2, 12.3)
   - No hardware modification
   - No OS-level changes without explicit approval
   - Graceful error handling

3. **Testability** (Section 9)
   - Test from the beginning
   - Use mocks extensively
   - Maintain test coverage

4. **Explainability** (Section 6.4.3)
   - Transparent rules
   - Evidence tracking
   - User-friendly explanations

5. **Incremental Delivery** (Section 13)
   - Each phase delivers usable value
   - Validate before moving forward
   - Prioritize core clarity over niche features

---

## Success Criteria Per Phase

### Phase 1 (MVP)
- ✅ Hardware detected and displayed
- ✅ Basic metrics collected (CPU, GPU, RAM)
- ✅ Simple bottleneck detection (CPU/GPU/RAM-bound)
- ✅ Functional GUI with overview and detail views
- ✅ Session recording and basic reports

### Phase 2
- ✅ Workload profiles implemented
- ✅ Enhanced metrics (disk I/O, temperatures, FPS)
- ✅ Workload-aware bottleneck analysis
- ✅ Improved recommendations
- ✅ Live monitoring screen

### Phase 3
- ✅ Comparison view functional
- ✅ Enhanced reporting with multiple export formats
- ✅ User customization (thresholds, sampling rates)
- ✅ Advanced visualizations
- ✅ Accessibility improvements

### Phase 4
- ✅ Multi-platform support
- ✅ Advanced bottleneck detection
- ✅ Optional cloud/community features (if approved)

---

## Notes

- **Technology Stack**: **Rust + Tauri** has been selected as the implementation stack:
  - **Backend**: Rust for core logic, hardware access, metrics collection, and analysis
  - **Frontend**: TypeScript/React for GUI, visualizations, and user interactions
  - **Framework**: Tauri v2 for cross-platform desktop application
  - **Rationale**: Low overhead, excellent performance, strong type safety, small binary size, modern web UI

- **Rust-Specific Considerations**:
  - Use `async/await` with Tokio for all I/O operations
  - Use `serde` for serialization between Rust and frontend
  - Use `thiserror` or `anyhow` for error handling
  - Use `mockall` for trait mocking in tests
  - Follow Rust best practices: ownership, borrowing, `Send + Sync` bounds

- **Tauri-Specific Considerations**:
  - Expose Rust functionality via Tauri commands (`#[tauri::command]`)
  - Use Tauri events for real-time updates (`app.emit()`)
  - Use Tauri's path resolver for app data directory
  - Follow Tauri security best practices (whitelist commands, validate inputs)

- **Platform Priority**: Start with one platform (recommended: Windows) for MVP, then expand to others in Phase 4.

- **Testing**: Always use mocks in automated tests. Real hardware testing should be manual/optional. Use `mockall` for Rust trait mocking, Jest for frontend testing.

- **Documentation**: Update documentation with each phase, including:
  - Rust API documentation (cargo doc)
  - TypeScript type definitions
  - User guides
  - Developer guides
  - Architecture diagrams

- **Backward Compatibility**: Maintain schema versioning and migration paths (Section 7.3) to ensure users can upgrade without data loss. Use `serde` versioning attributes.

- **Build & Distribution**:
  - Use `cargo tauri build` for production builds
  - Configure `tauri.conf.json` for app metadata, icons, permissions
  - Set up code signing for Windows/macOS distribution
  - Use GitHub Actions for CI/CD and automated builds

