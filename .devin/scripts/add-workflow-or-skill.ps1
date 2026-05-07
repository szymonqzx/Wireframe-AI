#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Interactive tool for creating new workflows and skills with proper structure.

.DESCRIPTION
    This script implements the add-workflow-or-skill workflow, providing
    interactive template generation for workflows and skills with proper
    structure, validation, and cross-reference management.
#>

param(
    [Parameter()]
    [ValidateSet("workflow", "skill", "auto")]
    [string]$Type = "auto",

    [Parameter()]
    [string]$Name,

    [Parameter()]
    [string]$Description,

    [Parameter()]
    [string]$Overview
)

$ErrorActionPreference = "Stop"

function Test-DirectoryStructure {
    # Ensure .windsurf directory structure exists
    $paths = @(
        ".windsurf/workflows",
        ".windsurf/skills"
    )

    foreach ($path in $paths) {
        if (-not (Test-Path $path)) {
            New-Item -ItemType Directory -Force -Path $path | Out-Null
            Write-Host "✓ Created directory: $path"
        }
    }
}

function Get-ExistingInstructionFiles {
    $existingWorkflows = Get-ChildItem .windsurf/workflows -Filter *.md -ErrorAction SilentlyContinue
    $existingSkills = Get-ChildItem .windsurf/skills -Filter *.md -Recurse -ErrorAction SilentlyContinue

    Write-Host "Found $($existingWorkflows.Count) existing workflows"
    Write-Host "Found $($existingSkills.Count) existing skills"
    Write-Host ""

    return @{
        Workflows = $existingWorkflows
        Skills = $existingSkills
    }
}

