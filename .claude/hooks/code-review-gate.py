#!/usr/bin/env python3
"""
Hook #16 — Code Review Gate (PreToolUse: Bash)
Kura Project

Bloqueia `gh pr merge` sem code review aprovado.

Fluxo:
  1. Detecta `gh pr merge` ou `gh pr merge <número>`
  2. Obtém o diff do PR via `gh pr diff`
  3. Roda `claude` CLI como reviewer (se disponível)
  4. Salva o review em .claude/reviews/<branch>.md
  5. Bloqueia se review encontrar issues CRITICAL ou HIGH
  6. Permite com aviso se apenas LOW/MEDIUM

Se `claude` CLI não estiver disponível, bloqueia e pede review manual.

Review é re-aproveitado se o diff não mudou desde o último review
(compara SHA do diff).
"""
import hashlib
import json
import os
import re
import subprocess
import sys
import textwrap

# ── Input ─────────────────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

command = data.get("tool_input", {}).get("command", "")
cwd     = data.get("cwd", os.getcwd())

if not re.search(r'\bgh\s+pr\s+merge\b', command):
    sys.exit(0)

# ── Encontra project root ──────────────────────────────────────────────────────
def find_project_root(start):
    current = os.path.abspath(start)
    while current != '/':
        if os.path.exists(os.path.join(current, 'mix.exs')):
            return current
        current = os.path.dirname(current)
    return os.path.abspath(start)

project_root = find_project_root(cwd)
reviews_dir  = os.path.join(project_root, '.claude', 'reviews')
os.makedirs(reviews_dir, exist_ok=True)

# ── Obtém PR number e branch atual ────────────────────────────────────────────
pr_num_match = re.search(r'\bgh\s+pr\s+merge\s+(\d+)', command)

try:
    if pr_num_match:
        pr_ref = pr_num_match.group(1)
    else:
        # Usa o PR da branch atual
        pr_ref = ''

    # Obtém info do PR
    pr_info_result = subprocess.run(
        ['gh', 'pr', 'view', pr_ref, '--json', 'number,title,headRefName,url'],
        cwd=project_root, capture_output=True, text=True, timeout=15
    )
    if pr_info_result.returncode != 0:
        # Não há PR aberto — permite (pode ser merge direto)
        sys.exit(0)

    pr_info   = json.loads(pr_info_result.stdout)
    pr_number = pr_info.get('number', 0)
    pr_title  = pr_info.get('title', '')
    pr_branch = pr_info.get('headRefName', 'unknown')
    pr_url    = pr_info.get('url', '')

except Exception:
    sys.exit(0)

# ── Obtém diff do PR ───────────────────────────────────────────────────────────
try:
    diff_result = subprocess.run(
        ['gh', 'pr', 'diff', str(pr_number)],
        cwd=project_root, capture_output=True, text=True, timeout=30
    )
    diff_content = diff_result.stdout.strip()
    if not diff_content:
        # Diff vazio — permite
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": "[code-review ✓] Diff vazio — merge permitido."}}))
        sys.exit(0)
except Exception:
    sys.exit(0)

# ── Verifica cache do review ───────────────────────────────────────────────────
diff_sha    = hashlib.sha256(diff_content.encode()).hexdigest()[:16]
review_file = os.path.join(reviews_dir, f"pr-{pr_number}.json")

if os.path.exists(review_file):
    try:
        with open(review_file) as f:
            cached = json.load(f)
        if cached.get('diff_sha') == diff_sha:
            decision    = cached.get('decision', 'block')
            review_text = cached.get('review_text', '')
            if decision == 'allow':
                print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                    "additionalContext": f"[code-review ✓] Review cacheado aprovado (diff inalterado).\n{review_file}"}}))
            else:
                print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
                    "permissionDecision": "deny",
                    "permissionDecisionReason": (
                        f"CODE REVIEW: Merge bloqueado (review cacheado).\n\n"
                        f"{review_text}\n\n"
                        f"Corrija os problemas e rode novamente.\nReview: {review_file}"
                    )}}))
            sys.exit(0)
    except Exception:
        pass

# ── Verifica se claude CLI está disponível ────────────────────────────────────
try:
    subprocess.run(['claude', '--version'], capture_output=True, timeout=5)
    has_claude = True
except (FileNotFoundError, subprocess.TimeoutExpired):
    has_claude = False

