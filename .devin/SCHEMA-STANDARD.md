# Schema Standard for Wireframe-AI

Consistent schema for skills, agents, and rules, aligned with [Devin CLI documentation](https://cli.devin.ai/docs/extensibility).

## Skill Schema

```yaml
---
name: skill-name
description: Brief one-line description
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
  - exec
triggers:
  - user
  - model
---
```

**Required:** `name`, `description`  
**Optional:** `allowed-tools`, `triggers`, `model`, `subagent`, `permissions`  
**Max lines:** 300 (split into SKILL-advanced.md if needed)

## Agent Schema

```yaml
---
name: agent-name
description: Brief one-line description
model: sonnet
allowed-tools:
  - read
  - grep
  - glob
  - exec
permissions:
  allow:
    - Exec(cargo check)
  deny:
    - write
    - edit
---
```

**Required:** `name`, `description`  
**Optional:** `model`, `allowed-tools`, `permissions`  
**Max lines:** 300

## Rule Schema

```yaml
---
paths:
  - "**/*.rs"
---
```

**Required:** `paths` (glob pattern)  
**Max lines:** 300 (split into rule-advanced.md if needed)

## Hooks Schema

```json
{
  "version": "1.0",
  "hooks": {
    "SessionStart": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "bash .devin/hooks/session-start.sh",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

**Events:** SessionStart, PreToolUse, PostToolUse, PermissionRequest  
**Types:** command, prompt

## File Locations

- Skills: `.devin/skills/<name>/SKILL.md`
- Agents: `.devin/agents/<name>/AGENT.md`
- Rules: `.devin/rules/<name>.md`
- Hooks: `.devin/hooks.v1.json` and `.devin/hooks/*.sh`