function New-WorkflowTemplate {
    param(
        [string]$Name,
        [string]$Description,
        [string]$Overview,
        [string[]]$RelatedSkills = @()
    )

    $skillReferences = if ($RelatedSkills.Count -gt 0) {
        $RelatedSkills | ForEach-Object { "  - `"$_`"" } | Out-String
    } else {
        "  # Add related skills here"
    }

    $template = @"
---
description: $Description
---

# $Name

$Overview

## Overview

[Brief description of what this workflow does]

## When to Use

- ✅ [Condition 1]
- ✅ [Condition 2]
- ✅ [Condition 3]

## When NOT to Use

- ❌ [Condition 1]
- ❌ [Condition 2]

## Pre-flight Checks

\`\`\`powershell
# Add pre-flight checks here
\`\`\`

## Loop Configuration

\`\`\`powershell
\$MAX_ITERS = 10
\$KILLSWITCH = "`$env:USERPROFILE\.workflow-stop"
\$LOGDIR = ".workflow-logs/\$(Get-Date -Format 'yyyyMMddHHmmss')"
New-Item -ItemType Directory -Force -Path \$LOGDIR | Out-Null
\`\`\`

## The Loop

\`\`\`powershell
for (\$i = 1; \$i -le \$MAX_ITERS; \$i++) {
    Write-Host "── Iteration \$i/\$MAX_ITERS ──"

    # 1. Killswitch check
    if (Test-Path \$KILLSWITCH) {
        Write-Host "Killswitch tripped — bailing."
        Remove-Item \$KILLSWITCH
        exit 2
    }

    # 2. Add your workflow logic here

    # 3. Check for completion

    # 4. Not clean — analyze and fix
}
\`\`\`

## Guardrails (Non-Negotiable)

1. **Guardrail 1** - Description
2. **Guardrail 2** - Description
3. **Killswitch file.** Create \`~/.workflow-stop\` to stop the loop.
4. **Iteration cap.** Default 10.

## Related Skills
$skillReferences

## Credits

Combines concepts from:

- **Reference 1**
- **Reference 2**

## Related Workflows
- [List related workflows]
"@

    return $template
}

function New-SkillTemplate {
    param(
        [string]$Name,
        [string]$Description,
        [string]$Topic,
        [string[]]$RelatedWorkflows = @()
    )

    $workflowReferences = if ($RelatedWorkflows.Count -gt 0) {
        $RelatedWorkflows | ForEach-Object { "  - \`"../workflows/$_\`"" } | Out-String
    } else {
        "  # Add related workflows here"
    }

    $template = @"
---
description: $Description
---

# $Name

## When to Use
- [Condition 1]
- [Condition 2]
- [Condition 3]

## Key Patterns

### Pattern 1
\`\`\`rust
// Example code
\`\`\`

### Pattern 2
\`\`\`rust
// Example code
\`\`\`

## Common Pitfalls
- [Pitfall 1]
- [Pitfall 2]

## Best Practices
- [Best practice 1]
- [Best practice 2]

## Related Workflows
$workflowReferences

## Related Skills
- [List related skills]
"@

    return $template
}

function Add-NewInstruction {
    param(
        [string]$Type,
        [string]$Name,
        [string]$Description,
        [string]$Overview
    )

    if ($Type -eq "auto") {
        $typeChoice = Read-Host "Create (W)orkflow or (S)kill?"
        if ($typeChoice -match "^[Ww]$") {
            $Type = "workflow"
        } elseif ($typeChoice -match "^[Ss]$") {
            $Type = "skill"
        } else {
            Write-Error "Invalid choice. Please enter W or S."
            return
        }
    }

    if (-not $Name) {
        $Name = Read-Host "Name (kebab-case, e.g., 'my-new-workflow')"
    }

    if ($Name -notmatch "^[a-z0-9-]+$") {
        Write-Error "Invalid name. Use kebab-case (lowercase letters, numbers, hyphens)."
        return
    }

    if (-not $Description) {
        $Description = Read-Host "Short description (for YAML frontmatter)"
    }

    if (-not $Overview) {
        $Overview = Read-Host "Overview/purpose (can be multi-line)"
    }

    if ($Type -eq "workflow") {
        $filename = ".windsurf/workflows/$Name.md"
        if (Test-Path $filename) {
            Write-Error "Workflow already exists: $filename"
            return
        }

        $template = New-WorkflowTemplate -Name $Name -Description $Description -Overview $Overview
        $template | Out-File -FilePath $filename -Encoding UTF8
        Write-Host "✓ Created workflow: $filename"

    } else {
        $skillDir = Read-Host "Skill directory name (e.g., 'my-topic')"
        $skillPath = ".windsurf/skills/$skillDir"
        $filename = "$skillPath/skill.md"

        if (Test-Path $filename) {
            Write-Error "Skill already exists: $filename"
            return
        }

        New-Item -ItemType Directory -Force -Path $skillPath | Out-Null
        $template = New-SkillTemplate -Name $Name -Description $Description -Topic $Overview
        $template | Out-File -FilePath $filename -Encoding UTF8
        Write-Host "✓ Created skill: $filename"
    }

    # Ask about cross-references
    $addRefs = Read-Host "Add cross-references now? (Y/N)"
    if ($addRefs -match "^[Yy]$") {
        Update-CrossReferences -Type $Type -Name $Name
    }
}

function Update-CrossReferences {
    param(
        [string]$Type,
        [string]$Name
    )

    Write-Host ""
    Write-Host "=== Cross-Reference Update ==="

    if ($Type -eq "workflow") {
        # Ask which skills this workflow should reference
        $skills = Get-ChildItem .windsurf/skills -Directory -ErrorAction SilentlyContinue
        if ($skills.Count -eq 0) {
            Write-Host "No existing skills found."
            return
        }

        Write-Host "Available skills:"
        $skills | ForEach-Object { Write-Host "  - $($_.Name)" }

        $selectedSkills = Read-Host "Enter skill names (comma-separated, or press Enter to skip)"
        if ($selectedSkills) {
            # Update workflow to reference skills
            $workflowFile = ".windsurf/workflows/$Name.md"
            $content = Get-Content $workflowFile -Raw
            $skillList = $selectedSkills -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ } | ForEach-Object { "  - `"$_`"" } | Out-String
            $content = $content -replace "  # Add related skills here", $skillList.Trim()
            $content | Out-File -FilePath $workflowFile -Encoding UTF8
            Write-Host "✓ Updated workflow with skill references"
        }
    } else {
        # Ask which workflows this skill should reference
        $workflows = Get-ChildItem .windsurf/workflows -Filter *.md -ErrorAction SilentlyContinue
        if ($workflows.Count -eq 0) {
            Write-Host "No existing workflows found."
            return
        }

        Write-Host "Available workflows:"
        $workflows | ForEach-Object { Write-Host "  - $($_.Name -replace '.md$', '')" }

        $selectedWorkflows = Read-Host "Enter workflow names (comma-separated, or press Enter to skip)"
        if ($selectedWorkflows) {
            # Find the skill file (it's in a subdirectory)
            $skillFile = Get-ChildItem .windsurf/skills -Filter skill.md -Recurse | Where-Object {
                $content = Get-Content $_ -Raw
                $content -match "# $Name"
            } | Select-Object -First 1

            if ($skillFile) {
                $content = Get-Content $skillFile.FullName -Raw
                $workflowList = $selectedWorkflows -split ',' | ForEach-Object { $_.Trim() } | Where-Object { $_ } | ForEach-Object { "  - \`"../workflows/$_\`"" } | Out-String
                $content = $content -replace "  # Add related workflows here", $workflowList.Trim()
                $content | Out-File -FilePath $skillFile.FullName -Encoding UTF8
                Write-Host "✓ Updated skill with workflow references"
            } else {
                Write-Host "Warning: Could not find skill file for: $Name" -ForegroundColor Yellow
            }
        }
    }
}

# Main execution
Write-Host "=== Instruction File Creator ==="
Write-Host ""

Test-DirectoryStructure
$existing = Get-ExistingInstructionFiles

Add-NewInstruction -Type $Type -Name $Name -Description $Description -Overview $Overview

Write-Host ""
Write-Host "Done!"
