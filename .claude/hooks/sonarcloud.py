#!/usr/bin/env python3
"""
Hook #14 — SonarCloud (PreToolUse: Bash)
Kura Project

Roda sonar-scanner antes de gh pr create e aguarda o resultado
do Quality Gate na SonarCloud. Bloqueia PR se o gate falhar.

Pré-requisitos:
  1. sonar-scanner instalado (brew install sonar-scanner)
  2. SONAR_TOKEN exportado no ambiente
  3. sonar-project.properties no root do projeto

sonar-project.properties mínimo:
  sonar.projectKey=renato-bardi_kura
  sonar.organization=renato-bardi
  sonar.sources=lib
  sonar.tests=test
  sonar.host.url=https://sonarcloud.io
  sonar.elixir.file.suffixes=.ex,.exs
  sonar.exclusions=_build/**,deps/**,priv/repo/migrations/**

Quality Gate bloqueia PR se:
  - Novos bugs introduzidos
  - Novas vulnerabilidades de segurança
  - Code smells acima do limiar configurado
  - Cobertura de testes abaixo do mínimo
"""
import json
import os
import re
import subprocess
import sys
import time
import urllib.request
import urllib.error
import base64

# ── Input ─────────────────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

command = data.get("tool_input", {}).get("command", "")
cwd     = data.get("cwd", os.getcwd())

if not re.search(r'\bgh\s+pr\s+create\b', command):
    sys.exit(0)

# ── Encontra root do projeto ───────────────────────────────────────────────────
def find_project_root(start):
    current = os.path.abspath(start)
    while current != '/':
        if os.path.exists(os.path.join(current, 'mix.exs')):
            return current
        current = os.path.dirname(current)
    return os.path.abspath(start)

project_root = find_project_root(cwd)
props_file   = os.path.join(project_root, 'sonar-project.properties')

# ── Verifica pré-requisitos ────────────────────────────────────────────────────
sonar_token = os.environ.get('SONAR_TOKEN', '')

if not sonar_token:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": (
            "SONARCLOUD: SONAR_TOKEN não encontrado — scan pulado.\n"
            "Configure: export SONAR_TOKEN=seu_token_aqui\n"
            "Gere em: https://sonarcloud.io/account/security"
        )}}))
    sys.exit(0)

if not os.path.exists(props_file):
    # Gera properties mínimo
    try:
        # Tenta inferir projectKey do git remote
        remote = subprocess.run(['git', 'remote', 'get-url', 'origin'],
                                cwd=project_root, capture_output=True, text=True)
        repo_match = re.search(r'github\.com[:/](.+?)(?:\.git)?$', remote.stdout.strip())
        repo_slug  = repo_match.group(1).replace('/', '_') if repo_match else 'renato-bardi_kura'
        org        = repo_slug.split('_')[0] if '_' in repo_slug else 'renato-bardi'

        with open(props_file, 'w') as f:
            f.write(f"""sonar.projectKey={repo_slug}
sonar.organization={org}
sonar.projectName=Kura
sonar.sources=lib
sonar.tests=test
sonar.host.url=https://sonarcloud.io
sonar.elixir.file.suffixes=.ex,.exs
sonar.swift.file.suffixes=.swift
sonar.exclusions=_build/**,deps/**,priv/repo/migrations/**,.claude/**
sonar.coverage.exclusions=test/**
""")
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": (
                f"SONARCLOUD: sonar-project.properties criado em {props_file}.\n"
                "Verifique as configurações antes de criar o PR novamente."
            )}}))
        sys.exit(0)
    except Exception:
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": "SONARCLOUD: sonar-project.properties não encontrado — scan pulado."}}))
        sys.exit(0)

try:
    subprocess.run(['sonar-scanner', '--version'], capture_output=True, timeout=5)
except FileNotFoundError:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": "SONARCLOUD: sonar-scanner não instalado.\nInstale: brew install sonar-scanner"}}))
    sys.exit(0)

# ── Extrai projectKey e organization do properties ────────────────────────────
props = {}
with open(props_file) as f:
    for line in f:
        line = line.strip()
        if '=' in line and not line.startswith('#'):
            k, _, v = line.partition('=')
            props[k.strip()] = v.strip()

project_key = props.get('sonar.projectKey', '')
organization = props.get('sonar.organization', '')

if not project_key:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": "SONARCLOUD: sonar.projectKey não definido em sonar-project.properties."}}))
    sys.exit(0)

