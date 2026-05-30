#!/usr/bin/env python3
"""
Hook #12 — Commit Message (PreToolUse: Bash)
Kura Project

Enforça Conventional Commits em toda mensagem de commit.

Formato obrigatório:
  <type>(<scope>): <description>    ← scope opcional
  <type>!(<scope>): <description>   ← breaking change

Tipos válidos:
  feat     — nova funcionalidade
  fix      — correção de bug
  docs     — documentação
  style    — formatação (sem mudança de lógica)
  refactor — refatoração
  test     — testes
  chore    — manutenção / dependências
  perf     — performance
  ci       — CI/CD
  build    — build system
  revert   — revert de commit anterior

Regras:
  • Primeira linha ≤ 72 caracteres
  • Descrição começa com minúscula
  • Descrição não termina com ponto
  • Body separado por linha em branco
  • BREAKING CHANGE: no footer ou ! após o tipo
"""
import json
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

if not re.search(r'\bgit\s+commit\b', command):
    sys.exit(0)

# ── Extrai mensagem do commit ─────────────────────────────────────────────────
msg_match = (
    re.search(r'git\s+commit\s+.*?-m\s+"([^"]+)"', command) or
    re.search(r"git\s+commit\s+.*?-m\s+'([^']+)'", command) or
    re.search(r'git\s+commit\s+.*?--message[= ]"([^"]+)"', command) or
    re.search(r"git\s+commit\s+.*?--message[= ]'([^']+)'", command)
)

if not msg_match:
    # git commit sem -m (abrirá editor) — lembra do formato
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": (
                "COMMIT MESSAGE: Editor abrirá para mensagem.\n"
                "Formato obrigatório: <type>(<scope>): <description>\n"
                "Exemplos:\n"
                "  feat(vault): add karpathy wiki incremental ingest\n"
                "  fix(chat): prevent title overwrite when locked\n"
                "  chore(deps): bump elixir to 1.17"
            ),
        }
    }))
    sys.exit(0)

message = msg_match.group(1).strip()
lines   = message.split('\n')
subject = lines[0].strip()

# ── Valida linha de assunto ───────────────────────────────────────────────────
VALID_TYPES = [
    'feat', 'fix', 'docs', 'style', 'refactor',
    'test', 'chore', 'perf', 'ci', 'build', 'revert',
]

SUBJECT_PATTERN = re.compile(
    r'^(?P<type>' + '|'.join(VALID_TYPES) + r')'
    r'(?P<breaking>!)?'
    r'(?:\((?P<scope>[a-z0-9][a-z0-9\-/]*)\))?'
    r':\s+'
    r'(?P<desc>.+)$'
)

errors   = []
warnings = []

match = SUBJECT_PATTERN.match(subject)

if not match:
    # Tenta dar dica do tipo
    type_hint = None
    for t in VALID_TYPES:
        if subject.lower().startswith(t):
            type_hint = t
            break

    if type_hint:
        errors.append(
            f"Formato inválido. Encontrado: '{subject}'\n"
            f"  Parece que o tipo é '{type_hint}', mas falta o ':' e espaço.\n"
            f"  Correto: {type_hint}: {subject[len(type_hint):].lstrip()}"
        )
    else:
        errors.append(
            f"Formato inválido: '{subject}'\n"
            f"  Esperado: <type>(<scope>): <description>\n"
            f"  Tipos válidos: {', '.join(VALID_TYPES)}"
        )
else:
    commit_type = match.group('type')
    scope       = match.group('scope')
    desc        = match.group('desc')
    is_breaking = bool(match.group('breaking'))

    # Comprimento da primeira linha
    if len(subject) > 72:
        errors.append(f"Primeira linha muito longa ({len(subject)} chars, máx 72).")

    # Descrição começa com minúscula
    if desc and desc[0].isupper():
        errors.append(
            f"Descrição deve começar com minúscula.\n"
            f"  Correto: {commit_type}{'(' + scope + ')' if scope else ''}: "
            f"{desc[0].lower() + desc[1:]}"
        )

    # Descrição não termina com ponto
    if desc and desc.endswith('.'):
        errors.append("Descrição não deve terminar com ponto.")

    # Descrição muito curta
    if desc and len(desc.strip()) < 5:
        errors.append("Descrição muito curta — seja mais descritivo.")

    # Verifica body (se existe)
    if len(lines) > 1:
        if lines[1].strip():
            errors.append(
                "Body deve ser separado do assunto por uma linha em branco.\n"
                "  Correto:\n"
                "    feat(vault): add karpathy ingest\n"
                "    \n"
                "    Descrição detalhada aqui..."
            )

    # BREAKING CHANGE no footer
    if len(lines) > 2:
        footer = '\n'.join(lines[2:])
        if 'BREAKING CHANGE' in footer and not re.search(r'^BREAKING[ -]CHANGE:', footer, re.MULTILINE):
            errors.append(
                "BREAKING CHANGE deve seguir o formato:\n"
                "  BREAKING CHANGE: descrição do que quebrou"
            )

    # Aviso para breaking change sem !
    if is_breaking:
        warnings.append("Breaking change (!) detectado — certifique-se de que o footer tem BREAKING CHANGE: <desc>")

# ── Resultado ─────────────────────────────────────────────────────────────────
if errors:
    err_text  = "\n\n".join(f"  ✗ {e}" for e in errors)
    examples  = (
        "\nExemplos corretos:\n"
        "  feat(vault): add karpathy wiki incremental ingest\n"
        "  fix(chat): prevent title overwrite when title_locked is true\n"
        "  chore(deps): bump phoenix to 1.8\n"
        "  feat!: redesign vault storage model\n\n"
        "  BREAKING CHANGE: vault.pages table renamed to vault.knowledge_pages"
    )
    reason = f"COMMIT MESSAGE: Mensagem não segue Conventional Commits.\n\n{err_text}\n{examples}"

    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
else:
    warn_text = ""
    if warnings:
        warn_text = "\n⚠️  " + "\n⚠️  ".join(warnings)
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": f"[commit-message ✓] Formato válido.{warn_text}",
        }
    }))

sys.exit(0)
