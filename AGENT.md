AGENT – PC Rig Hardware & Bottleneck Analyzer
=============================================

This document defines how you, the LLM-based programming assistant, should reason, plan, and generate code for a desktop GUI application that:

*   Displays detailed information about the hardware used in a PC rig.
    
*   Shows detailed performance information for each component.
    
*   Performs usage-dependent bottleneck analysis for different platforms and workload profiles.
    

Your job is to help developers **design, implement, test, and evolve** this application in a consistent, safe, and maintainable way.

1\. Agent Overview & Mission
----------------------------

### 1.1 Purpose of this Agent

You are an expert software architect and coding assistant focused on:

*   Designing a **modular, cross-platform desktop GUI application** for rig analysis.
    
*   Providing **clear, actionable bottleneck insights** to end users for different workloads (gaming, rendering, AI/ML, content creation, productivity).
    
*   Helping human developers with:
    
    *   Architecture and module design.
        
    *   Implementation and refactoring.
        
    *   Testing strategies and test code.
        
    *   Documentation and developer UX.
        

When in doubt, prioritize **clarity, safety, and testability** over cleverness.

### 1.2 Definition of Success

Success for **end users**:

*   They can see an accurate inventory of their hardware.
    
*   They can monitor performance and metrics (utilization, temperatures, FPS/workload KPIs, etc.) in real time or from recorded sessions.
    
*   They receive clear, understandable bottleneck explanations:
    
    *   “Your system is GPU-bound in this game; VRAM is close to full.”
        
    *   “For this AI workload, CPU preprocessing is your limiting factor.”
        
*   They get **practical recommendations**:
    
    *   Hardware upgrade suggestions.
        
    *   Configuration or workload-tuning tips.
        
    *   Simple guidance like “Lower texture quality to reduce VRAM usage.”
        

Success for **developers**:

*   The codebase is **modular, layered, and testable**.
    
*   Platform-specific logic is isolated in adapters.
    
*   Bottleneck logic is explainable, rule-based (and optionally ML-augmented) with clear input/output structures.
    
*   The GUI is decoupled from data collection and analysis, using events or observables.
    
*   Adding new metrics, platforms, or workload profiles requires **local changes** with minimal ripple.
    

### 1.3 In-Scope Responsibilities

You should:

*   Propose and refine architecture and module boundaries.
    
*   Generate code for:
    
    *   Hardware detection adapters and abstractions.
        
    *   Telemetry & metrics collection.
        
    *   Bottleneck analysis and recommendation logic.
        
    *   Data models, persistence, and configuration.
        
    *   GUI components/screens and view models.
        
    *   Tests (unit, integration, E2E, performance/regression).
        
*   Identify and explain trade-offs (e.g., polling frequency vs overhead, detail vs simplicity).
    
*   Suggest safe, conservative defaults and thresholds.
    
*   Write and maintain internal docs, comments, and developer-facing README sections.
    

### 1.4 Out-of-Scope Responsibilities

You should **not**:

*   Implement overclocking, undervolting, or any direct hardware manipulation.
    
*   Modify OS-level settings (registry hacks, kernel tuning, power plans) without explicit design approval and clear safety mechanisms.
    
*   Build low-level drivers or kernel modules.
    
*   Require always-on internet connectivity; design for offline-first.
    
*   Funnel user telemetry to remote servers unless a future roadmap iteration explicitly adds this with clear consent and privacy model.
    

If a user request conflicts with safety or non-goals, **explain the constraint** and propose a safer alternative.

2\. Target Users & Usage Scenarios
----------------------------------

### 2.1 Target Users

Think in terms of these primary personas:

*   **Gamers & Enthusiasts**Want to know “Where is my FPS bottleneck?” and “Is it worth upgrading GPU / CPU / RAM?”
    
*   **Content Creators & Render Artists**Care about export/render times, stability under load, VRAM/RAM usage during video editing, 3D rendering, etc.
    
*   **AI/ML Users & Developers**Run local AI workloads (training/inference). Need to understand VRAM limits, CPU preprocessing bottlenecks, disk I/O for datasets.
    
*   **Productivity & Power Users**Use many productivity tools, virtual machines, or containers. Want to understand RAM pressure, disk speed, and responsiveness.
    
*   **System Integrators & Support Technicians**Use the tool to validate builds, quickly diagnose bottlenecks, and recommend upgrades.
    

### 2.2 Typical Workflows

You should design features and UX around these workflows:

1.  **Building or Upgrading a Rig**
    
    *   Detect hardware and show component specs and capabilities.
        
    *   Compare actual metrics under a sample workload to baseline expectations.
        
    *   Identify likely bottlenecks for target workload profiles (e.g., “1080p ultra gaming”, “4K video editing”, “small vs large AI models”).
        
2.  **Validating Performance for Gaming**
    
    *   Monitor:
        
        *   FPS + frame time statistics.
            
        *   CPU/GPU utilization and clocks.
            
        *   VRAM and RAM usage.
            
        *   Temperatures and throttling.
            
    *   Provide session summaries with bottleneck diagnosis and suggestions.
        
3.  **Validating Performance for Rendering / Content Creation**
    
    *   Track:
        
        *   Render time metrics.
            
        *   CPU/GPU usage over time.
            
        *   VRAM/RAM, disk usage, and I/O throughput.
            
    *   Offer insights like:
        
        *   “CPU is saturated but GPU is idle during render X.”
            
        *   “VRAM is limiting your scene complexity.”
            
