#!/usr/bin/env python3
"""
Hook #14 — SonarCloud (PreToolUse: Bash)
Kura Project

Roda sonar-scanner antes de criar PR via gh pr create.
Aguarda o Quality Gate do SonarCloud e bloqueia se falhar.

Quality Gate verifica:
  - Cobertura de testes (mínimo configurado no SonarCloud)
  - Duplicação de código
  - Code smells
  - Bugs e vulnerabilidades
  - Security hotspots

Requer:
  - sonar-scanner instalado (brew install sonar-scanner)
  - SONAR_TOKEN em env ou .env
  - sonar-project.properties no root do projeto

sonar-project.properties mínimo:
  sonar.projectKey=bardi_kura
  sonar.organization=bardi
  sonar.host.url=https://sonarcloud.io
  sonar.sources=lib
  sonar.tests=test
  sonar.elixir.coverage.reportPath=cover/excoveralls.json
"""
import json
import os
import re
import subprocess
import sys
import time
import urllib.request
import urllib.error

# ── Input ─────────────────────────────────────────────────────────────────────
try:
    data = json.load(sys.stdin)
except Exception:
    sys.exit(0)

if data.get("tool_name") != "Bash":
    sys.exit(0)

command = data.get("tool_input", {}).get("command", "")
cwd     = data.get("cwd", os.getcwd())

# Só dispara em gh pr create
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

# ── Verifica pré-requisitos ───────────────────────────────────────────────────
def allow(msg):
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": msg,
        }
    }))
    sys.exit(0)

def deny(msg):
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": msg,
        }
    }))
    sys.exit(0)

# sonar-scanner instalado?
try:
    subprocess.run(['sonar-scanner', '--version'], capture_output=True, timeout=10)
except FileNotFoundError:
    allow("SONARCLOUD: sonar-scanner não instalado — análise pulada.\nInstale: brew install sonar-scanner")

# sonar-project.properties existe?
if not os.path.exists(props_file):
    allow(
        "SONARCLOUD: sonar-project.properties não encontrado — análise pulada.\n"
        "Crie o arquivo no root do projeto (veja o docstring deste hook)."
    )

# SONAR_TOKEN disponível?
sonar_token = os.environ.get('SONAR_TOKEN', '')
if not sonar_token:
    # Tenta ler do .env
    env_file = os.path.join(project_root, '.env')
    if os.path.exists(env_file):
        with open(env_file) as f:
            for line in f:
                m = re.match(r'^SONAR_TOKEN\s*=\s*(.+)$', line.strip())
                if m:
                    sonar_token = m.group(1).strip().strip('"\'')
                    break

if not sonar_token:
    allow("SONARCLOUD: SONAR_TOKEN não configurado — análise pulada.\nDefina SONAR_TOKEN no .env ou no ambiente.")

# ── Lê project key e organization do properties ───────────────────────────────
project_key  = None
organization = None
with open(props_file) as f:
    for line in f:
        m = re.match(r'^sonar\.projectKey\s*=\s*(.+)$', line.strip())
        if m:
            project_key = m.group(1).strip()
        m = re.match(r'^sonar\.organization\s*=\s*(.+)$', line.strip())
        if m:
            organization = m.group(1).strip()

if not project_key:
    allow("SONARCLOUD: sonar.projectKey não encontrado em sonar-project.properties.")

# ── Gera cobertura antes do scan (se excoveralls estiver disponível) ──────────
coverage_report = os.path.join(project_root, 'cover', 'excoveralls.json')
if not os.path.exists(coverage_report):
    try:
        subprocess.run(
            ['mix', 'coveralls.json'],
            cwd=project_root,
            capture_output=True,
            timeout=120,
        )
    except Exception:
        pass  # Continua mesmo sem cobertura

# ── Roda sonar-scanner ────────────────────────────────────────────────────────
try:
    scan_result = subprocess.run(
        ['sonar-scanner', f'-Dsonar.token={sonar_token}'],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=180,
        env={**os.environ, 'SONAR_TOKEN': sonar_token},
    )

    if scan_result.returncode != 0:
        err = (scan_result.stderr or scan_result.stdout)[:500]
        allow(f"SONARCLOUD: scan falhou (exit {scan_result.returncode}) — PR permitido.\n{err}")

    # Extrai task ID do output para aguardar resultado
    task_match = re.search(r'task\?id=([a-zA-Z0-9_-]+)', scan_result.stdout + scan_result.stderr)
    task_id    = task_match.group(1) if task_match else None

except subprocess.TimeoutExpired:
    allow("SONARCLOUD: scan timeout (>3min) — PR permitido. Verifique o SonarCloud manualmente.")
except Exception as e:
    allow(f"SONARCLOUD: erro ao rodar scan — {e}")

# ── Aguarda Quality Gate ──────────────────────────────────────────────────────
def sonar_api(path):
    url = f"https://sonarcloud.io/api/{path}"
    req = urllib.request.Request(url)
    req.add_header('Authorization', f'Bearer {sonar_token}')
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            return json.loads(resp.read())
    except Exception:
        return None

# Aguarda task completar (máx 90s)
if task_id:
    for _ in range(18):  # 18 × 5s = 90s
        time.sleep(5)
        task = sonar_api(f"ce/task?id={task_id}")
        if task and task.get('task', {}).get('status') in ('SUCCESS', 'FAILED', 'CANCELLED'):
            break

# Verifica Quality Gate
qg = sonar_api(f"qualitygates/project_status?projectKey={project_key}")

if not qg:
    allow("SONARCLOUD: não foi possível verificar Quality Gate — PR permitido.")

status = qg.get('projectStatus', {}).get('status', 'NONE')

if status == 'OK':
    print(json.dumps({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "allow",
            "additionalContext": f"[sonarcloud ✓] Quality Gate OK — PR liberado.",
        }
    }))

elif status in ('ERROR', 'WARN'):
    # Extrai condições que falharam
    conditions = qg.get('projectStatus', {}).get('conditions', [])
    failed     = [c for c in conditions if c.get('status') in ('ERROR', 'WARN')]

    lines = []
    for c in failed[:10]:
        metric   = c.get('metricKey', '').replace('_', ' ')
        actual   = c.get('actualValue', '?')
        thresh   = c.get('errorThreshold', '?')
        cstatus  = c.get('status', '')
        lines.append(f"  {'✗' if cstatus == 'ERROR' else '⚠'} {metric}: {actual} (limite: {thresh})")

    cond_text = "\n".join(lines) if lines else "  Verifique o dashboard do SonarCloud."
    url_link  = f"https://sonarcloud.io/project/overview?id={project_key}"

    deny(
        f"SONARCLOUD: Quality Gate {'FALHOU' if status == 'ERROR' else 'com AVISO'} — PR bloqueado.\n\n"
        f"Condições com problema:\n{cond_text}\n\n"
        f"Dashboard: {url_link}\n"
        f"Corrija os problemas e tente novamente."
    )
else:
    allow(f"SONARCLOUD: status '{status}' — PR permitido. Verifique o dashboard.")

sys.exit(0)
