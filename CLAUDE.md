# petekp/agent-skills

Agent skills monorepo for the Claude Code marketplace. Each skill is a separate installable plugin.

## Structure

- `skills/{skill-name}/` — Individual skills, each with `SKILL.md` and optional `references/`, `examples/`, `scripts/`
- `.claude-plugin/marketplace.json` — Marketplace configuration with per-skill plugin entries

## Adding a Skill

### 1. Create the skill directory

Create `skills/{skill-name}/SKILL.md` with required frontmatter:
```yaml
---
name: skill-name
description: When to use this skill (max 1024 chars)
license: MIT
metadata:
  author: petekp
  version: "0.1.0"
---
```

Add optional subdirectories: `references/`, `examples/`, `scripts/`

### 2. Register as a separate plugin in marketplace.json

Each skill must be its own plugin entry in `.claude-plugin/marketplace.json`. Add a new object to the `plugins` array:

```json
{
  "name": "skill-name",
  "description": "Brief description for marketplace listing",
  "source": "./",
  "strict": false,
  "skills": ["./skills/skill-name"]
}
```

**Important:** The `name` must match the skill directory name exactly.

## Skill Spec

- `name`: lowercase, hyphens only, max 64 chars, must match directory name
- `description`: triggers + purpose, max 1024 chars
- Keep main SKILL.md under 500 lines; use reference files for detailed content

## Conventions

- Skills are invocable as `/skill-name` in Claude Code
- Reference files use `references/{topic}.md` naming
- Examples use `examples/{example-name}.{ext}` naming
