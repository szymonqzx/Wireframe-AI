# Wireframe-AI TUI Modular Architecture Plan (jcode-inspired)

## Current State
- Single monolithic crate: `wireframe-tui`
- 11 Rust files in `src/` directory
- All functionality coupled together
- Poor compilation caching (changes trigger full rebuilds)

## Target State (jcode-inspired)
- 10-15 focused crates in workspace
- Each crate has clear responsibility
- Changes isolated to specific crates
- Excellent compilation caching
- Fast development iteration

## Proposed Crate Structure

### Core Crates
```
tools/tui-chat/
├── Cargo.toml (workspace root)
├── crates/
│   ├── wireframe-tui-core/          # Core TUI logic, state management
│   ├── wireframe-tui-render/        # Rendering pipeline, buffer management
│   ├── wireframe-tui-widgets/       # Widget library (text, lists, tables)
│   ├── wireframe-tui-input/         # Input handling, keybindings
│   ├── wireframe-tui-layout/        # Layout engine, constraints
│   ├── wireframe-tui-theme/         # Theme system, color management
│   ├── wireframe-tui-config/        # Configuration management
│   ├── wireframe-tui-nats/          # NATS integration, agent communication
│   └── wireframe-tui/              # Main binary, orchestrates crates
└── src/ (removed - moved to crates)
```

### Crate Responsibilities

#### wireframe-tui-core
- **Purpose**: Core TUI application logic
- **Exports**: `App`, `AppState`, event handling
- **Dependencies**: tui-config, tui-theme
- **Files**: app.rs, model.rs

#### wireframe-tui-render
- **Purpose**: Rendering pipeline and buffer management
- **Exports**: render functions, buffer utilities
- **Dependencies**: tui-core, tui-layout, tui-widgets
- **Files**: view.rs (rendering logic)

#### wireframe-tui-widgets
- **Purpose**: Widget library
- **Exports**: Text, List, Table, custom widgets
- **Dependencies**: tui-theme
- **Files**: widgets.rs

#### wireframe-tui-input
- **Purpose**: Input handling and keybindings
- **Exports**: input handlers, keybinding system
- **Dependencies**: tui-core, tui-config
- **Files**: command.rs (input parsing)

#### wireframe-tui-layout
- **Purpose**: Layout engine and constraints
- **Exports**: layout functions, constraint system
- **Dependencies**: None
- **Files**: panels.rs (layout logic)

#### wireframe-tui-theme
- **Purpose**: Theme system and color management
- **Exports**: Theme types, color utilities
- **Dependencies**: None
- **Files**: theme.rs

#### wireframe-tui-config
- **Purpose**: Configuration management
- **Exports**: Config types, loading/saving
- **Dependencies**: None
- **Files**: config.rs

#### wireframe-tui-nats
- **Purpose**: NATS integration and agent communication
- **Exports**: NATS client, message handling
- **Dependencies**: tui-core, agentic-sdk
- **Files**: process_manager.rs, NATS logic from app.rs

#### wireframe-tui
- **Purpose**: Main binary and orchestration
- **Exports**: Binary only
- **Dependencies**: All other crates
- **Files**: main.rs

## Migration Strategy

### Phase 1: Create Workspace Structure
1. Create `crates/` directory
2. Create workspace Cargo.toml with member crates
3. Set up each crate with basic Cargo.toml

### Phase 2: Extract Core Logic
1. Move `model.rs` → `wireframe-tui-core/src/lib.rs`
2. Move `app.rs` → `wireframe-tui-core/src/app.rs`
3. Create proper exports and dependencies

### Phase 3: Extract Rendering
1. Move `view.rs` → `wireframe-tui-render/src/lib.rs`
2. Extract rendering logic from app.rs
3. Set up dependencies on core and widgets

### Phase 4: Extract Widgets
1. Move `widgets.rs` → `wireframe-tui-widgets/src/lib.rs`
2. Create widget trait and implementations
3. Set up theme dependency

### Phase 5: Extract Input
1. Move `command.rs` → `wireframe-tui-input/src/lib.rs`
2. Extract input handling from app.rs
3. Set up dependencies on core and config

### Phase 6: Extract Layout
1. Move `panels.rs` → `wireframe-tui-layout/src/lib.rs`
2. Extract layout logic from view.rs
3. Set up dependencies

### Phase 7: Extract Theme
1. Move `theme.rs` → `wireframe-tui-theme/src/lib.rs`
2. Create theme types and utilities
3. No dependencies

### Phase 8: Extract Config
1. Move `config.rs` → `wireframe-tui-config/src/lib.rs`
2. Create config types and loading
3. No dependencies

### Phase 9: Extract NATS
1. Move `process_manager.rs` → `wireframe-tui-nats/src/lib.rs`
2. Extract NATS logic from app.rs
3. Set up dependencies on core and agentic-sdk

### Phase 10: Update Main Binary
1. Update `main.rs` to use new crate structure
2. Wire up all dependencies
3. Test functionality

## Build System Enhancements

### sccache Integration
```toml
[build]
# In .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

### Fast Linker Configuration
- Linux: mold (fallback to lld)
- macOS: lld
- Windows: lld-link

### Build Scripts
- `scripts/dev_cargo.sh` - Development build wrapper
- `scripts/bench_compile.sh` - Compile time benchmarking
- `scripts/bench_runtime.sh` - Runtime performance benchmarking

## Performance Targets

### Compile Time
- Cold build: < 30s (down from 4m 32s)
- Warm build: < 5s (down from 0.35s)
- Selfdev warm: < 2s (down from 0.38s)

### Runtime
- Time to first frame: < 20ms (target: 14ms)
- Time to first input: < 50ms (target: 48ms)
- Memory per session: < 20MB (target: 10MB)

## Benefits

### Compilation Caching
- Changes to widgets don't trigger core rebuilds
- Changes to theme don't trigger layout rebuilds
- Parallel compilation of independent crates

### Development Speed
- Faster iteration on specific features
- Better incremental compilation
- Clear module boundaries

### Code Organization
- Clear separation of concerns
- Easier testing per crate
- Better code reuse

## Risks

### Complexity
- More crates to manage
- Dependency graph complexity
- Build system complexity

### Migration Effort
- Significant refactoring required
- Potential for breaking changes
- Testing effort across crates

### Performance
- Overhead from crate boundaries
- Potential for slower cold builds
- Need to measure actual impact

## Mitigation

### Incremental Migration
- Migrate one crate at a time
- Test after each migration
- Keep monolithic version as fallback

### Measurement
- Benchmark at each phase
- Compare before/after metrics
- Stop if performance degrades

### Documentation
- Document crate boundaries
- Document dependency graph
- Document migration process

## Success Criteria

1. All functionality preserved
2. Compile time improved by 50%+
3. Runtime performance maintained or improved
4. Code organization clearer
5. Development iteration faster