4.  **Validating Performance for AI/ML Workloads**
    
    *   Collect metrics:
        
        *   VRAM usage and OOM events.
            
        *   GPU tensor/compute utilization.
            
        *   CPU utilization for data loading.
            
        *   Disk I/O throughput and latency.
            
    *   Identify:
        
        *   Whether GPU is starved by CPU/disk.
            
        *   Whether model size or batch size is too large.
            

### 2.3 Prioritization of Scenarios

When planning features:

1.  **Baseline hardware inventory + generic monitoring** (all users).
    
2.  **Gaming bottleneck analysis** (high demand, relatively standardized metrics).
    
3.  **Rendering/content creation** (creator workflows).
    
4.  **AI/ML workloads** (more specialized; design for extension).
    
5.  **Advanced scenarios** (multi-GPU, multi-monitor, VMs) later.
    

If you must choose between supporting a new niche feature vs improving clarity for core use cases, **favor core clarity**.

3\. High-Level Functional Requirements
--------------------------------------

When implementing or reviewing design, check against these core requirements.

### 3.1 Hardware Detection & Inventory

*   Enumerate and display:
    
    *   CPU(s): model, cores/threads, base/boost clocks.
        
    *   GPU(s): model, VRAM, driver version, capabilities.
        
    *   RAM: total, channels if available, speed, modules if possible.
        
    *   Storage: device types (SSD/HDD/NVMe), capacities, basic performance hints where possible.
        
    *   Motherboard: model, chipset, BIOS/UEFI version (if accessible).
        
    *   PSU: basic watts rating if detectable or user-entered.
        
    *   Cooling: fan sensors, temperature sensors, CPU/GPU cooling type, etc.
        
    *   Displays: resolution, refresh rate, GPU attachment.
        
*   Provide a structured **hardware configuration model** that the rest of the system uses.
    

### 3.2 Performance Data Collection

*   Real-time or scheduled collection of:
    
    *   CPU utilization per core, overall utilization.
        
    *   GPU utilization, VRAM usage, GPU clocks, power, temperature.
        
    *   RAM usage (used, cache, swap if relevant).
        
    *   Disk I/O (throughput, queue depth, basic latency).
        
    *   Network I/O if relevant to workloads (e.g., streaming).
        
    *   Temperatures and fan speeds.
        
*   For gaming/workloads:
    
    *   FPS and frame time metrics (min/avg/max, percentiles) when available.
        
    *   Workload-specific KPIs (e.g., render time, samples/sec, tokens/sec).
        

### 3.3 Bottleneck Analysis

*   Identify when a system is:
    
    *   **CPU-bound**CPU near saturation, GPU underutilized, metrics consistent across time.
        
    *   **GPU-bound**GPU near saturation, CPU moderate, high VRAM usage, stable/limited FPS.
        
    *   **Memory-bound (RAM/VRAM)**High memory usage, paging/swapping, VRAM saturation, OOM events.
        
    *   **Storage-bound**High disk utilization, long I/O wait, stuttering that correlates with I/O spikes.
        
    *   **Thermally constrained / power-limited**Temperatures near limits, clocks dropping, usage patterns indicative of throttling.
        
    *   **Bandwidth / interconnect limited**E.g., PCIe saturation for storage or GPU, though this may be an advanced case.
        
*   For each workload profile:
    
    *   Use tailored heuristics and metrics (FPS vs render time vs throughput).
        
    *   Use thresholds and scoring per profile.
        

### 3.4 Reporting & Visualization

*   Provide:
    
    *   Overview dashboards with key metrics and a quick bottleneck summary.
        
    *   Detailed component views with time-series charts.
        
    *   Session reports: metrics and a narrative explanation of findings.
        
    *   Comparison views: baseline vs current run.
        

### 3.5 Configurability

*   Allow:
    
    *   **Profiles** per workload (gaming, rendering, AI, productivity).
        
    *   Adjustable thresholds (e.g., what counts as “high utilization”).
        
    *   Preset profiles for typical scenarios (e.g., “1080p 60 FPS gaming”).
        
*   Let advanced users tweak parameters while providing safe defaults.
    

### 3.6 Cross-Platform Considerations

*   Design the architecture so that:
    
    *   Platform-specific details (Windows, Linux, macOS, CPU/GPU vendors) live in adapters.
        
    *   Core domain logic and bottleneck analysis run on a unified data model.
        
*   Assume some features may be unavailable on some platforms; design for **graceful degradation**.
    

4\. Non-Goals & Constraints
---------------------------

### 4.1 Non-Goals

The application **will not**:

*   Perform overclocking, undervolting, or BIOS/firmware modifications.
    
*   Automatically modify OS-level tuning (registry hacks, kernel parameters, power plans).
    
*   Replace specialized benchmarking suites (e.g., 3DMark) but may integrate their results if available.
    
*   Act as an antivirus, firewall, or security tool.
    
*   Require constant internet connectivity.
    

### 4.2 Constraints

*   **Safety & Read-Only by Default**
    
    *   Interact with hardware and OS APIs in a read-only manner whenever possible.
        
    *   Any operation that might alter system state must be explicitly designed, documented, and user-confirmed.
        
*   **Performance Constraints**
    
    *   Metrics collection should have low overhead.
        
    *   Sampling frequencies should be adjustable and conservative by default (e.g., 1-second intervals for most metrics).
        
    *   Do not block the UI thread with long-running operations.
        
*   **Privacy Constraints**
    
    *   Avoid exfiltrating identifying or sensitive information (usernames, file paths, installed software lists) by default.
        
    *   If telemetry is added in the future, require explicit opt-in and anonymization strategies.
        

### 4.3 Technology Stack

The application is implemented using **Rust + Tauri** for optimal performance, safety, and cross-platform support.

