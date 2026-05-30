#!/usr/bin/env python3
"""
Hook #7 — Commit Gate (PreToolUse: Bash)
Kura Project

Intercepta git commit e executa:
  1. mix test          — todos os testes devem passar
  2. mix credo --strict — zero violacoes

Se qualquer etapa falhar → commit bloqueado com output detalhado.
Commit só acontece com tudo verde.
"""
import json
import os
import re
import subprocess
import sys

# ── Input ─────────────────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

command = data.get("tool_input", {}).get("command", "")

# Só intercepta git commit (qualquer forma: --amend, -m, etc.)
if not re.search(r'\bgit\s+commit\b', command):
    sys.exit(0)

# ── Encontra root do projeto Elixir ───────────────────────────────────────────
cwd = data.get("cwd", os.getcwd())

def find_project_root(start):
    current = os.path.abspath(start)
    while current != '/':
        if os.path.exists(os.path.join(current, 'mix.exs')):
            return current
        current = os.path.dirname(current)
    return None

project_root = find_project_root(cwd)

if not project_root:
    # Não é projeto Elixir — deixa o commit passar
    sys.exit(0)

# ── Helpers ───────────────────────────────────────────────────────────────────
def run(cmd, timeout=300):
    return subprocess.run(
        cmd,
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=timeout,
    )

failures = []

# ── Etapa 1: mix test ─────────────────────────────────────────────────────────
try:
    result = run(["mix", "test", "--no-color"], timeout=300)
    if result.returncode != 0:
        output = (result.stdout + result.stderr).strip()
        # Extrai resumo da última linha (ex: "3 tests, 1 failure")
        lines   = output.splitlines()
        summary = next((l for l in reversed(lines) if re.search(r'\d+ test', l)), "Falhou")
        failures.append(("mix test", summary, output))
except subprocess.TimeoutExpired:
    failures.append(("mix test", "Timeout (>5min)", ""))
except FileNotFoundError:
    failures.append(("mix test", "mix nao encontrado", ""))

# ── Etapa 2: mix credo --strict (só se testes passaram) ───────────────────────
if not failures:
    try:
        result = run(["mix", "credo", "--strict", "--no-color", "--format", "oneline"], timeout=60)
        if result.returncode != 0:
            output  = (result.stdout + result.stderr).strip()
            lines   = output.splitlines()
            issues  = [l for l in lines if l.strip() and not l.startswith("Checking")]
            summary = f"{len(issues)} violacao(es)"
            failures.append(("mix credo", summary, "\n".join(issues[:20])))
    except subprocess.TimeoutExpired:
        failures.append(("mix credo", "Timeout (>60s)", ""))
    except FileNotFoundError:
        failures.append(("mix credo", "mix nao encontrado", ""))

# ── Resultado ─────────────────────────────────────────────────────────────────
if failures:
    sections = []
    for step, summary, output in failures:
        section = f"[{step}] FALHOU — {summary}"
        if output:
            # Limita output para não sobrecarregar
            lines = output.splitlines()
            shown = "\n".join(lines[:30])
            if len(lines) > 30:
                shown += f"\n... e mais {len(lines) - 30} linhas."
            section += f"\n{shown}"
        sections.append(section)

    reason = (
        "COMMIT GATE bloqueou o commit.\n\n"
        + "\n\n".join(sections)
        + "\n\nCorreja os problemas acima antes de commitar."
    )
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
else:
    # Tudo verde — permite com confirmação
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": "Commit gate: mix test OK, mix credo OK. Commit liberado.",
        }
    }))

sys.exit(0)
