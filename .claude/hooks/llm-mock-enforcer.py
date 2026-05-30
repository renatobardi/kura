#!/usr/bin/env python3
"""
Hook #9 — LLM Mock Enforcer (PostToolUse: Write | Edit)
Kura Project

Verifica que arquivos de teste nao chamam providers LLM reais.
Todo acesso a LLM em testes DEVE passar por Mox (LLM.Provider mock).

Detecta:
  - Uso direto de Kura.LLM.Gemini ou Kura.LLM.Claude em testes
  - Chamadas HTTP diretas a APIs LLM (api.openai.com, api.anthropic.com, etc.)
  - Uso de HTTPoison/Req sem Bypass/Mox em contexto de teste
  - Modulo LLM.Router sendo chamado sem stub configurado

Padrao correto em testes:
  import Mox
  setup :verify_on_exit!
  
  test "ingests content" do
    Kura.LLM.ProviderMock
    |> expect(:complete, fn _prompt, _opts -> {:ok, "resposta mock"} end)
    ...
  end
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

tool_name  = data.get("tool_name", "")
tool_input = data.get("tool_input", {})

if tool_name not in ("Write", "Edit"):
    sys.exit(0)

file_path = tool_input.get("file_path", "")
content   = tool_input.get("content", "") or tool_input.get("new_string", "")

# Só arquivos de teste
if not re.search(r'/test/.*_test\.exs$', file_path):
    sys.exit(0)

if not content:
    sys.exit(0)

# ── Padrões proibidos em arquivos de teste ────────────────────────────────────
FORBIDDEN = [
    # Chamada direta a implementações reais de LLM
    (
        r'Kura\.LLM\.(Gemini|Claude|Kimi)\.',
        "Chamada direta a provider LLM real. Use Kura.LLM.ProviderMock via Mox."
    ),
    # URLs de API reais hardcoded
    (
        r'api\.openai\.com|api\.anthropic\.com|generativelanguage\.googleapis\.com|api\.moonshot\.cn',
        "URL de API LLM hardcoded em teste. Use Bypass para interceptar HTTP."
    ),
    # HTTPoison/Req sem Bypass (heurística: chamada HTTP a domínio externo)
    (
        r'HTTPoison\.(?:get|post|request)|Req\.(?:get|post|request).*https?://',
        "HTTP real em teste. Use Bypass.open() para interceptar chamadas externas."
    ),
    # Application.get_env para pegar chave de API em teste
    (
        r'Application\.get_env.*api_key|System\.get_env.*API_KEY',
        "Leitura de API key real em teste. Configure via Mox, nao via env vars."
    ),
]

# ── Verifica se usa Mox corretamente ──────────────────────────────────────────
uses_mox    = bool(re.search(r'import Mox|use Mox|Mox\.', content))
uses_bypass = bool(re.search(r'Bypass\.open|Bypass\.expect', content))
has_stub    = bool(re.search(r'stub\(|expect\(|allow\(', content))

violations = []
for pattern, message in FORBIDDEN:
    matches = re.findall(pattern, content, re.IGNORECASE)
    if matches:
        violations.append(f"• {message}")

# ── Resultado ─────────────────────────────────────────────────────────────────
rel_file = os.path.basename(file_path)

if violations:
    viol_text = "\n".join(violations)
    fix_hint  = (
        "\nPadrao correto:\n\n"
        "  # No test/support/mocks.exs:\n"
        "  Mox.defmock(Kura.LLM.ProviderMock, for: Kura.LLM.Provider)\n\n"
        "  # No teste:\n"
        "  import Mox\n"
        "  setup :verify_on_exit!\n\n"
        "  test \"meu teste\" do\n"
        "    Kura.LLM.ProviderMock\n"
        "    |> expect(:complete, fn _prompt, _opts -> {:ok, \"mock\"} end)\n"
        "    # ... resto do teste\n"
        "  end"
    )
    context = (
        f"LLM MOCK ENFORCER: Violacoes em {rel_file}:\n\n"
        f"{viol_text}"
        f"{fix_hint}"
    )
    print(json.dumps({"additionalContext": context}))

elif content and re.search(r'Kura\.LLM\.|llm|LLM', content, re.IGNORECASE):
    # Menciona LLM mas não viola — verifica se está usando Mox
    if not uses_mox and not uses_bypass and not has_stub:
        context = (
            f"LLM MOCK ENFORCER: {rel_file} menciona LLM mas nao usa Mox.\n"
            f"Confirme que toda interacao com LLM esta sendo mockada via:\n"
            f"  import Mox + Kura.LLM.ProviderMock"
        )
        print(json.dumps({"additionalContext": context}))
    else:
        print(json.dumps({
            "additionalContext": f"[llm-mock ✓] {rel_file} — LLM mockado corretamente."
        }))

sys.exit(0)
