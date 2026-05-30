#!/usr/bin/env python3
"""
Hook #4 — Elixir Credo (PostToolUse: Edit)
Kura Project

Roda mix credo --strict no arquivo editado após cada Edit em .ex/.exs.
Aponta violações de estilo, qualidade e padrões do projeto.

Credo verifica:
  - Complexidade ciclomática
  - Módulos sem @moduledoc
  - Funções muito longas
  - Aliases desnecessários
  - Comparações com booleanos
  - IO.inspect/IO.puts esquecidos
  - e muito mais via .credo.exs
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

if data.get("tool_name") != "Edit":
    sys.exit(0)

file_path = data.get("tool_input", {}).get("file_path", "")

# Só arquivos Elixir
if not re.search(r'\.(ex|exs)$', file_path):
    sys.exit(0)

# Ignora arquivos gerados e migrations (ruído desnecessário)
SKIP_PATTERNS = [
    r'/_build/',
    r'/deps/',
    r'\.formatter\.exs$',
    r'/priv/repo/migrations/',
]
if any(re.search(p, file_path) for p in SKIP_PATTERNS):
    sys.exit(0)

# ── Encontra root do projeto ───────────────────────────────────────────────────
def find_project_root(start):
    current = os.path.abspath(start if os.path.isdir(start) else os.path.dirname(start))
    while current != '/':
        if os.path.exists(os.path.join(current, 'mix.exs')):
            return current
        current = os.path.dirname(current)
    return None

project_root = find_project_root(file_path)

if not project_root:
    sys.exit(0)

abs_file = file_path if os.path.isabs(file_path) else os.path.join(project_root, file_path)

if not os.path.exists(abs_file):
    sys.exit(0)

# ── Roda mix credo ────────────────────────────────────────────────────────────
try:
    result = subprocess.run(
        ["mix", "credo", abs_file, "--strict", "--no-color", "--format", "oneline"],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=60,
    )

    output = (result.stdout + result.stderr).strip()

    # Credo retorna 0 se sem issues, 1 se encontrou issues
    has_issues = result.returncode != 0 or bool(output)

    rel_file = os.path.relpath(abs_file, project_root)

    if not output or "no issues found" in output.lower():
        context = f"[credo ✓] {rel_file} — sem violacoes."
    else:
        # Conta issues por severidade
        errors   = len(re.findall(r'\[C\]', output))  # Consistency
        warnings = len(re.findall(r'\[W\]', output))  # Warning
        design   = len(re.findall(r'\[D\]', output))  # Design
        refactor = len(re.findall(r'\[R\]', output))  # Refactor

        summary_parts = []
        if errors:   summary_parts.append(f"{errors} consistencia")
        if warnings: summary_parts.append(f"{warnings} aviso")
        if design:   summary_parts.append(f"{design} design")
        if refactor: summary_parts.append(f"{refactor} refactor")
        summary = ", ".join(summary_parts) if summary_parts else "issues encontrados"

        context = f"[credo ✗] {rel_file} — {summary}:\n\n{output}"

    print(json.dumps({"additionalContext": context}))

except subprocess.TimeoutExpired:
    print(json.dumps({"additionalContext": "credo timeout (>60s)."}))
except FileNotFoundError:
    print(json.dumps({"additionalContext": "mix nao encontrado — Elixir instalado?"}))
except Exception as e:
    print(json.dumps({"additionalContext": f"Erro ao rodar credo: {e}"}))

sys.exit(0)
