#!/usr/bin/env python3
"""
Hook #6 — Swift Format (PostToolUse: Edit)
Kura Project

Roda swift-format --in-place no arquivo .swift editado.
Garante estilo consistente em todo código SwiftUI sem intervenção manual.

Requer: swift-format instalado (brew install swift-format)
Config: .swift-format no root do projeto (gerado automaticamente se ausente)
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

if not file_path.endswith('.swift'):
    sys.exit(0)

# Ignora arquivos gerados
SKIP_PATTERNS = [r'\.build/', r'DerivedData/', r'Pods/', r'\.generated\.swift$']
if any(re.search(p, file_path) for p in SKIP_PATTERNS):
    sys.exit(0)

abs_file = os.path.abspath(file_path)
if not os.path.exists(abs_file):
    sys.exit(0)

# ── Encontra root do projeto (onde está Package.swift ou .xcodeproj) ──────────
def find_project_root(start):
    current = os.path.abspath(start if os.path.isdir(start) else os.path.dirname(start))
    while current != '/':
        has_package  = os.path.exists(os.path.join(current, 'Package.swift'))
        has_xcodeproj = any(f.endswith('.xcodeproj') for f in os.listdir(current))
        has_swiftfmt  = os.path.exists(os.path.join(current, '.swift-format'))
        if has_package or has_xcodeproj or has_swiftfmt:
            return current
        current = os.path.dirname(current)
    return os.path.dirname(abs_file)

project_root = find_project_root(abs_file)
rel_file     = os.path.relpath(abs_file, project_root)

# ── Garante config .swift-format existe ──────────────────────────────────────
config_path = os.path.join(project_root, '.swift-format')
if not os.path.exists(config_path):
    default_config = {
        "version": 1,
        "lineLength": 120,
        "indentation": {"spaces": 4},
        "maximumBlankLines": 1,
        "respectsExistingLineBreaks": True,
        "lineBreakBeforeControlFlowKeywords": True,
        "lineBreakBeforeEachArgument": False,
        "fileScopedDeclarationPrivacy": {"accessLevel": "private"},
        "indentConditionalCompilationBlocks": True,
        "indentSwitchCaseLabels": False,
        "noAssignmentInExpressions": {"allowedFunctions": []},
        "prioritizeKeepingFunctionOutputTogether": True
    }
    import json as _json
    with open(config_path, 'w') as f:
        _json.dump(default_config, f, indent=2)

# ── Roda swift-format --in-place ─────────────────────────────────────────────
try:
    result = subprocess.run(
        ["swift-format", "--in-place", "--configuration", config_path, abs_file],
        cwd=project_root,
        capture_output=True,
        text=True,
        timeout=30,
    )

    if result.returncode == 0:
        context = f"[swift-format ✓] {rel_file} — formatado."
    else:
        err = (result.stderr or result.stdout).strip()
        context = f"[swift-format ✗] {rel_file}:\n{err}"

    print(json.dumps({"additionalContext": context}))

except subprocess.TimeoutExpired:
    print(json.dumps({"additionalContext": f"swift-format timeout (>30s) em {rel_file}."}))

except FileNotFoundError:
    print(json.dumps({
        "additionalContext": (
            "swift-format nao encontrado.\n"
            "Instale com: brew install swift-format\n"
            "Ou: mint install nicklockwood/SwiftFormat"
        )
    }))

except Exception as e:
    print(json.dumps({"additionalContext": f"Erro ao rodar swift-format: {e}"}))

sys.exit(0)
