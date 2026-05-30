#!/usr/bin/env python3
"""
Hook #13 — Trivy Scanner (PreToolUse: Bash)
Kura Project

Severidades:
  CRITICAL → bloqueia
  HIGH     → bloqueia
  MEDIUM   → bloqueia
  LOW      → aviso, não bloqueia

Requer: trivy instalado (brew install trivy)
"""
import json
import os
import re
import subprocess
import sys

BLOCK_ON = {'CRITICAL', 'HIGH', 'MEDIUM'}
WARN_ON  = {'LOW'}
TIMEOUT  = 120

try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

command = data.get("tool_input", {}).get("command", "")
cwd     = data.get("cwd", os.getcwd())

TRIGGER_PATTERNS = [
    r'\bdocker\s+build\b', r'\bdocker\s+push\b', r'\bmix\s+release\b',
    r'\bdeploy\.sh\b', r'\bship\.sh\b', r'\bfly\s+deploy\b',
    r'ssh\b.*oracle', r'rsync\b.*kura',
]

if not any(re.search(p, command, re.IGNORECASE) for p in TRIGGER_PATTERNS):
    sys.exit(0)

def find_project_root(start):
    current = os.path.abspath(start)
    while current != '/':
        if os.path.exists(os.path.join(current, 'mix.exs')):
            return current
        current = os.path.dirname(current)
    return os.path.abspath(start)

project_root = find_project_root(cwd)

try:
    subprocess.run(['trivy', '--version'], capture_output=True, timeout=5)
except FileNotFoundError:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "allow",
        "additionalContext": "TRIVY: não instalado — scan pulado.\nInstale: brew install trivy"}}))
    sys.exit(0)
except Exception:
    sys.exit(0)

try:
    result = subprocess.run(
        ['trivy', 'fs', '--format', 'json', '--severity', 'CRITICAL,HIGH,MEDIUM,LOW',
         '--no-progress', '--skip-dirs', '_build,deps,.git,node_modules', '.'],
        cwd=project_root, capture_output=True, text=True, timeout=TIMEOUT,
    )

    try:
        trivy_data = json.loads(result.stdout) if result.stdout.strip() else {}
    except json.JSONDecodeError:
        trivy_data = {}

    by_sev = {'CRITICAL': [], 'HIGH': [], 'MEDIUM': [], 'LOW': []}

    for res in trivy_data.get('Results', []):
        for vuln in res.get('Vulnerabilities', []) or []:
            sev   = vuln.get('Severity', 'UNKNOWN').upper()
            vid   = vuln.get('VulnerabilityID', '')
            pkg   = vuln.get('PkgName', '')
            ver   = vuln.get('InstalledVersion', '')
            fixed = vuln.get('FixedVersion', '')
            title = vuln.get('Title', '')[:60]
            entry = f"{vid} [{pkg}@{ver}]{' → fix: ' + fixed if fixed else ''} — {title}"
            if sev in by_sev:
                by_sev[sev].append(entry)

    blocking = by_sev['CRITICAL'] + by_sev['HIGH'] + by_sev['MEDIUM']
    low_vulns = by_sev['LOW']

    counts = {s: len(v) for s, v in by_sev.items()}
    total_blocking = counts['CRITICAL'] + counts['HIGH'] + counts['MEDIUM']

    if blocking:
        sections = []
        for sev in ('CRITICAL', 'HIGH', 'MEDIUM'):
            if by_sev[sev]:
                lines = [f"  • {e}" for e in by_sev[sev][:8]]
                if len(by_sev[sev]) > 8:
                    lines.append(f"  ... e mais {len(by_sev[sev]) - 8}")
                sections.append(f"{sev} ({counts[sev]}):\n" + "\n".join(lines))

        summary = ", ".join(f"{counts[s]} {s}" for s in ('CRITICAL','HIGH','MEDIUM') if counts[s])
        low_note = f"\n\n+ {counts['LOW']} LOW (não bloqueante)." if counts['LOW'] else ""

        reason = (
            f"TRIVY SCANNER: Deploy bloqueado — {summary}.\n\n"
            + "\n\n".join(sections)
            + low_note
            + "\n\nCorrija antes de fazer deploy.\n"
            f"Detalhes: trivy fs . --severity CRITICAL,HIGH,MEDIUM"
        )
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "deny", "permissionDecisionReason": reason}}))

    elif low_vulns:
        lines   = [f"  • {e}" for e in low_vulns[:5]]
        context = (
            f"TRIVY: {counts['LOW']} vulnerabilidade(s) LOW — deploy permitido.\n"
            + "\n".join(lines)
            + "\n\nConsidere corrigir em próxima release."
        )
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "allow", "additionalContext": context}}))

    else:
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": "[trivy ✓] Sem vulnerabilidades. Deploy liberado."}}))

except subprocess.TimeoutExpired:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": f"TRIVY: timeout ({TIMEOUT}s) — rode manualmente: trivy fs ."}}))
except Exception as e:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow", "additionalContext": f"TRIVY: erro — {e}"}}))

sys.exit(0)
