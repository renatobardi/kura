#!/usr/bin/env python3
"""
Hook #1 — Secret Scanner (PreToolUse: Write | Edit)
Kura Project

Bloqueia escrita de API keys, tokens e credenciais em qualquer arquivo.
Roda antes de toda chamada Write ou Edit.

Exit 0 + JSON deny  → bloqueia a ação e mostra motivo ao Claude
Exit 0 sem JSON     → permite normalmente
"""
import json
import re
import sys

# ── Leitura do input ─────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

tool_name  = data.get("tool_name", "")
tool_input = data.get("tool_input", {})

# ── Extrai conteúdo a escanear ───────────────────────────────────────────────
content   = ""
file_path = ""

if tool_name == "Write":
    content   = tool_input.get("content", "")
    file_path = tool_input.get("file_path", "")
elif tool_name == "Edit":
    content   = tool_input.get("new_string", "")
    file_path = tool_input.get("file_path", "")
else:
    sys.exit(0)

if not content:
    sys.exit(0)

# ── Whitelist: conteúdo claramente falso/placeholder ─────────────────────────
SAFE_INDICATORS = [
    "[REDACTED", "<API_KEY>", "<YOUR_KEY>", "your_api_key",
    "your-api-key", "placeholder", "fake_key", "test_key",
    "mock_key", "example_key", "sk-xxx", "sk-ant-xxx",
    "AIzaXXX", "AKIAIOSFODNN7EXAMPLE",
]
if any(ind.lower() in content.lower() for ind in SAFE_INDICATORS):
    sys.exit(0)

# ── Padrões de segredos ──────────────────────────────────────────────────────
PATTERNS = [
    # Providers LLM (alta confiança)
    (r'sk-[a-zA-Z0-9]{32,}',                              "OpenAI API key"),
    (r'sk-ant-[a-zA-Z0-9\-_]{20,}',                       "Anthropic API key"),
    (r'AIza[0-9A-Za-z\-_]{35}',                           "Google API key"),
    # Cloud providers
    (r'AKIA[0-9A-Z]{16}',                                  "AWS Access Key ID"),
    (r'(?i)aws_secret_access_key\s*[=:]\s*\S{40}',        "AWS Secret Access Key"),
    # Git platforms
    (r'gh[pousr]_[A-Za-z0-9_]{36,}',                      "GitHub token"),
    (r'github_pat_[a-zA-Z0-9]{22}_[a-zA-Z0-9]{59}',       "GitHub PAT"),
    # Comunicação
    (r'xoxb-[0-9]{11}-[0-9]{11}-[a-zA-Z0-9]{24}',         "Slack Bot token"),
    (r'xoxp-[0-9]+-[0-9]+-[0-9]+-[a-zA-Z0-9]+',           "Slack User token"),
    # Chaves privadas
    (r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----', "Private key"),
    # Atribuições genéricas (moderada confiança)
    (
        r'(?<![a-zA-Z_])'
        r'(?:password|passwd|api_key|apikey|secret_key|secret|'
        r'access_token|auth_token|private_key|client_secret)'
        r'\s*[=:]\s*'
        r'["\']?'
        r'(?!your_|<|\{|test|fake|mock|example|placeholder|true|false|nil|null|""|\'\')'
        r'[A-Za-z0-9\-_\.\/\+]{16,}'
        r'["\']?',
        "Generic credential assignment"
    ),
]

# ── Scan ─────────────────────────────────────────────────────────────────────
found = []
for pattern, label in PATTERNS:
    match = re.search(pattern, content, re.IGNORECASE | re.MULTILINE)
    if match:
        raw    = match.group(0)
        masked = raw[:6] + "*" * min(len(raw) - 6, 20) if len(raw) > 6 else "****"
        found.append(f"{label}  →  {masked}")

# ── Resultado ────────────────────────────────────────────────────────────────
if found:
    lines  = "\n".join(f"  • {item}" for item in found)
    reason = (
        f"SECRET SCANNER bloqueou escrita em '{file_path or 'arquivo'}'.\n\n"
        f"Credenciais detectadas:\n{lines}\n\n"
        f"Use variaveis de ambiente ou o placeholder [REDACTED:tipo].\n"
        f"Nunca escreva segredos reais em arquivos de codigo."
    )
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))

sys.exit(0)
