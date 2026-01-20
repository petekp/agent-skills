# petekp/skills

Agent skills monorepo for the Claude Code marketplace.

## Structure

- `skills/` — Individual skills, each with `SKILL.md` and optional `references/`, `examples/`, `scripts/`
- `.claude-plugin/marketplace.json` — Marketplace configuration

## Adding a Skill

1. Create `skills/{skill-name}/SKILL.md` with required frontmatter:
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
2. Add optional `references/`, `examples/`, `scripts/` subdirectories
3. Update `.claude-plugin/marketplace.json` plugins array

## Skill Spec

- `name`: lowercase, hyphens only, max 64 chars, must match directory name
- `description`: triggers + purpose, max 1024 chars
- Keep main SKILL.md under 500 lines; use reference files for detailed content

## Conventions

- Skills are invocable as `/skill-name` in Claude Code
- Reference files use `references/{topic}.md` naming
- Examples use `examples/{example-name}.{ext}` naming