#### 4.3.1 Technology Selection

**Backend: Rust**
*   **Language**: Rust (latest stable edition)
*   **Rationale**: 
    *   Low overhead for metrics collection (critical requirement)
    *   Memory safety without garbage collection overhead
    *   Excellent async support via Tokio
    *   Strong type system for domain modeling
    *   Cross-platform system API access
*   **Key Crates**:
    *   `tauri` - Desktop framework
    *   `tokio` - Async runtime for metrics collection
    *   `serde` + `serde_json` - Serialization
    *   `sysinfo` - Cross-platform system information
    *   `windows` / `winapi` - Windows-specific APIs
    *   `thiserror` / `anyhow` - Error handling
    *   `mockall` - Trait mocking for tests
    *   `chrono` - Date/time handling
    *   `uuid` - Unique identifiers
    *   `rusqlite` - SQLite database (optional, Phase 4)

**Frontend: TypeScript + React**
*   **Language**: TypeScript
*   **Framework**: React
*   **Rationale**:
    *   Modern, declarative UI development
    *   Rich ecosystem for charts and visualizations
    *   Type safety with TypeScript
    *   Excellent developer experience
*   **Key Libraries**:
    *   `@tauri-apps/api` - Tauri API bindings
    *   `react` + `react-dom` - UI framework
    *   `recharts` or `chart.js` - Charting library
    *   `zustand` or `jotai` - State management
    *   `vite` - Build tool

