#!/usr/bin/env python3
"""
Hook #11 — Branch Taxonomy (PreToolUse: Bash)
Kura Project

Intercepta git checkout -b / git switch -c e enforça a taxonomia de branches.

Taxonomia obrigatória:
  feat/<descricao>      — nova funcionalidade
  fix/<descricao>       — correção de bug
  hotfix/<descricao>    — correção urgente
  chore/<descricao>     — manutenção, dependências, infra
  docs/<descricao>      — documentação
  refactor/<descricao>  — refatoração sem mudança de comportamento
  test/<descricao>      — testes
  ci/<descricao>        — CI/CD e pipelines
  perf/<descricao>      — performance

Regras da descrição:
  - Apenas letras minúsculas, números e hífens
  - Mínimo 3 caracteres, máximo 50
  - Sem espaços ou caracteres especiais
  - Sem sufixo de número de versão (use chore/bump-version)
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

# Detecta criação de branch
branch_match = (
    re.search(r'git\s+checkout\s+-b\s+(\S+)', command) or
    re.search(r'git\s+switch\s+-c\s+(\S+)', command) or
    re.search(r'git\s+branch\s+(\S+)', command)
)

if not branch_match:
    sys.exit(0)

branch_name = branch_match.group(1)

# Ignora flags (ex: git branch -d, git branch --list)
if branch_name.startswith('-'):
    sys.exit(0)

# ── Branches protegidas (nunca criar diretamente) ─────────────────────────────
PROTECTED = {'main', 'master', 'develop', 'staging', 'production'}
if branch_name in PROTECTED:
    reason = (
        f"BRANCH TAXONOMY: '{branch_name}' é uma branch protegida.\n\n"
        f"Branches protegidas não podem ser criadas diretamente.\n"
        f"Trabalhe em uma feature branch e abra um PR:\n"
        f"  git checkout -b feat/minha-feature"
    )
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }))
    sys.exit(0)

# ── Valida taxonomia ──────────────────────────────────────────────────────────
VALID_TYPES = [
    'feat', 'fix', 'hotfix', 'chore',
    'docs', 'refactor', 'test', 'ci', 'perf',
]

# Padrão: <type>/<slug>
BRANCH_PATTERN = re.compile(
    r'^(?P<type>' + '|'.join(VALID_TYPES) + r')'
    r'/(?P<slug>[a-z0-9][a-z0-9\-]{2,49})$'
)

match = BRANCH_PATTERN.match(branch_name)

if match:
    branch_type = match.group('type')
    slug        = match.group('slug')

    # Avisos adicionais (não bloqueantes)
    warnings = []
    if len(slug) > 40:
        warnings.append(f"Slug longo ({len(slug)} chars) — considere encurtar para melhor legibilidade.")
    if slug.endswith('-'):
        warnings.append("Slug termina com hífen — remova o hífen final.")

    msg = f"[branch-taxonomy ✓] '{branch_name}' — tipo: {branch_type}, slug: {slug}."
    if warnings:
        msg += "\n⚠️  " + "\n⚠️  ".join(warnings)

    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": msg,
        }
    }))
    sys.exit(0)

# ── Branch inválida — bloqueia com sugestão ───────────────────────────────────
# Tenta inferir o tipo correto a partir do nome
suggested_type = None
slug_candidate = branch_name.lower().replace(' ', '-').replace('_', '-')

for keyword, suggested in [
    (['bug', 'fix', 'corr'], 'fix'),
    (['feat', 'add', 'new', 'impl'], 'feat'),
    (['doc', 'readme', 'wiki'], 'docs'),
    (['test', 'spec', 'mock'], 'test'),
    (['refact', 'clean', 'extract'], 'refactor'),
    (['perf', 'optim', 'speed', 'fast'], 'perf'),
    (['ci', 'pipeline', 'deploy', 'action'], 'ci'),
    (['chore', 'bump', 'dep', 'upgrade'], 'chore'),
    (['hotfix', 'urgent', 'critical', 'patch'], 'hotfix'),
]:
    if any(k in branch_name.lower() for k in keyword):
        suggested_type = suggested
        break

suggestion = (
    f"  git checkout -b {suggested_type}/{slug_candidate[:40]}"
    if suggested_type
    else "\n".join([f"  git checkout -b {t}/minha-descricao" for t in ['feat', 'fix', 'chore']])
)

reason = (
    f"BRANCH TAXONOMY: '{branch_name}' não segue a taxonomia do projeto.\n\n"
    f"Formato obrigatório: <tipo>/<slug>\n\n"
    f"Tipos válidos:\n"
    + "\n".join(f"  {t:<12} — {desc}" for t, desc in [
        ('feat',     'nova funcionalidade'),
        ('fix',      'correção de bug'),
        ('hotfix',   'correção urgente'),
        ('chore',    'manutenção / dependências'),
        ('docs',     'documentação'),
        ('refactor', 'refatoração'),
        ('test',     'testes'),
        ('ci',       'CI/CD e pipelines'),
        ('perf',     'performance'),
    ])
    + f"\n\nRegras do slug:\n"
    f"  • Apenas letras minúsculas, números e hífens\n"
    f"  • Entre 3 e 50 caracteres\n"
    f"  • Sem espaços ou underscores\n\n"
    f"Sugestão:\n{suggestion}"
)

print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": reason,
    }
}))
sys.exit(0)