if not has_claude:
    reason = (
        f"CODE REVIEW: Merge bloqueado — review obrigatório.\n\n"
        f"PR #{pr_number}: {pr_title}\n{pr_url}\n\n"
        f"`claude` CLI não encontrado. Faça o review manual:\n\n"
        f"  gh pr diff {pr_number} | claude -p \"Review this diff for bugs, security issues, and code quality. "
        f"Rate issues as CRITICAL, HIGH, MEDIUM, or LOW.\"\n\n"
        f"Após review, crie .claude/reviews/pr-{pr_number}.json com:\n"
        f'  {{"diff_sha": "{diff_sha}", "decision": "allow", "review_text": "LGTM"}}'
    )
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "deny", "permissionDecisionReason": reason}}))
    sys.exit(0)

# ── Roda Claude como reviewer ─────────────────────────────────────────────────
REVIEW_PROMPT = textwrap.dedent(f"""
    You are a senior software engineer reviewing a pull request for the Kura project.
    Kura is built with Elixir (Phoenix + Ecto + SurrealDB) and Swift (iOS).

    PR #{pr_number}: {pr_title}
    Branch: {pr_branch}

    Review the diff below for:
    1. Security vulnerabilities (SQL injection, secret exposure, auth bypass)
    2. Logic errors and edge cases
    3. Performance issues (N+1 queries, missing indexes, unbounded loops)
    4. Test coverage gaps
    5. Elixir/Swift best practices

    Rate each issue as: CRITICAL | HIGH | MEDIUM | LOW

    Output format (strict):
    DECISION: APPROVE or BLOCK
    ISSUES:
    [SEVERITY] description
    ...
    SUMMARY: one-line summary

    If no issues: DECISION: APPROVE / ISSUES: none / SUMMARY: LGTM

    Diff:
    {diff_content[:8000]}
    {"[diff truncated — first 8000 chars shown]" if len(diff_content) > 8000 else ""}
""").strip()

try:
    review_result = subprocess.run(
        ['claude', '-p', REVIEW_PROMPT],
        capture_output=True, text=True, timeout=120,
        cwd=project_root
    )
    review_output = review_result.stdout.strip()
except subprocess.TimeoutExpired:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": "CODE REVIEW: timeout (>2min) — merge permitido com aviso. Rode o review manualmente."}}))
    sys.exit(0)
except Exception as e:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": f"CODE REVIEW: erro ao rodar claude CLI — {e}. Merge permitido com aviso."}}))
    sys.exit(0)

# ── Parse do resultado do review ──────────────────────────────────────────────
decision_match = re.search(r'DECISION:\s*(APPROVE|BLOCK)', review_output, re.IGNORECASE)
decision_str   = decision_match.group(1).upper() if decision_match else 'BLOCK'

# Conta issues por severidade
critical_issues = re.findall(r'\[CRITICAL\].*', review_output, re.IGNORECASE)
high_issues     = re.findall(r'\[HIGH\].*', review_output, re.IGNORECASE)
medium_issues   = re.findall(r'\[MEDIUM\].*', review_output, re.IGNORECASE)
low_issues      = re.findall(r'\[LOW\].*', review_output, re.IGNORECASE)

blocking = critical_issues + high_issues
should_block = decision_str == 'BLOCK' or bool(blocking)
final_decision = 'block' if should_block else 'allow'

# Salva review no cache
review_cache = {
    'diff_sha':    diff_sha,
    'pr_number':   pr_number,
    'pr_title':    pr_title,
    'decision':    final_decision,
    'review_text': review_output,
}
try:
    with open(review_file, 'w') as f:
        json.dump(review_cache, f, ensure_ascii=False, indent=2)
except Exception:
    pass

# ── Output ────────────────────────────────────────────────────────────────────
summary_match = re.search(r'SUMMARY:\s*(.+)', review_output, re.IGNORECASE)
summary = summary_match.group(1).strip() if summary_match else ''

if should_block:
    blocking_lines = "\n".join(f"  {i}" for i in (critical_issues + high_issues)[:10])
    reason = (
        f"CODE REVIEW: Merge bloqueado — issues críticos encontrados.\n\n"
        f"PR #{pr_number}: {pr_title}\n"
        f"CRITICAL: {len(critical_issues)} | HIGH: {len(high_issues)} | "
        f"MEDIUM: {len(medium_issues)} | LOW: {len(low_issues)}\n\n"
        f"Bloqueantes:\n{blocking_lines}\n\n"
        f"Review completo em: {review_file}\n\n"
        f"Corrija os problemas e rode novamente (o review será refeito)."
    )
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "deny", "permissionDecisionReason": reason}}))
else:
    warn = ""
    if medium_issues or low_issues:
        warn = (
            f"\n\n⚠️  {len(medium_issues)} MEDIUM + {len(low_issues)} LOW — "
            f"revise em {review_file}"
        )
    ctx = (
        f"[code-review ✓] PR #{pr_number} aprovado. {summary}{warn}\n"
        f"Review: {review_file}"
    )
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow", "additionalContext": ctx}}))

sys.exit(0)
