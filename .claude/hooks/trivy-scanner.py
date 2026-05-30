#!/usr/bin/env python3
"""
Hook #13 — Trivy Scanner (PreToolUse: Bash)
Kura Project

Roda trivy fs antes de deploy ou build Docker para detectar
vulnerabilidades em dependências (mix.lock, Elixir packages, etc.)

Severidades:
  CRITICAL → bloqueia sempre
  HIGH     → bloqueia (configurável para avisar)
  MEDIUM   → bloqueia
  LOW      → aviso, não bloqueia

Requer: trivy instalado (brew install trivy)
"""
import json
import os
import re
import subprocess
import sys

# ── Configuração ──────────────────────────────────────────────────────────────
BLOCK_ON    = {"CRITICAL", "HIGH", "MEDIUM"}   # Severidades que bloqueiam
WARN_ON     = {"LOW"}             # Severidades que apenas avisam
TIMEOUT     = 120                    # segundos

# ── Input ─────────────────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

command = data.get("tool_input", {}).get("command", "")
cwd     = data.get("cwd", os.getcwd())

# ── Detecta comandos que exigem scan ──────────────────────────────────────────
TRIGGER_PATTERNS = [
    r'\bdocker\s+build\b',
    r'\bdocker\s+push\b',
    r'\bmix\s+release\b',
    r'\bdeploy\.sh\b',
    r'\bship\.sh\b',
    r'\bfly\s+deploy\b',
    r'ssh\b.*oracle',
    r'rsync\b.*kura',
]

if not any(re.search(p, command, re.IGNORECASE) for p in TRIGGER_PATTERNS):
    sys.exit(0)

# ── Encontra root do projeto ───────────────────────────────────────────────────
def find_project_root(start):
    current = os.path.abspath(start)
    while current != '/':
        if os.path.exists(os.path.join(current, 'mix.exs')):
            return current
        current = os.path.dirname(current)
    return os.path.abspath(start)

project_root = find_project_root(cwd)

# ── Verifica se trivy está instalado ─────────────────────────────────────────
try:
    subprocess.run(['trivy', '--version'], capture_output=True, timeout=5)
except FileNotFoundError:
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": (
                "TRIVY: não instalado — scan de segurança pulado.\n"
                "Instale: brew install trivy\n"
                "Docs: https://trivy.dev"
            ),
        }
    }))
    sys.exit(0)
except Exception:
    sys.exit(0)

# ── Roda trivy fs ─────────────────────────────────────────────────────────────
try:
    result = subprocess.run(
        [
            'trivy', 'fs',
            '--format', 'json',
            '--severity', 'CRITICAL,HIGH,MEDIUM,LOW',
            '--no-progress',
            '--skip-dirs', '_build,deps,.git,node_modules',
            '.',
        ],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=TIMEOUT,
    )

    # Trivy retorna 0 (sem vuln) ou 1 (vulnerabilidades encontradas)
    try:
        trivy_data = json.loads(result.stdout) if result.stdout.strip() else {}
    except json.JSONDecodeError:
        trivy_data = {}

    # ── Agrega resultados ─────────────────────────────────────────────────────
    by_severity = {'CRITICAL': [], 'HIGH': [], 'MEDIUM': [], 'LOW': []}
    results     = trivy_data.get('Results', [])

    for res in results:
        target = res.get('Target', '')
        for vuln in res.get('Vulnerabilities', []) or []:
            sev   = vuln.get('Severity', 'UNKNOWN').upper()
            vid   = vuln.get('VulnerabilityID', '')
            pkg   = vuln.get('PkgName', '')
            ver   = vuln.get('InstalledVersion', '')
            fixed = vuln.get('FixedVersion', '')
            title = vuln.get('Title', '')[:60]

            entry = f"{vid} [{pkg}@{ver}]{' → fix: ' + fixed if fixed else ''} — {title}"
            if sev in by_severity:
                by_severity[sev].append((target, entry))

    # ── Monta relatório ───────────────────────────────────────────────────────
    blocking_vulns = by_severity['CRITICAL'] + by_severity['HIGH'] + by_severity['MEDIUM']
    warning_vulns  = by_severity['LOW']

    total_critical = len(by_severity['CRITICAL'])
    total_high     = len(by_severity['HIGH'])
    total_medium   = len(by_severity['MEDIUM'])

    if blocking_vulns:
        sections = []

        if by_severity['CRITICAL']:
            lines = [f"  • {e}" for _, e in by_severity['CRITICAL'][:10]]
            if len(by_severity['CRITICAL']) > 10:
                lines.append(f"  ... e mais {len(by_severity['CRITICAL']) - 10}")
            sections.append("CRITICAL (" + str(total_critical) + "):\n" + "\n".join(lines))

        if by_severity['HIGH']:
            lines = [f"  • {e}" for _, e in by_severity['HIGH'][:10]]
            if len(by_severity['HIGH']) > 10:
                lines.append(f"  ... e mais {len(by_severity['HIGH']) - 10}")
            sections.append("HIGH (" + str(total_high) + "):\n" + "\n".join(lines))

        summary = f"{total_critical} CRITICAL, {total_high} HIGH"
        if total_medium:
            summary += f", {total_medium} MEDIUM"

        reason = (
            f"TRIVY SCANNER: Deploy bloqueado — {summary}.\n\n"
            + "\n\n".join(sections)
            + "\n\nCorrija as vulnerabilidades antes de fazer deploy.\n"
            f"Detalhes: trivy fs . --severity CRITICAL,HIGH,MEDIUM"
        )
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        }))

    elif warning_vulns:
        total_low = len(by_severity['LOW'])
        lines   = [f"  • {e}" for _, e in warning_vulns[:5]]
        context = (
            f"TRIVY: {total_low} vulnerabilidade(s) LOW encontrada(s) — deploy permitido.\n"
            + "\n".join(lines)
            + "\n\nMonitore e corrija quando possível."
        )
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "additionalContext": context,
            }
        }))

    else:
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "additionalContext": "[trivy ✓] Sem vulnerabilidades CRITICAL/HIGH/MEDIUM. Deploy liberado.",
            }
        }))

except subprocess.TimeoutExpired:
    # Timeout — permite mas avisa
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": f"TRIVY: timeout ({TIMEOUT}s) — scan pulado. Rode manualmente: trivy fs .",
        }
    }))

except Exception as e:
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": f"TRIVY: erro ao executar — {e}",
        }
    }))

sys.exit(0)
