#!/usr/bin/env bash
#
# newskill - Create Claude Code skills interactively
#
# Uses Claude Code's /skill-creator to generate production-ready skills
# with proper SKILL.md, asking clarifying questions before generating.
#
# PREREQUISITES:
#   - Claude Code CLI (https://claude.ai/code)
#   - /skill-creator skill (auto-installed if missing)
#   - jq (optional, for marketplace registration)
#   - npx (for installing skill-creator if needed)
#
# INSTALLATION:
#   1. Download this script:
#      curl -o ~/.local/bin/newskill https://gist.githubusercontent.com/YOUR_GIST_URL/raw/newskill.sh
#      chmod +x ~/.local/bin/newskill
#
#   2. Or clone and alias:
#      alias newskill="/path/to/newskill.sh"
#
# CONFIGURATION:
#   Set SKILLS_DIR to your skills output directory:
#      export SKILLS_DIR="$HOME/.claude/skills"
#
#   Optionally set MARKETPLACE_JSON for auto-registration:
#      export MARKETPLACE_JSON="/path/to/.claude-plugin/marketplace.json"
#
# USAGE:
#   newskill                    # prompts for name and description
#   newskill my-skill           # prompts for description only
#   newskill --help
#
# EXAMPLES:
#   $ newskill
#   Skill name (lowercase-with-hyphens): code-review
#   Description (one line): Review code for best practices and security issues
#
#   Skill: code-review
#   Description: Review code for best practices and security issues
#   Output: /Users/you/.claude/skills/code-review/
#
#   Launch Claude Code to create this skill? [Y/n] y
#
#   Launching Claude Code...
#   # Claude asks clarifying questions, then generates the skill
#
set -euo pipefail

# Configuration - override with environment variables
SKILLS_DIR="${SKILLS_DIR:-$HOME/.claude/skills}"
MARKETPLACE_JSON="${MARKETPLACE_JSON:-}"

usage() {
    cat <<EOF
Usage: newskill [skill-name]

Creates a skill using Claude Code's /skill-creator skill.

Options:
  -h, --help    Show this help message

Environment variables:
  SKILLS_DIR        Output directory for skills (default: ~/.claude/skills)
  MARKETPLACE_JSON  Path to marketplace.json for auto-registration (optional)

Examples:
  newskill                 # prompts for name and description
  newskill code-review     # prompts for description only
EOF
    exit 0
}

cleanup_backup() {
    local backup_dir="$1"
    if [[ -d "$backup_dir" ]]; then
        rm -rf "$backup_dir"
    fi
}

restore_backup() {
    local skill_dir="$1"
    local backup_dir="$2"
    if [[ -d "$backup_dir" ]]; then
        rm -rf "$skill_dir" 2>/dev/null || true
        mv "$backup_dir" "$skill_dir"
        echo "Restored previous version of skill."
    fi
}

extract_description() {
    local skill_md="$1"
    if [[ -f "$skill_md" ]]; then
        sed -n '/^---$/,/^---$/p' "$skill_md" | grep -E "^description:" | sed 's/^description: *//' | sed 's/^["'"'"']//' | sed 's/["'"'"']$//'
    fi
}

check_skill_creator() {
    # Check common locations for skill-creator
    local locations=(
        "$HOME/.claude/skills/skill-creator"
        "$HOME/.claude/plugins/local/skill-creator"
        "$HOME/.claude/plugins/marketplaces/anthropic-agent-skills/skills/skill-creator"
    )

    for loc in "${locations[@]}"; do
        if [[ -d "$loc" ]] && [[ -f "$loc/SKILL.md" ]]; then
            return 0
        fi
    done

    # Also check plugin cache (glob for any version)
    if compgen -G "$HOME/.claude/plugins/cache/*/compound-engineering/*/skills/skill-creator/SKILL.md" > /dev/null 2>&1; then
        return 0
    fi
    if compgen -G "$HOME/.claude/plugins/marketplaces/*/plugins/compound-engineering/skills/skill-creator/SKILL.md" > /dev/null 2>&1; then
        return 0
    fi

    return 1
}

install_skill_creator() {
    echo "The /skill-creator skill is required but not installed."
    echo ""
    printf "Install it now? [Y/n] "
    read -r -n 1 REPLY
    echo
    if [[ "$REPLY" =~ ^[Nn]$ ]]; then
        echo "Cannot continue without /skill-creator. Aborted."
        exit 1
    fi

    echo ""
    echo "Installing skill-creator..."
    if ! npx @anthropic-ai/claude-code-skills add https://github.com/anthropics/skills --skill skill-creator; then
        echo ""
        echo "Failed to install skill-creator. Please install manually:"
        echo "  npx @anthropic-ai/claude-code-skills add https://github.com/anthropics/skills --skill skill-creator"
        exit 1
    fi
    echo ""
    echo "skill-creator installed successfully."
    echo ""
}

if [[ "${1:-}" == "-h" ]] || [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "help" ]]; then
    usage
fi

# Check for claude CLI
if ! command -v claude &> /dev/null; then
    echo "Error: Claude Code CLI not found."
    echo "Install it from: https://claude.ai/code"
    exit 1
fi

# Check for skill-creator dependency
if ! check_skill_creator; then
    install_skill_creator
fi

# Ensure skills directory exists
mkdir -p "$SKILLS_DIR"

