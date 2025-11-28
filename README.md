# Stats-IO: PC Rig Hardware & Bottleneck Analyzer

A cross-platform desktop application for analyzing PC hardware and identifying performance bottlenecks.

## Technology Stack

- **Backend**: Rust (with Tauri)
- **Frontend**: TypeScript + React
- **Framework**: Tauri v2

## Prerequisites

- **Rust**: Latest stable version (1.77.2 or later)
  - Install from [rustup.rs](https://rustup.rs/)
- **Node.js**: v18 or later
  - Install from [nodejs.org](https://nodejs.org/)
- **Tauri CLI**: Will be installed automatically via npm

## Setup Instructions

### 1. Clone the Repository

```bash
git clone https://github.com/tlex-web/stats-io.git
cd stats-io
```

### 2. Install Frontend Dependencies

```bash
npm install
```

### 3. Build and Run

**Development mode:**
```bash
npm run tauri dev
```

**Build for production:**
```bash
npm run tauri build
```

The built application will be in `src-tauri/target/release/`.

## Project Structure

```
stats-io/
├── src-tauri/          # Rust backend
│   ├── src/
│   │   ├── core/       # Core domain models and interfaces
│   │   ├── hardware/   # Hardware detection
│   │   ├── metrics/    # Metrics collection
│   │   ├── analysis/   # Bottleneck analysis
│   │   ├── persistence/ # Data persistence
│   │   └── tauri/      # Tauri commands
│   └── Cargo.toml
├── src/                # Frontend (TypeScript/React)
│   ├── components/     # React components
│   ├── hooks/          # React hooks
│   ├── stores/         # State management
│   ├── types/          # TypeScript types
│   └── utils/          # Utility functions
└── tests/              # Tests
    ├── unit/           # Unit tests
    ├── integration/    # Integration tests
    └── fixtures/       # Test fixtures
```

## Development

### Running Tests

**Rust tests:**
```bash
cd src-tauri
cargo test
```

**Frontend tests:**
```bash
npm test
```

### Code Formatting

**Rust:**
```bash
cd src-tauri
cargo fmt
```

**TypeScript:**
```bash
npm run lint
```

## Current Status

**Phase 0-4: Core Implementation** ✅
- Project structure established
- Core domain models defined
- Trait interfaces created
- Error types implemented
- Frontend structure set up
- Testing infrastructure prepared
- Hardware detection (Windows, Linux, macOS)
- Metrics collection (CPU, GPU, Memory, Storage)
- Bottleneck analysis engine
- Session management and persistence
- GPU detection and metrics (Windows, Linux)
- CPU temperature detection (Windows, Linux)
- Storage detection and metrics (Windows, Linux)
- Comprehensive test suite (48+ tests)

## Next Steps

See `IMPLEMENTATION_PLAN.md` for the full implementation roadmap.

## Documentation

- [AGENT.md](./AGENT.md) - Architecture and design guidelines
- [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md) - Detailed implementation plan

## License

MIT

