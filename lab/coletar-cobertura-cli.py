#!/usr/bin/env python3
# Crystalline Lineage
# @prompt 00_nucleo/prompts/coleta-cobertura-cli.md
# @prompt-hash da104846
# @layer L_lab
# @updated 2026-11-26

import argparse
import json
import os
from pathlib import Path
import re
import subprocess
from tempfile import TemporaryDirectory


RAIZ = Path(__file__).resolve().parents[1]
CHECAGEM = re.compile(
    r"^pub fn (?:check\w*|prompt_access_violation|source_error_to_violation|parse_error_to_violation)\b"
)


def compilar(suites, destino, ambiente, offline):
    comando = ["cargo", "test", "--no-run", "--locked", "--message-format=json"]
    if offline:
        comando.append("--offline")
    for suite in suites:
        comando.extend(["--test", suite])
    saida = subprocess.check_output(comando, env=ambiente, cwd=RAIZ, text=True)
    testes = {}
    binario = None
    for linha in saida.splitlines():
        evento = json.loads(linha)
        if evento.get("reason") != "compiler-artifact" or not evento["executable"]:
            continue
        alvo = evento["target"]
        if alvo["kind"] == ["test"] and alvo["name"] in suites:
            testes[alvo["name"]] = (
                Path(alvo["src_path"]).resolve().relative_to(RAIZ).as_posix(),
                evento["executable"],
            )
        if alvo["kind"] == ["bin"] and alvo["name"] == "crystalline-lint" and not evento["profile"]["test"]:
            binario = evento["executable"]
    if set(testes) != set(suites) or binario is None:
        raise RuntimeError(f"Executáveis dos testes ou da CLI ausentes no build em {destino}")
    return binario, testes


def ferramenta_llvm():
    sysroot = subprocess.check_output(["rustc", "--print", "sysroot"], cwd=RAIZ, text=True).strip()
    versao = subprocess.check_output(["rustc", "-vV"], cwd=RAIZ, text=True)
    host = re.search(r"^host: (.+)$", versao, re.MULTILINE)[1]
    pasta = Path(sysroot) / "lib" / "rustlib" / host / "bin"
    if not all((pasta / nome).is_file() for nome in ("llvm-cov", "llvm-profdata")):
        raise RuntimeError("Instale llvm-tools na toolchain ativa do Rust")
    return pasta


def coletar(suites, destino, temporario, offline):
    llvm = ferramenta_llvm()
    ambiente = {
        **os.environ,
        "CARGO_TARGET_DIR": str(destino),
        "CARGO_ENCODED_RUSTFLAGS": "-C\x1finstrument-coverage",
        "CARGO_INCREMENTAL": "0",
        "CARGO_PROFILE_TEST_DEBUG": "0",
        "LLVM_PROFILE_FILE": str(temporario / "build-%m-%p.profraw"),
    }
    binario, testes = compilar(suites, destino, ambiente, offline)
    ambiente["LLVM_PROFILE_FILE"] = str(temporario / "amostra-%m-%p.profraw")
    subprocess.run([binario, "--help"], env=ambiente, cwd=RAIZ, stdout=subprocess.DEVNULL, check=True)
    amostra, = temporario.glob("amostra-*.profraw")
    assinatura = amostra.name.removeprefix("amostra-").rsplit("-", 1)[0]

    funcoes = {}
    for fonte in sorted((RAIZ / "01_core" / "rules").glob("*.rs")):
        linhas = {
            numero for numero, linha in enumerate(fonte.read_text(encoding="utf-8").splitlines(), 1)
            if CHECAGEM.match(linha)
        }
        if linhas:
            funcoes[fonte] = linhas

    regras = {fonte.relative_to(RAIZ).as_posix(): [] for fonte in funcoes}
    for suite, (caminho, executavel) in sorted(testes.items()):
        perfis_suite = temporario / suite
        perfis_suite.mkdir()
        ambiente["LLVM_PROFILE_FILE"] = str(perfis_suite / "%m-%p.profraw")
        subprocess.run([executavel], env=ambiente, cwd=RAIZ, check=True)
        perfis = sorted(perfis_suite.glob(f"{assinatura}-*.profraw"))
        if not perfis:
            continue
        perfil = perfis_suite / "cli.profdata"
        subprocess.run([llvm / "llvm-profdata", "merge", "-sparse", "-o", perfil, *perfis], check=True)
        dados = json.loads(subprocess.check_output(
            [llvm / "llvm-cov", "export", f"--instr-profile={perfil}", "--skip-expansions",
             binario, "--sources", *funcoes], cwd=RAIZ
        ))
        exercitadas = set()
        for funcao in dados["data"][0]["functions"]:
            fonte = (RAIZ / funcao["filenames"][0]).resolve()
            if funcao["count"] and funcao["regions"][0][0] in funcoes.get(fonte, ()):
                exercitadas.add(fonte.relative_to(RAIZ).as_posix())
        for regra in exercitadas:
            regras[regra].append(caminho)

    revisao = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=RAIZ, text=True).strip()
    return {
        "versao": 1,
        "revisao": revisao,
        "testes": sorted(caminho for caminho, _ in testes.values()),
        "regras": {regra: sorted(suites) for regra, suites in regras.items()},
    }


def main():
    parser = argparse.ArgumentParser(description="Coleta quais testes executam checagens pela CLI.")
    parser.add_argument("suites", nargs="+", help="Nomes dos alvos de integração do Cargo")
    parser.add_argument("-o", "--saida", type=Path, required=True)
    parser.add_argument("--offline", action="store_true", help="Usa o cache local do Cargo")
    args = parser.parse_args()
    destino = Path(os.environ.get("CARGO_TARGET_DIR", RAIZ / "target")) / "cobertura-cli"
    with TemporaryDirectory(prefix="tekt-cobertura-cli-") as temporario:
        dados = coletar(sorted(set(args.suites)), destino.resolve(), Path(temporario), args.offline)
    args.saida.parent.mkdir(parents=True, exist_ok=True)
    args.saida.write_text(json.dumps(dados, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