# Get skill name - from arg or prompt
if [[ $# -ge 1 ]]; then
    SKILL_NAME="$1"
else
    printf "Skill name (lowercase-with-hyphens): "
    read -r SKILL_NAME
fi

if [[ -z "$SKILL_NAME" ]] || [[ ! "$SKILL_NAME" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]]; then
    echo "Invalid skill name. Use lowercase-with-hyphens."
    exit 1
fi

SKILL_DIR="$SKILLS_DIR/$SKILL_NAME"
BACKUP_DIR="$SKILLS_DIR/.${SKILL_NAME}.bak"
REPLACING=false

if [[ -d "$SKILL_DIR" ]]; then
    printf "Skill '$SKILL_NAME' already exists. Replace it? [y/N] "
    read -r -n 1 REPLY
    echo
    if [[ "$REPLY" =~ ^[Yy]$ ]]; then
        REPLACING=true
    else
        echo "Aborted."
        exit 0
    fi
fi

printf "Description (one line): "
read -r DESCRIPTION
DESCRIPTION="${DESCRIPTION:-A skill that does X}"

# Ask about marketplace registration if available
REGISTER_MARKETPLACE=false
if [[ -n "$MARKETPLACE_JSON" ]] && [[ -f "$MARKETPLACE_JSON" ]] && command -v jq &> /dev/null; then
    printf "Register in marketplace.json? [Y/n] "
    read -r -n 1 REPLY
    echo
    if [[ ! "$REPLY" =~ ^[Nn]$ ]]; then
        REGISTER_MARKETPLACE=true
    fi
fi

echo ""
echo "Skill: $SKILL_NAME"
echo "Description: $DESCRIPTION"
echo "Output: $SKILL_DIR/"
if [[ "$REPLACING" == true ]]; then
    echo "Mode: Replace existing"
fi
if [[ "$REGISTER_MARKETPLACE" == true ]]; then
    echo "Register: Yes (marketplace.json)"
fi
echo ""
printf "Launch Claude Code to create this skill? [Y/n] "
read -r -n 1 REPLY
echo
if [[ "$REPLY" =~ ^[Nn]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Backup existing skill before replacing
if [[ "$REPLACING" == true ]]; then
    cleanup_backup "$BACKUP_DIR"
    mv "$SKILL_DIR" "$BACKUP_DIR"
    # Remove existing marketplace entry if re-registering
    if [[ "$REGISTER_MARKETPLACE" == true ]]; then
        jq --arg name "$SKILL_NAME" 'del(.plugins[] | select(.name == $name))' \
            "$MARKETPLACE_JSON" > "$MARKETPLACE_JSON.tmp" && mv "$MARKETPLACE_JSON.tmp" "$MARKETPLACE_JSON"
    fi
fi

echo ""
echo "Launching Claude Code..."
echo ""

# Invoke Claude Code with skill-creator to generate the actual skill
claude "Use /skill-creator to create a new skill with:
- Name: $SKILL_NAME
- Description: $DESCRIPTION
- Output directory: $SKILL_DIR/

Follow the skill-creator workflow. Ask me clarifying questions about the skill's purpose and usage patterns before generating. Create a complete, production-ready SKILL.md - not just a template with TODOs.

IMPORTANT: Do NOT run package_skill.py or create a .skill package. Just create the skill directory and files."

# Check if skill was created
SKILL_MD="$SKILL_DIR/SKILL.md"

if [[ -f "$SKILL_MD" ]]; then
    # Success - clean up backup
    cleanup_backup "$BACKUP_DIR"

    # Extract description from SKILL.md for marketplace (sync with what Claude wrote)
    FINAL_DESC=$(extract_description "$SKILL_MD")
    if [[ -z "$FINAL_DESC" ]]; then
        FINAL_DESC="$DESCRIPTION"
    fi

    # Register in marketplace if user opted in
    REGISTERED=""
    if [[ "$REGISTER_MARKETPLACE" == true ]]; then
        jq --arg name "$SKILL_NAME" \
           --arg desc "$FINAL_DESC" \
           '.plugins += [{
             "name": $name,
             "description": $desc,
             "source": "./",
             "strict": false,
             "skills": ["./skills/" + $name]
           }]' "$MARKETPLACE_JSON" > "$MARKETPLACE_JSON.tmp" && mv "$MARKETPLACE_JSON.tmp" "$MARKETPLACE_JSON"
        REGISTERED=" (registered in marketplace)"
    fi

    echo ""
    echo "Created: $SKILL_NAME$REGISTERED"
    echo "  $SKILL_MD"
else
    # Failed - restore backup if we were replacing
    if [[ "$REPLACING" == true ]]; then
        restore_backup "$SKILL_DIR" "$BACKUP_DIR"
        # Re-add marketplace entry if we had removed it
        if [[ "$REGISTER_MARKETPLACE" == true ]]; then
            OLD_DESC=$(extract_description "$SKILL_DIR/SKILL.md")
            OLD_DESC="${OLD_DESC:-$DESCRIPTION}"
            jq --arg name "$SKILL_NAME" \
               --arg desc "$OLD_DESC" \
               '.plugins += [{
                 "name": $name,
                 "description": $desc,
                 "source": "./",
                 "strict": false,
                 "skills": ["./skills/" + $name]
               }]' "$MARKETPLACE_JSON" > "$MARKETPLACE_JSON.tmp" && mv "$MARKETPLACE_JSON.tmp" "$MARKETPLACE_JSON"
        fi
    fi
    echo ""
    echo "Skill was not created. No changes made."
fi
