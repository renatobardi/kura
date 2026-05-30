#!/usr/bin/env python3
"""
Hook #15 — PR Standards (PreToolUse: Bash)
Kura Project

Enforça padrões de título e body em `gh pr create`.

Título deve seguir Conventional Commits:
  type(scope): descrição  OU  type: descrição
  Tipos válidos: feat, fix, docs, style, refactor, perf, test, chore, ci, revert

Body deve conter pelo menos:
  ## What  (o quê foi feito)
  ## Why   (motivação)
  ## How   (como testar / validar)

Bloqueia o PR se qualquer padrão não for respeitado.
"""
import json
import os
import re
import sys

# ── Input ─────────────────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

command = data.get("tool_input", {}).get("command", "")
cwd     = data.get("cwd", os.getcwd())

if not re.search(r'\bgh\s+pr\s+create\b', command):
    sys.exit(0)

# ── Padrões aceitos ────────────────────────────────────────────────────────────
VALID_TYPES = r'feat|fix|docs|style|refactor|perf|test|chore|ci|revert'
TITLE_RE    = re.compile(
    rf'^({VALID_TYPES})(\([a-z0-9_/-]+\))?!?: .{{3,}}$'
)

REQUIRED_SECTIONS = ['## What', '## Why', '## How']

# ── Extrai título e body do comando ───────────────────────────────────────────
def extract_flag(cmd, flag_short, flag_long):
    """Extrai valor de --flag 'valor' ou --flag='valor'."""
    pattern = rf'(?:{flag_short}|{flag_long})[\s=]+(?:\'([^\']*)\'|"([^"]*)")'
    m = re.search(pattern, cmd)
    if m:
        return m.group(1) or m.group(2) or ''
    return None

title = extract_flag(command, '-t', '--title')
body  = extract_flag(command, '-b', '--body')

# Tenta ler body de arquivo (--body-file)
body_file_match = re.search(r'--body-file[\s=]+([^\s"\']+)', command)
if body_file_match and not body:
    bf = body_file_match.group(1)
    if not os.path.isabs(bf):
        bf = os.path.join(cwd, bf)
    try:
        with open(bf) as f:
            body = f.read()
    except Exception:
        body = ''

errors = []

# ── Valida título ──────────────────────────────────────────────────────────────
if title is None:
    errors.append(
        "Título não encontrado no comando.\n"
        "Use: gh pr create --title 'type(scope): descrição' ...\n"
        f"Tipos válidos: {VALID_TYPES.replace('|', ', ')}"
    )
elif not TITLE_RE.match(title.strip()):
    errors.append(
        f"Título inválido: \"{title}\"\n"
        "Formato esperado: type(scope): descrição  OU  type: descrição\n"
        f"Tipos válidos: {VALID_TYPES.replace('|', ', ')}\n\n"
        "Exemplos corretos:\n"
        "  feat(auth): add JWT refresh token\n"
        "  fix: correct vault search pagination\n"
        "  chore(deps): upgrade elixir to 1.17"
    )

# ── Valida body ────────────────────────────────────────────────────────────────
if body is None:
    errors.append(
        "Body do PR não encontrado.\n"
        "Use: --body '...' ou --body-file path/to/body.md\n\n"
        "Body deve conter as seções:\n"
        + "\n".join(f"  {s}" for s in REQUIRED_SECTIONS)
    )
else:
    missing = [s for s in REQUIRED_SECTIONS if s.lower() not in body.lower()]
    if missing:
        errors.append(
            "Body incompleto. Seções obrigatórias faltando:\n"
            + "\n".join(f"  {s}" for s in missing)
            + "\n\nTemplate mínimo:\n"
            "  ## What\n  Descreva o que foi feito.\n\n"
            "  ## Why\n  Motivação / problema que resolve.\n\n"
            "  ## How\n  Como testar ou validar."
        )
    elif len(body.strip()) < 80:
        errors.append(
            "Body muito curto (< 80 chars). Descreva as mudanças adequadamente."
        )

# ── Resultado ─────────────────────────────────────────────────────────────────
if errors:
    reason = (
        "PR STANDARDS: PR bloqueado — padrões não respeitados.\n\n"
        + "\n\n".join(f"✗ {e}" for e in errors)
        + "\n\nCorrija e rode o comando novamente."
    )
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "deny", "permissionDecisionReason": reason}}))
else:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": "[pr-standards ✓] Título e body do PR validados."}}))

sys.exit(0)