# ── Roda sonar-scanner ────────────────────────────────────────────────────────
print(json.dumps({"additionalContext": "SONARCLOUD: Enviando análise para SonarCloud..."}), flush=True)

try:
    scan_result = subprocess.run(
        ['sonar-scanner', f'-Dsonar.token={sonar_token}'],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=300,
        env={**os.environ, 'SONAR_TOKEN': sonar_token},
    )

    scan_output = scan_result.stdout + scan_result.stderr

    # Extrai task URL do output
    task_url_match = re.search(r'More about the report processing at (https?://\S+)', scan_output)

    if scan_result.returncode != 0 or not task_url_match:
        err = scan_output[-800:].strip()
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": f"SONARCLOUD: Scanner falhou — PR permitido com aviso.\n{err}"}}))
        sys.exit(0)

    task_url = task_url_match.group(1)
    # Extrai task ID
    task_id_match = re.search(r'[?&]id=([A-Za-z0-9_-]+)', task_url)
    task_id = task_id_match.group(1) if task_id_match else None

except subprocess.TimeoutExpired:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": "SONARCLOUD: timeout no scanner (>5min) — PR permitido com aviso."}}))
    sys.exit(0)

# ── Aguarda processamento na SonarCloud (poll) ────────────────────────────────
def sonar_request(url):
    token_b64 = base64.b64encode(f"{sonar_token}:".encode()).decode()
    req = urllib.request.Request(url, headers={"Authorization": f"Basic {token_b64}"})
    with urllib.request.urlopen(req, timeout=15) as resp:
        return json.loads(resp.read())

POLL_TIMEOUT = 120  # segundos
POLL_INTERVAL = 5
elapsed = 0
task_status = None

if task_id:
    api_base = props.get('sonar.host.url', 'https://sonarcloud.io')
    while elapsed < POLL_TIMEOUT:
        try:
            task_data   = sonar_request(f"{api_base}/api/ce/task?id={task_id}")
            task_status = task_data.get('task', {}).get('status', '')
            if task_status in ('SUCCESS', 'FAILED', 'CANCELLED'):
                break
        except Exception:
            pass
        time.sleep(POLL_INTERVAL)
        elapsed += POLL_INTERVAL

# ── Verifica Quality Gate ─────────────────────────────────────────────────────
if task_status != 'SUCCESS':
    msg = (
        f"SONARCLOUD: Análise {task_status or 'inconclusiva'} após {elapsed}s.\n"
        f"Verifique em: https://sonarcloud.io/project/overview?id={project_key}\n"
        "PR permitido com aviso."
    )
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow", "additionalContext": msg}}))
    sys.exit(0)

try:
    api_base   = props.get('sonar.host.url', 'https://sonarcloud.io')
    gate_data  = sonar_request(
        f"{api_base}/api/qualitygates/project_status"
        f"?projectKey={project_key}"
        + (f"&organization={organization}" if organization else "")
    )
    gate       = gate_data.get('projectStatus', {})
    gate_status = gate.get('status', 'UNKNOWN')  # OK, WARN, ERROR

    conditions = gate.get('conditions', [])
    failed     = [c for c in conditions if c.get('status') == 'ERROR']
    warned     = [c for c in conditions if c.get('status') == 'WARN']

    sonar_url  = f"https://sonarcloud.io/project/overview?id={project_key}"

    if gate_status == 'OK':
        warn_note = ""
        if warned:
            warn_note = f"\n⚠️  {len(warned)} condição(ões) em aviso — revise em {sonar_url}"
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": f"[sonarcloud ✓] Quality Gate passou. PR liberado.{warn_note}"}}))

    else:
        def fmt_condition(c):
            metric  = c.get('metricKey', '').replace('_', ' ')
            actual  = c.get('actualValue', '?')
            error   = c.get('errorThreshold', '?')
            return f"  • {metric}: {actual} (limiar: {error})"

        failed_lines = "\n".join(fmt_condition(c) for c in failed[:8])
        reason = (
            f"SONARCLOUD: Quality Gate FALHOU — PR bloqueado.\n\n"
            f"Condições com falha:\n{failed_lines}\n\n"
            f"Detalhes: {sonar_url}\n\n"
            f"Corrija os problemas e rode o scan novamente."
        )
        print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
            "permissionDecision": "deny", "permissionDecisionReason": reason}}))

except Exception as e:
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "additionalContext": f"SONARCLOUD: erro ao checar Quality Gate — {e}\nVerifique: https://sonarcloud.io"}}))

sys.exit(0)
