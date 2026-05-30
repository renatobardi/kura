#!/usr/bin/env python3
"""
Hook #8 — TDD Enforcer (PreToolUse: Write)
Kura Project

Garante que o arquivo de teste exista ANTES de criar qualquer módulo em lib/.
Implementa o Red step obrigatório do ciclo TDD Red-Green-Refactor.

Fluxo correto:
  1. Cria test/kura/vault/ingest_test.exs  (Red — teste falha)
  2. Cria lib/kura/vault/ingest.ex          (Green — implementa)
  3. Refatora

Este hook bloqueia o passo 2 se o passo 1 não foi feito.

Exceções permitidas (não bloqueia):
  - Arquivos de configuração: config.ex, application.ex, repo.ex
  - Módulos de migração (priv/repo/migrations/)
  - Módulos behaviour/protocol puro (só @callback/@spec)
  - Mix tasks (lib/mix/tasks/)
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

if data.get("tool_name") != "Write":
    sys.exit(0)

tool_input = data.get("tool_input", {})
file_path  = tool_input.get("file_path", "")
content    = tool_input.get("content", "")

# Só arquivos .ex em lib/
if not re.search(r'/lib/.+\.ex$', file_path):
    sys.exit(0)

# ── Exceções: arquivos que legitimamente não têm teste próprio ────────────────
EXEMPT_PATTERNS = [
    r'/lib/kura/application\.ex$',
    r'/lib/kura_web\.ex$',
    r'/lib/kura\.ex$',
    r'/lib/mix/tasks/',
    r'/priv/repo/migrations/',
    r'_web/endpoint\.ex$',
    r'_web/router\.ex$',
    r'_web/telemetry\.ex$',
    r'_web/gettext\.ex$',
    r'/config\.ex$',
    r'/repo\.ex$',
]
if any(re.search(p, file_path) for p in EXEMPT_PATTERNS):
    sys.exit(0)

# Conteúdo só com behaviours/protocols (sem lógica a testar)
if content and re.search(r'defprotocol|defimpl|@callback', content):
    behaviour_lines = [l for l in content.splitlines()
                       if l.strip() and not l.strip().startswith('#')]
    has_real_funcs  = any(re.match(r'\s*def\b', l) for l in behaviour_lines)
    if not has_real_funcs:
        sys.exit(0)

# ── Mapeia lib/ → test/ ───────────────────────────────────────────────────────
def to_test_path(lib_path):
    test_path = re.sub(r'/lib/', '/test/', lib_path, count=1)
    test_path = re.sub(r'\.ex$', '_test.exs', test_path)
    return test_path

expected_test = to_test_path(file_path)

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

abs_test = (
    expected_test if os.path.isabs(expected_test)
    else os.path.join(project_root, expected_test)
)

# ── Verifica se teste existe ───────────────────────────────────────────────────
if os.path.exists(abs_test):
    # Teste existe — TDD seguido corretamente
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": f"TDD OK: arquivo de teste encontrado em {expected_test}",
        }
    }))
    sys.exit(0)

# ── Teste não existe — bloqueia ───────────────────────────────────────────────
module_name = (
    re.search(r'defmodule\s+([\w.]+)', content).group(1)
    if content and re.search(r'defmodule', content)
    else os.path.basename(file_path).replace('.ex', '')
)

reason = (
    f"TDD ENFORCER: Arquivo de teste nao encontrado.\n\n"
    f"Tentativa de criar:  {file_path}\n"
    f"Teste esperado em:   {expected_test}\n\n"
    f"Ciclo TDD obrigatorio:\n"
    f"  1. RED   — crie {expected_test} com testes falhando\n"
    f"  2. GREEN — crie {file_path} para fazer os testes passarem\n"
    f"  3. REFACTOR — melhore o codigo mantendo testes verdes\n\n"
    f"Exemplo minimo para comecar:\n\n"
    f"  defmodule {module_name}Test do\n"
    f"    use ExUnit.Case, async: true\n\n"
    f"    describe \"{module_name}\" do\n"
    f"      test \"placeholder — implemente antes de criar o modulo\" do\n"
    f"        assert false, \"TDD: escreva o teste real aqui\"\n"
    f"      end\n"
    f"    end\n"
    f"  end"
)

print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": reason,
    }
}))

sys.exit(0)
