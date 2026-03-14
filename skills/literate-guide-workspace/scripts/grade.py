#!/usr/bin/env python3
"""Grade a literate guide against programmatic assertions."""

import json
import re
import sys
from pathlib import Path


def grade(guide_path: str) -> dict:
    text = Path(guide_path).read_text()

    results = []

    # 1. has_numbered_sections: Uses §N notation
    section_headings = re.findall(r'§\d+', text)
    results.append({
        "text": "has_numbered_sections",
        "passed": len(section_headings) >= 3,
        "evidence": f"Found {len(section_headings)} §N references: {section_headings[:8]}"
    })

    # 2. has_cross_references: §N refs in prose (not in headings)
    # Remove headings, then look for §N
    lines = text.split('\n')
    prose_lines = [l for l in lines if not l.startswith('#')]
    prose_text = '\n'.join(prose_lines)
    cross_refs = re.findall(r'§\d+', prose_text)
    # Filter to refs that appear in body text, not just the section heading itself
    results.append({
        "text": "has_cross_references",
        "passed": len(cross_refs) >= 2,
        "evidence": f"Found {len(cross_refs)} cross-references in prose body"
    })

    # 3. has_code_excerpts_with_paths: file paths near code blocks
    code_blocks = re.findall(r'```[\s\S]*?```', text)
    blocks_with_paths = 0
    for block in code_blocks:
        # Check if there's a file path pattern in or near the block
        if re.search(r'[\w/]+\.\w+:\d+', block) or re.search(r'//\s*[\w/]+\.\w+', block):
            blocks_with_paths += 1
    # Also check lines just before code blocks
    for i, line in enumerate(lines):
        if line.startswith('```') and i > 0:
            prev = lines[i-1] if i > 0 else ''
            prev2 = lines[i-2] if i > 1 else ''
            if re.search(r'[\w/]+\.\w+:\d+', prev) or re.search(r'[\w/]+\.\w+:\d+', prev2):
                blocks_with_paths += 1
            if re.search(r'`[\w/]+\.\w+`', prev) or re.search(r'`[\w/]+\.\w+`', prev2):
                blocks_with_paths += 1
    results.append({
        "text": "has_code_excerpts_with_paths",
        "passed": blocks_with_paths >= 2,
        "evidence": f"Found ~{blocks_with_paths} code blocks with file path annotations out of {len(code_blocks)} total blocks"
    })

    # 4. has_design_reasoning: "why" / "because" / "instead of" / "tradeoff" / "alternative"
    reasoning_patterns = [
        r'\bwhy\b.*\b(chose|use|designed|picked|opted|decided)\b',
        r'\bbecause\b',
        r'\binstead of\b',
        r'\btradeoff\b',
        r'\btrade-off\b',
        r'\balternative\b',
        r'\bdesign decision\b',
        r'\bwe chose\b',
        r'\bthe reason\b',
        r'\brather than\b',
    ]
    reasoning_count = sum(
        len(re.findall(p, text, re.IGNORECASE))
        for p in reasoning_patterns
    )
    results.append({
        "text": "has_design_reasoning",
        "passed": reasoning_count >= 3,
        "evidence": f"Found {reasoning_count} design reasoning indicators"
    })

    # 5. has_mermaid_diagrams
    mermaid_blocks = re.findall(r'```mermaid', text)
    results.append({
        "text": "has_mermaid_diagrams",
        "passed": len(mermaid_blocks) >= 1,
        "evidence": f"Found {len(mermaid_blocks)} Mermaid diagram(s)"
    })

    # 6. has_narrative_transitions: forward/backward references in prose
    transition_patterns = [
        r'\b(we (can )?now (turn|look|move|see|examine))\b',
        r'\b(as we (saw|discussed|noted|established|explored))\b',
        r'\b(with (this|that) (in place|established|understood))\b',
        r'\b(this connects (to|back|with))\b',
        r'\b(building on)\b',
        r'\b(before we)\b',
        r'\b(having (seen|established|understood))\b',
        r'\b(let\'s (now|turn|move))\b',
        r'\b(next|previously)\b',
        r'\b(recall (that|from))\b',
    ]
    transition_count = sum(
        len(re.findall(p, text, re.IGNORECASE))
        for p in transition_patterns
    )
    results.append({
        "text": "has_narrative_transitions",
        "passed": transition_count >= 3,
        "evidence": f"Found {transition_count} narrative transition phrases"
    })

    return {
        "guide_path": guide_path,
        "expectations": results,
        "pass_rate": sum(1 for r in results if r["passed"]) / len(results)
    }


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: grade.py <guide_path> <output_path>")
        sys.exit(1)

    guide_path = sys.argv[1]
    output_path = sys.argv[2]

    result = grade(guide_path)
    Path(output_path).write_text(json.dumps(result, indent=2))
    print(f"Pass rate: {result['pass_rate']:.0%}")
    for r in result["expectations"]:
        status = "PASS" if r["passed"] else "FAIL"
        print(f"  [{status}] {r['text']}: {r['evidence']}")
