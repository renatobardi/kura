#!/usr/bin/env python3
"""
Hook #3 — Elixir Test Runner (PostToolUse: Edit)
Kura Project

Após editar qualquer arquivo .ex/.exs, encontra o teste correspondente
e roda mix test automaticamente. Fecha o loop TDD sem intervenção manual.

Mapeamento:
  lib/kura/vault/ingest.ex   → test/kura/vault/ingest_test.exs
  test/kura/vault/ingest_test.exs → roda diretamente
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

# ── Resolve caminho do arquivo de teste ───────────────────────────────────────
def find_test_file(path):
    """Mapeia lib/ → test/ ou retorna o próprio caminho se já for teste."""
    if re.search(r'/test/.*_test\.exs$', path):
        return path  # já é um arquivo de teste

    if '/lib/' in path:
        test_path = re.sub(r'/lib/', '/test/', path)
        test_path = re.sub(r'\.ex$', '_test.exs', test_path)
        return test_path

    # Arquivo .exs genérico em lib/ ou outro lugar
    if path.endswith('.exs') and '/test/' not in path:
        test_path = re.sub(r'/lib/', '/test/', path, count=1)
        if not test_path.endswith('_test.exs'):
            test_path = re.sub(r'\.exs$', '_test.exs', test_path)
        return test_path

    return None

test_file = find_test_file(file_path)

if not test_file:
    sys.exit(0)

# ── Encontra root do projeto (onde está mix.exs) ──────────────────────────────
def find_project_root(start):
    current = os.path.abspath(start)
    while current != '/':
        if os.path.exists(os.path.join(current, 'mix.exs')):
            return current
        current = os.path.dirname(current)
    return None

project_root = find_project_root(os.path.dirname(file_path) or os.getcwd())

if not project_root:
    # mix.exs não encontrado — não é um projeto Elixir ainda
    sys.exit(0)

# ── Verifica se arquivo de teste existe ───────────────────────────────────────
abs_test = test_file if os.path.isabs(test_file) else os.path.join(project_root, test_file)

if not os.path.exists(abs_test):
    msg = (
        f"TDD: arquivo de teste nao encontrado: {test_file}\n"
        f"➜ Escreva o teste ANTES da implementacao (Red step do TDD).\n"
        f"➜ Caminho esperado: {test_file}"
    )
    print(json.dumps({"additionalContext": msg}))
    sys.exit(0)

# ── Roda mix test ─────────────────────────────────────────────────────────────
try:
    result = subprocess.run(
        ["mix", "test", abs_test, "--no-color", "--formatter", "ExUnit.CLIFormatter"],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=120,
    )
    output   = (result.stdout + result.stderr).strip()
    passed   = result.returncode == 0
    status   = "PASS ✓" if passed else "FAIL ✗"
    rel_test = os.path.relpath(abs_test, project_root)

    context = f"[mix test {status}] {rel_test}\n\n{output}"
    print(json.dumps({"additionalContext": context}))

except subprocess.TimeoutExpired:
    print(json.dumps({"additionalContext": "mix test timeout (>120s) — verifique se o banco está acessível."}))
except FileNotFoundError:
    print(json.dumps({"additionalContext": "mix não encontrado no PATH — Elixir instalado?"}))
except Exception as e:
    print(json.dumps({"additionalContext": f"Erro ao rodar mix test: {e}"}))

sys.exit(0)
