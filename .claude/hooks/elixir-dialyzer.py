#!/usr/bin/env python3
"""
Hook #5 — Elixir Dialyzer (PostToolUse: Edit)
Kura Project

Roda mix dialyzer nos módulos core após cada edição.
Só dispara para arquivos nos diretórios críticos — evita overhead em todo .ex.

Módulos core monitorados:
  lib/kura/llm/
  lib/kura/vault/
  lib/kura/collectors/
  lib/kura/ingestion/
  lib/kura/governance/

Dialyzer verifica:
  - Discrepâncias de tipo (type mismatches)
  - Cláusulas impossíveis de alcançar
  - Funções que nunca retornam
  - Chamadas a funções inexistentes
  - Contratos @spec violados
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

# Só arquivos .ex (não .exs — dialyzer não analisa scripts)
if not file_path.endswith('.ex'):
    sys.exit(0)

# ── Módulos core monitorados ───────────────────────────────────────────────────
CORE_MODULES = [
    r'/lib/kura/llm/',
    r'/lib/kura/vault/',
    r'/lib/kura/collectors/',
    r'/lib/kura/ingestion/',
    r'/lib/kura/governance/',
    r'/lib/kura/outputs/',
    r'/lib/kura/inbox/',
    r'/lib/kura/accounts/',
]

is_core = any(re.search(pat, file_path) for pat in CORE_MODULES)
if not is_core:
    # Não é módulo core — skip silencioso
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

rel_file = os.path.relpath(
    file_path if os.path.isabs(file_path) else os.path.join(project_root, file_path),
    project_root
)

# ── Roda mix dialyzer ─────────────────────────────────────────────────────────
TIMEOUT = 180  # segundos — PLT já construída: ~15-30s; primeira vez: pode demorar

try:
    result = subprocess.run(
        ["mix", "dialyzer", "--no-check", "--format", "short"],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=TIMEOUT,
    )

    output = (result.stdout + result.stderr).strip()
    passed = result.returncode == 0

    # Filtra linhas relevantes (remove ruído de progresso)
    lines = [
        l for l in output.splitlines()
        if l.strip()
        and not l.startswith("Finding")
        and not l.startswith("Checking")
        and not l.startswith("done")
        and "No warnings" not in l
    ]
    filtered = "\n".join(lines).strip()

    if passed or not filtered:
        context = f"[dialyzer ✓] {rel_file} — sem erros de tipo."
    else:
        # Conta warnings por arquivo
        warnings_in_file = [l for l in lines if rel_file in l or file_path in l]
        other_warnings   = [l for l in lines if rel_file not in l and file_path not in l]

        parts = []
        if warnings_in_file:
            parts.append(f"No arquivo editado:\n" + "\n".join(warnings_in_file))
        if other_warnings:
            parts.append(f"Em outros modulos ({len(other_warnings)} warnings):\n" + "\n".join(other_warnings[:5]))
            if len(other_warnings) > 5:
                parts.append(f"  ... e mais {len(other_warnings) - 5} warnings.")

        context = f"[dialyzer ✗] {rel_file}:\n\n" + "\n\n".join(parts)

    print(json.dumps({"additionalContext": context}))

except subprocess.TimeoutExpired:
    msg = (
        f"[dialyzer] Timeout ({TIMEOUT}s) para {rel_file}.\n"
        f"Se a PLT ainda nao foi construida, rode manualmente:\n"
        f"  cd {project_root} && mix dialyzer --plt\n"
        f"Proximas execucoes serao rapidas (analise incremental)."
    )
    print(json.dumps({"additionalContext": msg}))

except FileNotFoundError:
    print(json.dumps({"additionalContext": "mix nao encontrado — Elixir instalado?"}))

except Exception as e:
    print(json.dumps({"additionalContext": f"Erro ao rodar dialyzer: {e}"}))

sys.exit(0)
