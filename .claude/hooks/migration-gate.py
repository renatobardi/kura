#!/usr/bin/env python3
"""
Hook #10 — Migration Gate (PreToolUse: Bash)
Kura Project

Verifica migrations do SurrealDB antes de qualquer deploy ao Oracle ARM.
Bloqueia deploy se houver migrations pendentes nao aplicadas.

Detecta comandos de deploy:
  - mix release + scp/rsync/ssh para o ARM
  - scripts de deploy (deploy.sh, ship.sh, push.sh)
  - mix release diretamente
  - docker build/push seguido de deploy

Estrategia de verificacao:
  1. Lê o SCHEMA_VERSION atual do projeto (priv/surreal/schema_meta.ex ou similar)
  2. Lista arquivos em priv/surreal/migrations/
  3. Compara com a versao aplicada (via arquivo de estado local)
  4. Se houver pendentes → bloqueia com lista das migrations
"""
import json
import os
import re
import subprocess
import sys
from pathlib import Path

# ── Input ─────────────────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

command = data.get("tool_input", {}).get("command", "")
cwd     = data.get("cwd", os.getcwd())

# ── Detecta comandos de deploy ─────────────────────────────────────────────────
DEPLOY_PATTERNS = [
    r'\bmix\s+release\b',
    r'\bdeploy\.sh\b',
    r'\bship\.sh\b',
    r'\bpush\.sh\b',
    r'\bscp\b.*\.tar\.gz',
    r'\brsync\b.*kura',
    r'ssh\b.*oracle',
    r'ssh\b.*arm',
    r'\bdocker\s+push\b',
    r'\bfly\s+deploy\b',
]

is_deploy = any(re.search(p, command, re.IGNORECASE) for p in DEPLOY_PATTERNS)
if not is_deploy:
    sys.exit(0)

# ── Encontra root do projeto ───────────────────────────────────────────────────
def find_project_root(start):
    current = os.path.abspath(start)
    while current != '/':
        if os.path.exists(os.path.join(current, 'mix.exs')):
            return current
        current = os.path.dirname(current)
    return None

project_root = find_project_root(cwd)
if not project_root:
    sys.exit(0)

# ── Verifica migrations SurrealDB ─────────────────────────────────────────────
migrations_dir = os.path.join(project_root, 'priv', 'surreal', 'migrations')
state_file     = os.path.join(project_root, '.claude', '.migration_state')

issues = []
pending_migrations = []

if not os.path.exists(migrations_dir):
    # Ainda sem migrations — ok para projetos novos
    sys.exit(0)

# Lista todas as migrations em ordem
all_migrations = sorted([
    f for f in os.listdir(migrations_dir)
    if re.match(r'^\d{14}_.*\.(surql|sql|exs)$', f)
])

if not all_migrations:
    sys.exit(0)

# Lê estado de migrations aplicadas
applied = set()
if os.path.exists(state_file):
    with open(state_file) as f:
        applied = set(line.strip() for line in f if line.strip())

# Migrations pendentes
pending_migrations = [m for m in all_migrations if m not in applied]

# ── Verifica também se há arquivos de migration não versionados ───────────────
try:
    result = subprocess.run(
        ['git', 'status', '--porcelain', migrations_dir],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=10,
    )
    untracked = [
        l.strip() for l in result.stdout.splitlines()
        if l.strip() and (l.startswith('??') or l.startswith('A '))
    ]
    if untracked:
        issues.append(f"Migrations nao commitadas:\n  " + "\n  ".join(untracked))
except Exception:
    pass

# ── Resultado ─────────────────────────────────────────────────────────────────
if pending_migrations or issues:
    sections = []

    if pending_migrations:
        mig_list = "\n  ".join(pending_migrations)
        sections.append(
            f"{len(pending_migrations)} migration(s) pendente(s):\n  {mig_list}\n\n"
            f"Aplique antes do deploy:\n"
            f"  mix kura.migrate   (ou o comando equivalente do projeto)"
        )

    if issues:
        sections.append("\n".join(issues))

    reason = (
        "MIGRATION GATE: Deploy bloqueado — migrations pendentes.\n\n"
        + "\n\n".join(sections)
        + "\n\nApos aplicar as migrations, atualize .claude/.migration_state."
    )

    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
else:
    # Tudo ok
    total = len(all_migrations)
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": f"Migration gate: {total} migration(s) aplicada(s). Deploy liberado.",
        }
    }))

sys.exit(0)
