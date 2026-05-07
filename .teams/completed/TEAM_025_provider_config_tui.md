---
status: completed
created: 2026-05-06
completed: 2026-05-06
---

# TEAM_025 - Provider Config in TUI

## Task
Implement provider configuration management into the TUI, allowing users to view, add, and manage LLM provider configurations directly from the TUI interface.

## Progress
- [x] Research existing provider config structure (providers.json in adapter/python)
- [x] Design TUI UI for provider management (panel, commands, keybindings)
- [x] Add provider config to TUI config structure
- [x] Implement provider listing functionality
- [x] Implement provider addition functionality
- [x] Implement provider editing/deletion functionality
- [x] Add provider status display (availability, setup state)
- [x] Test provider config integration
- [x] Update documentation

## Decisions
- Use existing providers.json format from adapter/python as the source of truth
- Add provider config section to .wireframe-tui.toml
- Create a dedicated provider management panel in TUI
- Support slash commands for provider management (/provider, /provider-add, /provider-edit, /provider-remove)
- Add keybinding for toggling provider panel (P for Providers)
- Display provider count in header for quick visibility

## Implementation Details

### Config Structure
- Added `ProvidersConfig` and `ProviderConfig` structs to `src/config.rs`
- Config path defaults to `../adapter/python/providers.json`
- Implemented `load_providers()` and `save_providers()` methods for JSON I/O

### Model Updates
- Added provider management state to `AppState` in `src/model.rs`
- Added `ProviderEditMode` enum (View, Add, Edit)
- Added edit fields for provider configuration

### UI/View
- Added `render_providers_overlay()` function in `src/view.rs`
- Three modes: View (list), Add (form), Edit (form)
- Keyboard navigation: Up/Down for selection, A/E/D for actions, Esc to close
- Added provider count indicator to header

### App Logic
- Added provider CRUD methods: `add_provider()`, `remove_provider()`, `edit_provider()`, `save_edited_provider()`
- Added keyboard handlers for provider overlay
- Added slash command handlers for provider management

### Slash Commands
- `/provider` - List configured providers
- `/provider-add` - Open provider add overlay
- `/provider-edit <index>` - Edit provider by index
- `/provider-remove <index>` - Remove provider by index

### Keybindings
- `P` - Toggle provider management overlay
- `A` - Add provider (in view mode)
- `E` - Edit selected provider (in view mode)
- `D` - Delete selected provider (in view mode)
- `Up/Down` - Navigate provider list
- `Esc` - Close overlay/cancel edit
- `Enter` - Save provider (in add/edit mode)

## Handoff Notes
- Provider config follows the same structure as adapter/python/providers.json
- Provider types: openai_compatible, anthropic
- Each provider has: name, display_name, type, base_url (optional), api_key_env, default_model
- Changes are persisted to the providers.json file immediately
- Provider count is displayed in the header for quick visibility
- Build passes successfully with all changes integrated
