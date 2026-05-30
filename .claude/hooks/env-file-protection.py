#!/usr/bin/env python3
"""
Hook #2 — Env File Protection (PreToolUse: Write | Edit | Bash)
Kura Project

Comportamentos:
  Write/Edit em .env        → aviso informativo (não bloqueia — é ok editar localmente)
  Write/Edit em .env.example com valor real → bloqueia (deve ter só placeholders)
  Bash: git add/commit .env → bloqueia sempre
"""
import json
import re
import sys

# ── Input ─────────────────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

tool_name  = data.get("tool_name", "")
tool_input = data.get("tool_input", {})

# ═════════════════════════════════════════════════════════════════════════════
# CASO 1: Write / Edit em arquivos .env*
# ═════════════════════════════════════════════════════════════════════════════
if tool_name in ("Write", "Edit"):
    file_path = tool_input.get("file_path", "")
    content   = tool_input.get("content", "") or tool_input.get("new_string", "")

    import os
    filename = os.path.basename(file_path)

    # .env.example / .env.template / .env.sample → só placeholders permitidos
    if re.match(r'\.env\.(example|template|sample)$', filename, re.IGNORECASE):
        # Detecta valores que parecem reais (não placeholders)
        real_value_pattern = (
            r'(?:^|\n)'
            r'[A-Z_]+=\s*'
            r'(?!your_|<|{|\[|xxx|example|placeholder|changeme|todo|true|false|""|\'\')' 
            r'[A-Za-z0-9\-_\.\/\+]{16,}'
        )
        if re.search(real_value_pattern, content, re.IGNORECASE):
            reason = (
                f"ENV PROTECTION: '{filename}' deve conter APENAS placeholders.\n\n"
                f"Exemplo correto:\n"
                f"  GEMINI_API_KEY=your_gemini_api_key_here\n"
                f"  ANTHROPIC_API_KEY=<YOUR_ANTHROPIC_KEY>\n\n"
                f"Valores reais pertencem ao .env local (nunca versionado)."
            )
            print(json.dumps({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason,
                }
            }))
        sys.exit(0)

    # .env (sem sufixo) → aviso informativo, não bloqueia
    if filename == ".env":
        print(json.dumps({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "additionalContext": (
                    "INFO: Editando .env local — ok. "
                    "Confirme que .env está no .gitignore e nunca será commitado."
                ),
            }
        }))
        sys.exit(0)

    sys.exit(0)

# ═════════════════════════════════════════════════════════════════════════════
# CASO 2: Bash — bloqueia git add/commit com arquivos sensíveis
# ═════════════════════════════════════════════════════════════════════════════
if tool_name == "Bash":
    command = tool_input.get("command", "")

    # Padrões de arquivos que nunca devem ser staged
    PROTECTED_PATTERNS = [
        r'\.env(?!\.(example|template|sample))',  # .env mas não .env.example etc.
        r'\.pem\b',
        r'\.key\b',
        r'id_rsa',
        r'id_ed25519',
        r'private_key',
    ]

    is_git_add    = re.search(r'\bgit\s+add\b', command)
    is_git_commit = re.search(r'\bgit\s+commit\b', command)

    if is_git_add or is_git_commit:
        for pat in PROTECTED_PATTERNS:
            if re.search(pat, command, re.IGNORECASE):
                action = "git add" if is_git_add else "git commit"
                reason = (
                    f"ENV PROTECTION: Bloqueado '{action}' com arquivo sensível.\n\n"
                    f"Arquivos como .env, chaves privadas e certificados\n"
                    f"NUNCA devem ser versionados no git.\n\n"
                    f"Verifique seu .gitignore:\n"
                    f"  .env\n"
                    f"  *.pem\n"
                    f"  *.key\n"
                    f"  id_rsa\n"
                    f"  id_ed25519"
                )
                print(json.dumps({
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "deny",
                        "permissionDecisionReason": reason,
                    }
                }))
                sys.exit(0)

sys.exit(0)
