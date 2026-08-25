# Assessment 0025/B1 — inventário estrutural

**Papel:** B1, produção/testes em somente leitura
**Resultado:** PASS factual
**Autoridade derivada da produção:** não
**Recomendação de P0097:** proibida e não realizada

## Seams encontradas

### S1 — extração de observáveis Rust e snapshot

- Produtor L3: `03_infra/refinement_extractor.rs`, de
  `load_observable_specs[_from_bytes]` até `write_snapshot_atomic`.
- Consumidores: comandos Snapshot/Seal/RefineRevisions em `04_wiring/main.rs` e extração
  de conteúdo Git em `03_infra/git_refinement.rs`.
- Efeitos: TOML, paths, fontes, tree-sitter query e escrita/rename.
- Evidência: quatro unitários, `tests/refinement_cli.rs` e CLIs revisionais transitivos.
- Lacunas factuais: sem Assessment dedicado; DTO aberto; sem cotas visíveis; erros de
  path/leitura colapsados em MissingObservable; temporário fixo sem `create_new`, sync,
  preservação de permissões ou limpeza explícita.

### S2 — extração imutável via Git/subprocesso

- Produtor L3: `03_infra/git_refinement.rs`, incluindo self-contained DB, resolução,
  `ls-tree`, `cat-file` e snapshot revisional.
- Consumidores L4: Seal e RefineRevisions.
- Efeitos: `.git`, subprocesso, threads, timeout/kill e objetos.
- Evidência: `git_refinement_assessment`, `refinement_cli` e unitários.
- Lacunas factuais: consumidores duplicam orquestração; Git real em vez de port injetável;
  protocolos hostis de subprocesso não possuem teste direto simulado.

### S3 — manifesto, recibo e publicação de selo

- Produtor L3: `03_infra/refinement_seal.rs`.
- Consumidor L4: bloco Seal em `04_wiring/main.rs`.
- Efeitos: leitura/canonicalização, hashes, serialização, criação e rename.
- Evidência: três unitários e quinze casos em `segregated_materialization_cli`.
- Lacunas factuais: sem Assessment dedicado; leituras sem limites/regularidade explícita;
  temporário PID-based sem preservação de modo/sync; composição/exit monolíticos em L4.

### S4 — orquestração principal do lint

- Coordenador L4: montagem, `run_pipeline`, `run_checks`, pós-reduce e branches mutantes em
  `04_wiring/main.rs`.
- Consumidores: CLI/exit e adapters L3 de hash/snapshot.
- Efeitos: filesystem, Rayon, stdout/stderr, exit e rerun após escrita.
- Evidência fragmentada pelos Assessments de walker, parser, regras, apresentação e
  writers.
- Lacunas factuais: sem gate direto da composição completa; precedência entre
  `emit_resolution`, fix/update, prompt scan, pós-reduce, fail level e quiet não possui
  Assessment dedicado; montagem dos nove parsers é duplicada nos reruns.

### S5 — nove parsers concretos para IR comum

- Produtores L3: parsers Rust, TS, Python, C, C++, Zig, Go, Java e Elixir.
- Contrato L1: `LanguageParser`; composição/consumo em L4 e regras L1.
- Efeitos: tree-sitter, prompt/hash/snapshot e resolução de registry/root em variantes.
- Evidência: gates MultiParser provam slots/propagação; unitários e fixtures de parsers são
  acoplados.
- Lacunas factuais: nenhum Assessment dedicado à fidelidade IR por linguagem; gates das
  regras usam IR sintético e gates de roteamento usam spies.

### S6 — argumentos/preflight e apresentação geral

- Produtor L2: `02_shell/cli.rs` (`validate_args`, `EnabledChecks`, formatadores, sort e
  `should_fail`).
- Consumidor L4: parse, validação, seleção, output e exit.
- Evidência: unitários, `shell_presentation_assessment` e CLIs transitivos.
- Lacunas factuais: gate de apresentação não cobre precedência global dos subcomandos nem
  toda matriz clap/config/quiet/dry-run/emit-resolution.

## Busca reversa

1. `compare_refinement` recebe fatos de três produtores: loader explícito, extractor local
   e extractor Git.
2. Todas as regras recebem IR dos nove parsers concretos; gates de regra constroem IR
   sintético.
3. Writers L3 são alcançados por adapters L4 e planos L2; gates L2 usam spies e testes CLI
   são a confrontação fim-a-fim restante.

Nenhum arquivo foi alterado.