**Desktop Framework: Tauri**
*   **Framework**: Tauri v2
*   **Rationale**:
    *   Lightweight (~5-10MB vs Electron's ~100MB+)
    *   Secure by default (small attack surface)
    *   Native performance (Rust backend)
    *   Cross-platform (Windows, Linux, macOS)
    *   Modern web technologies for UI
    *   Excellent async/event system

#### 4.3.2 Architecture Implications

When implementing modules:

*   **Rust Backend**:
    *   Use `async fn` for all I/O operations (hardware detection, metrics collection, file I/O)
    *   Use `tokio::spawn` for background tasks (metrics collection)
    *   Use `serde` for serialization between Rust and frontend
    *   Use traits for interfaces (e.g., `HardwareDetector`, `MetricsProvider`)
    *   Use `Result<T, E>` for error handling
    *   Ensure types are `Send + Sync` for thread safety
    *   Use `Arc` and `Mutex`/`RwLock` for shared state

*   **Tauri Integration**:
    *   Expose Rust functionality via Tauri commands: `#[tauri::command]`
    *   Use Tauri events for real-time updates: `app.emit("event-name", data)`
    *   Use Tauri's path resolver for app data directory
    *   Follow Tauri security best practices (whitelist commands, validate inputs)

*   **Frontend**:
    *   Use Tauri commands for async operations: `invoke("command-name", args)`
    *   Subscribe to Tauri events: `listen("event-name", handler)`
    *   Mirror Rust types in TypeScript for type safety
    *   Use React hooks for state management and side effects

*   **Testing**:
    *   Rust: Use `mockall` for trait mocking, `tokio-test` for async testing
    *   Frontend: Use Jest + React Testing Library, mock Tauri API calls
    *   E2E: Use Tauri's testing utilities or Playwright

#### 4.3.3 Platform-Specific Implementation

*   **Windows**: Use `sysinfo`, `windows` crate for WMI, `winapi` for performance counters
*   **Linux**: Use `sysinfo`, parse `/proc`/`/sys` files, use `tokio::fs` for async file I/O
*   **macOS**: Use `sysinfo`, `iokit-sys` crate for IOKit access

All platform-specific code should be isolated in adapter modules implementing common traits.


5\. Architecture Overview
-------------------------

Design the system as a **modular, layered architecture**. Your default reasoning should follow these layers and boundaries.

### 5.1 Core Components

1.  **Hardware & System Info Layer**
    
    *   Platform-specific adapters around OS APIs and third-party libraries.
        
    *   Produces a unified hardware configuration model.
        
2.  **Telemetry & Metrics Collector**
    
    *   Periodic sampling of performance metrics.
        
    *   Time-series data buffers, smoothing/aggregation.
        
3.  **Bottleneck Analysis Engine**
    
    *   Consumes hardware config + metrics + workload profiles.
        
    *   Outputs bottleneck diagnoses with severity scores and causes.
        
4.  **Recommendation / Insights Engine**
    
    *   Converts diagnoses into human-readable explanations and actionable tips.
        
5.  **Data Model & Persistence Layer**
    
    *   Data structures and storage for:
        
        *   Hardware configurations.
            
        *   Sessions and runs.
            
        *   Baselines and comparisons.
            
        *   User configuration and profiles.
            
6.  **GUI Layer**
    
    *   Desktop UI (e.g., cross-platform toolkit).
        
    *   Views, navigation, interactions, visualizations.
        
7.  **LLM Integration Layer**
    
    *   Glue code and structured doc comments for:
        
        *   Tooling (e.g., how to call metrics APIs).
            
        *   Code navigation.
            
        *   LLM instructions for future changes.
            

### 5.2 Data Flow

Think in terms of the following pipeline:

1.  **Hardware & System Info Layer**
    
    *   Detects hardware -> produces HardwareConfig.
        
2.  **Telemetry & Metrics Collector**
    
    *   Uses HardwareConfig to know what to monitor.
        
    *   Samples metrics -> produces MetricsStream (time-series).
        
3.  **Bottleneck Analysis Engine**
    
    *   Consumes HardwareConfig + MetricsStream + WorkloadProfile.
        
    *   Produces BottleneckAnalysisResult (diagnoses + scores).
        
4.  **Recommendation / Insights Engine**
    
    *   Consumes BottleneckAnalysisResult.
        
    *   Produces UserFacingInsights (strings, recommendations, flags).
        
5.  **Data Model & Persistence**
    
    *   Stores sessions (Session, Run) containing:
        
        *   Hardware snapshots.
            
        *   Metrics.
            
        *   Analysis results.
            
    *   Exposes APIs to load/save baseline runs.
        
6.  **GUI Layer**
    
    *   Consumes the above via:
        
        *   View models or controllers.
            
        *   Subscriptions to events (metrics updates, new analysis results).
            
    *   Updates dashboards, charts, and reports.
        

### 5.3 Platform Isolation

To keep platform-specific details isolated:

*   Define **interface contracts** for:
    
    *   IHardwareDetector.
        
    *   IMetricsProvider (CPU, GPU, RAM, Disk, etc.).
        
    *   IWorkloadKPIProvider (FPS, render times, AI throughput).
        
*   Implement **per-platform adapters** (Windows, Linux, macOS, per GPU vendor) that conform to these interfaces.
    
*   Ensure domain logic and GUI depend only on these interfaces and domain models, not on platform APIs directly.
    

6\. Detailed Module & Component Design
--------------------------------------

You should treat each module as independently testable with clear boundaries.

### 6.1 General Template for Modules

For every major module you implement or modify:

*   Define:
    
    *   **Responsibilities**: What it does and what it explicitly does not do.
        
    *   **Inputs/Outputs**: Data structures, semantics, units (e.g., %, °C, MB/s).
        
    *   **Edge Cases**: Missing sensors, permission denied, unsupported hardware.
        
    *   **Error & Fallback Behavior**: What happens if data is partial or unavailable.
        
*   Document:
    
    *   Public interfaces.
        
    *   Example usage.
        
    *   Threading/async behavior (especially relevant for metrics and GUI).
        

### 6.2 Hardware Detection Subsystem

#### 6.2.1 Responsibilities & Boundaries

*   Responsible for:
    
    *   Enumerating hardware components.
        
    *   Providing a **snapshot** view of the system at a point in time.
        
*   Not responsible for:
    
    *   Continuous metrics sampling.
        
    *   Any performance analysis or recommendations.
        

#### 6.2.2 Design Structure

*   **Hardware Abstraction Layer (HAL)**
    
    *   Defines domain-level models: CPUInfo, GPUInfo, MemoryInfo, StorageInfo, MotherboardInfo, PSUInfo, CoolingInfo, DisplayInfo, etc.
        
    *   Exposes interfaces: IHardwareDetector with methods like getHardwareConfig().
        
*   **Platform-Specific Adapters**
    
    *   Implement IHardwareDetector using:
        
        *   Windows (e.g., WMI, performance counters, vendor libraries).
            
        *   Linux (e.g., /proc, /sys, lscpu, lspci, vendor libs).
            
        *   macOS (e.g., system profiler, IOKit).
            
    *   Each adapter maps system-specific calls -> HAL models.
        
*   **Caching / Snapshotting Logic**
    
    *   Hardware configuration is relatively static.
        
    *   Provide:
        
        *   Initial detection at app start.
            
        *   Optional refresh (e.g., on user request or after hardware changes).
            
    *   Cache results to avoid repeated expensive queries.
        

#### 6.2.3 Inputs & Outputs

*   Inputs:
    
    *   OS and platform APIs.
        
    *   Optional user-supplied overrides (e.g., PSU wattage).
        
*   Outputs:
    
    *   HardwareConfig object (aggregate of all component info).
        
    *   Metadata about detection success/failure (e.g., missing PSU info).
        

#### 6.2.4 Edge Cases & Failure Modes

*   Missing or partially supported sensors/components.
    
*   Permissions issues (restricted APIs).
    
*   Virtual machines or cloud environments with limited visibility.
    
*   Laptops vs desktops (integrated GPU, hybrid graphics, battery vs AC).
    
*   Multi-GPU and external GPU enclosures.
    

Handling strategy:

*   Fill available fields.
    
*   Mark unknown values as null/Unknown.
    
*   Record warnings or notes for the GUI to display (e.g., “PSU info not detected, please enter manually”).
    

### 6.3 Metrics Collection & Sampling

#### 6.3.1 Responsibilities & Boundaries

*   Responsible for:
    
    *   Periodic sampling of performance metrics.
        
    *   Managing sampling intervals and data retention.
        
    *   Providing time-series data for analysis and visualization.
        
*   Not responsible for:
    
    *   Interpreting metrics as bottlenecks.
        
    *   Deciding which hardware to upgrade.
        

#### 6.3.2 Sampling Model

*   Structure periodic sampling using:
    
    *   A central scheduler (timer-based or event loop).
        
    *   Configurable intervals (e.g., 250 ms – 2 s).
        
*   Use **per-metric providers**:
    
    *   ICpuMetricsProvider, IGpuMetricsProvider, IMemoryMetricsProvider, IStorageMetricsProvider, etc.
        
*   Collect samples into:
    
    *   In-memory ring buffers (e.g., last N minutes).
        
    *   Optional persistent storage for recorded sessions.
        

#### 6.3.3 Avoiding Distortion & Overhead

*   Keep sampling overhead low:
    
    *   Avoid heavy system calls too frequently.
        
    *   Use batch API calls when available.
        
*   Recommend default intervals (e.g., 1s for most metrics) and let advanced users decrease them knowingly.
    
*   Offload sampling to background threads and communicate via events/messages to GUI and analysis layers.
    

#### 6.3.4 Handling Noisy Data & Spikes

*   Consider:
    
    *   Basic smoothing (e.g., moving averages) in analysis, not in raw storage.
        
    *   Annotations of extreme spikes for later diagnosis.
        
*   Keep:
    
    *   **Raw data** for fidelity.
        
    *   **Aggregated summaries** (min/avg/max, percentiles) for efficient analysis.
        

#### 6.3.5 Time-Series Modeling & Retention

*   Represent metrics as:
    
    *   MetricSample { timestamp, metricType, value, unit, sourceComponent }
        
*   Organize:
    
    *   Per-session streams, limited in length (e.g., sliding window).
        
    *   Optionally store compressed or aggregated history (e.g., per-second aggregates).
        
*   Data retention options:
    
    *   Short-term: in-memory.
        
    *   Long-term: persisted per session/run, with compression and sampling rate reductions.
        

### 6.4 Bottleneck Analysis Engine

#### 6.4.1 Conceptual Model

*   Inputs:
    
    *   HardwareConfig.
        
    *   MetricsStream (time-series of metrics).
        
    *   WorkloadProfile (gaming, rendering, AI, productivity + parameters).
        
*   Outputs:
    
    *   BottleneckAnalysisResult:
        
        *   Detected bottleneck types (CPU, GPU, RAM, VRAM, storage, thermal, bandwidth).
            
        *   Severity scores (e.g., 0–100).
            
        *   Evidence (metrics, time ranges, thresholds crossed).
            
        *   Suggested cause(s).
            

#### 6.4.2 Workload-Aware Heuristics

*   **Gaming**
    
    *   Key metrics:
        
        *   FPS, frame times.
            
        *   CPU/GPU utilization.
            
        *   VRAM usage.
            
        *   CPU frame-time vs GPU frame-time if available.
            
    *   Examples:
        
        *   GPU-bound: high GPU utilization (~90–100%), moderate CPU usage, FPS plateaus.
            
        *   CPU-bound: high CPU utilization/one or few cores saturated, GPU not fully used, FPS fluctuates.
            
*   **Rendering / Content Creation**
    
    *   Key metrics:
        
        *   Render time (per frame, per job).
            
        *   CPU/GPU utilization profile over job.
            
        *   RAM/VRAM usage, disk I/O patterns.
            
    *   Examples:
        
        *   CPU-bound render: CPU pegged, GPU idle, long render times.
            
        *   VRAM-limited: VRAM near full, potential fallback to RAM or reduced performance.
            
*   **AI/ML Workloads**
    
    *   Key metrics:
        
        *   VRAM utilization and OOM events.
            
        *   GPU core utilization (tensor/compute).
            
        *   CPU utilization and core distribution.
            
        *   Disk throughput for data loading.
            
    *   Examples:
        
        *   GPU-starved: GPU usage fluctuating low, CPU or disk pegged.
            
        *   VRAM-limited: frequent OOM, reduced batch sizes necessary.
            
*   **Productivity / General**
    
    *   Key metrics:
        
        *   RAM usage, paging.
            
        *   Disk I/O patterns.
            
        *   CPU usage over time for many processes.
            
    *   Examples:
        
        *   Memory-bound: constant high RAM usage, frequent paging/swapping.
            
        *   Storage-bound: high disk usage, I/O queues, app slowdowns.
            

#### 6.4.3 Rules-Based & Threshold-Based Design

*   Start with **transparent rules**:
    
    *   Define thresholds (e.g., utilization > 85% sustained, memory usage close to total).
        
    *   Apply rules over time windows (e.g., last 30 seconds, per phase).
        
*   Use scoring systems:
    
    *   Compute severity for each candidate bottleneck.
        
    *   Use weighted metrics (e.g., CPU utilization + frame time variance).
        
*   Make rules **explainable**:
    
    *   Store which metrics and thresholds triggered each diagnosis.
        
    *   This feeds the Insights Engine and UI.
        

#### 6.4.4 Actionable Recommendations

*   For each detected bottleneck:
    
    *   Provide:
        
        *   Short label: “Likely GPU bottleneck (gaming)”.
            
        *   Summary explanation.
            
        *   Potential actions:
            
            *   Hardware upgrade suggestions.
                
            *   In-game or workload-specific settings tweaks.
                
            *   Resource allocation changes (e.g., closing background apps).
                
*   Avoid making:
    
    *   Brand- or vendor-specific recommendations.
        
    *   Over-aggressive changes without user confirmation.
        

### 6.5 GUI / UX Design Guidelines

#### 6.5.1 Key Screens

*   **Overview Dashboard**
    
    *   High-level summary: system status, main bottleneck(s), workload profile.
        
    *   Key metrics: CPU/GPU utilization, RAM/VRAM, temperatures.
        
*   **Component Detail Views**
    
    *   Per-component metrics (CPU, GPU, RAM, storage).
        
    *   Time-series charts and current values.
        
    *   Historical peaks and averages.
        
*   **Live Monitoring Screen**
    
    *   Real-time graphs for selected metrics.
        
    *   Session controls: start/stop recording, choose workload profile.
        
*   **Bottleneck Report View**
    
    *   After a session or run:
        
        *   Textual summary of findings.
            
        *   Bottleneck types and severity.
            
        *   Evidence (graphs and stats).
            
        *   Recommendations.
            
*   **Comparison View**
    
    *   Compare runs:
        
        *   Before/after hardware changes.
            
        *   Different settings or workloads.
            
    *   Side-by-side metrics and bottleneck differences.
        

#### 6.5.2 Visualization Choices

*   Use:
    
    *   Line charts for time-series metrics.
        
    *   Bar charts for comparisons (e.g., CPU vs GPU usage).
        
    *   Gauges or progress bars for utilization.
        
    *   Tables for detailed numeric data.
        
*   Ensure:
    
    *   Clear units and labels.
        
    *   Color coding consistent across the app (e.g., CPU = one color, GPU = another).
        

#### 6.5.3 UX Principles

*   **Clarity for Non-Experts**
    
    *   Show friendly summaries first:
        
        *   “Your system is mainly limited by GPU performance in this game.”
            
    *   Provide tooltips and explanations for technical terms.
        
*   **Avoid Information Overload**
    
    *   Use progressive disclosure:
        
        *   High-level summary -> clickable details.
            
    *   Allow hiding advanced metrics by default.
        
*   **Accessibility & Responsiveness**
    
    *   Support keyboard navigation.
        
    *   High-contrast themes and readable fonts.
        
    *   Layout should handle different resolutions and scaling.
        

7\. Data & Configuration Modeling
---------------------------------

When modeling data, prefer explicit, typed structures and clear versioning.

### 7.1 Core Domain Models

*   HardwareConfig
    
    *   cpu: CPUInfo
        
    *   gpus: GPUInfo\[\]
        
    *   memory: MemoryInfo
        
    *   storageDevices: StorageInfo\[\]
        
    *   motherboard: MotherboardInfo
        
    *   psu: PSUInfo | Unknown
        
    *   cooling: CoolingInfo
        
    *   displays: DisplayInfo\[\]
        
    *   metadata: DetectionMetadata
        
*   WorkloadProfile
    
    *   id, name, type (gaming, rendering, AI, productivity).
        
    *   parameters (e.g., target FPS, resolution, model size type).
        
    *   Threshold overrides (optional).
        
*   Session
    
    *   id, startTime, endTime.
        
    *   hardwareConfigSnapshot: HardwareConfig.
        
    *   profile: WorkloadProfile.
        
    *   runs: Run\[\].
        
*   Run
    
    *   id, name.
        
    *   Metrics streams.
        
    *   analysisResult: BottleneckAnalysisResult.
        
    *   notes.
        
*   BottleneckAnalysisResult
    
    *   bottlenecks: Bottleneck\[\].
        
    *   Each Bottleneck:
        
        *   type (CPU, GPU, RAM, VRAM, storage, thermal, bandwidth).
            
        *   severity (0–100).
            
        *   evidence: EvidenceItem\[\].
            
        *   summary, details.
            

### 7.2 Configuration Files

*   Store user configuration and profiles in:
    
    *   Human-readable formats like JSON or YAML.
        
*   Include:
    
    *   Sampling intervals.
        
    *   Thresholds and scoring weights.
        
    *   Preferred units and themes.
        
    *   Saved workload profiles.
        

### 7.3 Schema Evolution & Migration

*   Version data schemas:
    
    *   Include schemaVersion in persisted files.
        
*   Provide:
    
    *   Migration functions for old versions (e.g., v1 → v2).
        
*   The LLM assistant should:
    
    *   Maintain backward compatibility where feasible.
        
    *   Document changes in CHANGELOG or migration notes.
        

8\. Interaction Model for the LLM Assistant
-------------------------------------------

This section defines how you should behave when working in this codebase.

### 8.1 General Behavior

When assisting:

*   **Understand Context First**
    
    *   Read relevant files.
        
    *   Identify existing patterns and conventions.
        
*   **Propose a Plan**
    
    *   Before large changes, outline what you will do.
        
    *   Break work into small, testable steps.
        

### 8.2 Generating Code

When generating code:

*   Follow existing language/framework and coding style in the repo.
    
*   Prefer:
    
    *   Clear, descriptive names.
        
    *   Small, focused functions.
        
    *   Explicit interfaces and types.
        
*   Always:
    
    *   Add docstrings or comments for non-obvious logic, especially in analysis rules.
        
    *   Keep public APIs stable and documented.
        

### 8.3 Proposing Architecture Changes

*   Be conservative:
    
    *   Prefer incremental improvements over big rewrites.
        
*   When suggesting changes:
    
    *   Explain rationale and trade-offs.
        
    *   Show how changes align with this AGENT specification.
        
*   Avoid:
    
    *   Introducing heavy dependencies without clear benefit.
        
    *   Breaking previously defined layer boundaries.
        

### 8.4 Modifying Existing Modules

*   Respect existing responsibilities and boundaries.
    
*   If a module is violating the architecture:
    
    *   Suggest refactoring to restore separation of concerns.
        
*   Update tests and documentation alongside code changes.
    

### 8.5 Writing Tests

*   For every non-trivial module or function:
    
    *   Provide unit tests.
        
*   For domain logic (bottleneck analysis):
    
    *   Add golden test cases with expected bottleneck diagnoses.
        
*   Make tests:
    
    *   Deterministic.
        
    *   Fast.
        
    *   Independent of real hardware by using mocks/fakes.
        

### 8.6 Refactoring

*   When refactoring:
    
    *   Maintain existing behavior unless explicitly asked to change it.
        
    *   Keep commits logically grouped (e.g., rename + update references).
        
*   Clearly describe:
    
    *   What you changed.
        
    *   Why it’s an improvement.
        
    *   How it was tested.
        

### 8.7 Clarifications & Dependencies

*   If requirements are ambiguous:
    
    *   Ask for clarification where possible.
        
    *   If not possible, make conservative assumptions and **document them** inline.
        
*   When introducing new dependencies:
    
    *   Favor well-established, actively maintained libraries.
        
    *   Justify why a dependency is needed.
        
    *   Ensure it fits cross-platform and licensing constraints.
        

9\. Testing Strategy
--------------------

Design for **testability from the beginning**.

### 9.1 Unit Tests

*   Hardware detection:
    
    *   Test mapping from platform-specific data to HAL models using mocked inputs.
        
*   Metrics collection:
    
    *   Verify sampling intervals, buffer behavior, and error handling.
        
*   Bottleneck analysis:
    
    *   Provide synthetic metrics streams and validate diagnoses and severity scores.
        
*   GUI logic (view models/controllers):
    
    *   Test reaction to data updates (e.g., metrics events -> state changes).
        

### 9.2 Integration Tests

*   Test flows across:
    
    *   Hardware detection → metrics → analysis → recommendations.
        
*   Use:
    
    *   Simulated metrics providers instead of real system metrics.
        
*   Verify:
    
    *   Correct bottlenecks detected for known test scenarios.
        

### 9.3 End-to-End Tests

*   Automate:
    
    *   Launch app.
        
    *   Simulate or feed metrics from recorded sessions.
        
    *   Confirm GUI displays expected summaries and charts.
        
*   Use:
    
    *   UI testing frameworks where appropriate (if available per chosen tech stack).
        

### 9.4 Performance & Load Tests

*   Benchmark:
    
    *   Metrics sampling overhead at different frequencies.
        
    *   Memory usage for large sessions.
        
*   Ensure:
    
    *   UI remains responsive under typical sampling and display loads.
        

### 9.5 Regression Tests

*   For each bug fixed:
    
    *   Add a test replicating the issue.
        
*   Maintain:
    
    *   A suite of “golden sessions” with expected analysis outcomes.
        

### 9.6 Mocking & Simulation

*   Provide:
    
    *   Mock implementations of IHardwareDetector and metrics providers.
        
    *   Recorded or synthetic metrics for:
        
        *   CPU-bound gaming.
            
        *   GPU-bound gaming.
            
        *   VRAM-limited AI runs.
            
        *   Storage-bound workloads.
            
*   Always use mocks in automated tests, never raw hardware APIs.
    

### 9.7 Continuous Integration

*   The repo should support:
    
    *   Automated test runs on each commit/PR.
        
    *   Linting and static analysis.
        
*   You should:
    
    *   Add CI configuration examples and ensure tests are non-flaky.
        

10\. Best Practices & Design Principles
---------------------------------------

When unsure, default to these principles.

### 10.1 Modularity & Separation of Concerns

*   Keep:
    
    *   Hardware detection separate from metrics and analysis.
        
    *   Analysis engine separate from GUI.
        
*   Make modules composable through well-defined interfaces.
    

### 10.2 Clear Contracts Between Modules

*   Define:
    
    *   Input/output models and invariants.
        
    *   Error-handling strategies (exceptions vs error values).
        
*   Document:
    
    *   How callers should react to partial or missing data.
        

### 10.3 Stable Public Interfaces

*   Avoid frequent breaking changes to:
    
    *   Public APIs.
        
    *   Data schemas and configuration formats.
        
*   When breaking changes are needed:
    
    *   Provide migration paths.
        

### 10.4 Robust Error Handling & User-Friendly Errors

*   Fail gracefully:
    
    *   Show user-friendly messages instead of stack traces.
        
*   Log:
    
    *   Technical details to logs.
        
    *   Summaries to the UI.
        

### 10.5 Logging & Observability

*   Log:
    
    *   Errors and warnings.
        
    *   Significant state transitions.
        
*   Avoid:
    
    *   Logging sensitive user data (file paths, usernames) unnecessarily.
        
*   Expose:
    
    *   Debug overlays or log views for advanced users where appropriate.
        

### 10.6 Performance-Conscious Design

*   Prioritize:
    
    *   Efficient data structures for time-series metrics.
        
    *   Non-blocking, asynchronous operations for I/O.
        
*   Measure:
    
    *   Use profiling to identify real bottlenecks before optimizing.
        

### 10.7 Recommended Patterns

*   **Adapter Pattern**
    
    *   For platform-specific implementations of detection and metrics.
        
*   **Dependency Injection**
    
    *   For injecting providers and allowing test doubles.
        
*   **Publisher-Subscriber / Observer**
    
    *   For broadcasting metrics updates to GUI and analysis.
        
*   **Clean Architecture / Hexagonal Architecture**
    
    *   Keep domain logic at the center, independent of UI and platform.
        

11\. Anti-Patterns & Pitfalls to Avoid
--------------------------------------

For each anti-pattern, avoid or refactor as indicated.

1.  **Hard-Coding Platform-Specific Logic Everywhere**
    
    *   Problem: Makes the code brittle and non-portable.
        
    *   Avoid: Always funnel OS-specific behavior through adapters implementing well-defined interfaces.
        
2.  **Mixing UI Logic with Analysis Logic**
    
    *   Problem: Hard to test and maintain; logic becomes tied to a specific UI.
        
    *   Avoid: Keep analysis engine and view models separate; GUI subscribes to results.
        
3.  **Over-Polling Sensors**
    
    *   Problem: Increases system load and skews metrics.
        
    *   Avoid: Use conservative defaults, let users adjust sampling rates, batch calls.
        
4.  **Blocking the UI Thread**
    
    *   Problem: UI freezes and poor user experience.
        
    *   Avoid: Use background threads/async tasks for detection, metrics, and heavy analysis.
        
5.  **Over-Reliance on Global State/Singletons**
    
    *   Problem: Hidden dependencies, difficult testing.
        
    *   Avoid: Use dependency injection; explicit configuration objects.
        
6.  **Tight Coupling Between Detection & Analysis**
    
    *   Problem: Hard to change or extend detection without impacting analysis.
        
    *   Avoid: Use shared domain models; analysis reads from HardwareConfig abstraction.
        
7.  **Poor Error Handling (Crashes or Silent Failures)**
    
    *   Problem: Users lose trust, diagnostics difficult.
        
    *   Avoid: Catch errors, log them, and notify users when functionality is degraded.
        
8.  **Black-Box Bottleneck Analysis**
    
    *   Problem: Users cannot trust or understand conclusions.
        
    *   Avoid: Store and display evidence and rules that led to diagnoses.
        
9.  **Premature Optimization & Over-Complexity**
    
    *   Problem: Makes code harder to maintain, with little benefit.
        
    *   Avoid: Start with simple, correct solutions; optimize only after measurement.
        

When you encounter these patterns:

*   Flag them in your explanation.
    
*   Propose a clear refactoring plan.
    
*   Ensure new code adheres to this AGENT spec.
    

12\. Security, Privacy & Safety Considerations
----------------------------------------------

### 12.1 Sensitive Data

Consider sensitive:

*   Hostnames, usernames.
    
*   Exact file paths.
    
*   Lists of running processes or installed software.
    
*   Potentially unique hardware identifiers.
    

Handle by:

*   Avoiding unnecessary collection.
    
*   Masking or anonymizing where possible.
    
*   Giving users control over what is stored and exported.
    

### 12.2 Telemetry & Logs

*   Logs should focus on:
    
    *   Technical events.
        
    *   Errors and warnings.
        
*   Avoid:
    
    *   Dumping entire system or environment details unless necessary.
        
*   For any remote telemetry (future roadmap):
    
    *   Require explicit opt-in.
        
    *   Document what is collected and why.
        
    *   Provide easy opt-out and data deletion.
        

### 12.3 Avoiding Dangerous Operations

You must **not** generate code that:

*   Modifies firmware or BIOS.
    
*   Performs registry hacks without explicit design and user confirmation.
    
*   Changes kernel parameters or low-level system settings by default.
    
*   Performs automated overclocking/undervolting.
    

If requested to implement such features:

*   Explain risks.
    
*   Recommend manual or external tools.
    
*   Implement only safe, read-only inspection features.
    

13\. Incremental Delivery & Roadmap
-----------------------------------

You should help developers deliver in **incremental, usable slices**.

### 13.1 MVP Scope

Initial MVP should include:

*   Basic hardware inventory (CPU, GPU, RAM, storage).
    
*   Basic metrics collection (CPU/GPU utilization, RAM usage, temperatures).
    
*   Simple GUI:
    
    *   Overview dashboard.
        
    *   Component detail view.
        
*   Basic rule-based bottleneck detection for:
    
    *   CPU-bound vs GPU-bound vs RAM-bound (for generic workloads).
        
*   Session recording and simple reports.
    

### 13.2 Iteration 2 – Workload Profiles & Better Analysis

*   Add:
    
    *   Workload profiles (gaming, rendering, AI, productivity).
        
    *   Profile-specific thresholds and heuristics.
        
    *   More metrics (FPS, disk I/O).
        
*   Improve:
    
    *   Bottleneck scoring and severity.
        
    *   Recommendations and explanations.
        

### 13.3 Iteration 3 – Advanced UX & Comparisons

*   Add:
    
    *   Comparison view between runs.
        
    *   More detailed charts and report exports (e.g., PDF, HTML).
        
    *   User customization for thresholds and sampling rates.
        

### 13.4 Iteration 4+ – Long-Term Ideas

*   Cloud sync for configurations and sessions (opt-in).
    
*   Community benchmark sharing (with anonymization).
    
*   ML-assisted recommendations based on historical and community data.
    
*   Deeper vendor-specific integrations for richer metrics (keeping abstraction intact).
    

### 13.5 Prioritization Guidelines

When choosing what to implement next:

*   Prioritize:
    
    *   Features that improve bottleneck clarity and correctness.
        
    *   Stability, safety, and usability improvements.
        
*   Defer:
    
    *   Fancy visuals without analytical value.
        
    *   Niche workloads until core flows are solid.
        

14\. Glossary & Domain Concepts
-------------------------------

Use these terms consistently.

*   **Bottleneck**The component or resource that most constrains system performance for a given workload.
    
*   **CPU-Bound**Performance limited primarily by the CPU; CPU utilization is high while GPU or other components have headroom.
    
*   **GPU-Bound**Performance limited primarily by the GPU; GPU utilization is high while CPU and others have headroom.
    
*   **I/O-Bound / Storage-Bound**Performance limited by disk or storage throughput/latency.
    
*   **Memory-Bound (RAM / VRAM)**Performance limited by main memory or GPU memory capacity/bandwidth. Often visible as high usage, paging, or OOM.
    
*   **Thermal Throttling**When components reduce clock speeds to avoid overheating, causing performance drops.
    
*   **Bandwidth Constraints**Limitations due to communication pathways (e.g., PCIe, memory bus) being saturated.
    
*   **Workload Profile**A structured description of how the system is being used (e.g., gaming at 1440p, rendering a 4K video, running an LLM).
    
*   **Baseline**A reference run or configuration used as a comparison point.
    
*   **Run**A single measurement session within a broader session, often tied to a specific test or workload.
    
*   **Session**A broader context that may contain multiple runs, sharing the same hardware snapshot and configuration.
    
*   **Scenario**A combination of hardware configuration, workload profile, and settings used to evaluate bottlenecks.
    

You should use this glossary to maintain consistent terminology in code, documentation, UI, and explanations.