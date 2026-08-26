# Assessment P0117 — V1 e caminhos absolutos simbólicos

**Data:** 2026-08-26  
**Baseline:** `4457695`  
**Regime:** protocolo completo executado sem atestação de isolamento entre agentes  
**Veredito:** `APPROVED FOR MERGE — V1 TRI-STATE FIX`

## Obrigação congelada

V1 representa somente header ausente ou prompt realmente ausente. Falha de confinamento,
permissão, raiz, symlink, I/O ou budget é infraestrutura Fatal e nunca pode ser convertida
em ausência. A política que rejeita raízes simbólicas permanece inalterada.

Reprodução física:

- canônica: `/repos/Antigravity/tekt-linter`;
- alias: `/home/dikluwe/Documentos/Antigravity/tekt-linter`;
- ambas resolvem para `/repos/Antigravity/tekt-linter`;
- baseline canônica: zero V1;
- baseline alias: falsos V1 para os consumidores de prompts.

## Contrato e transporte

`PromptReader` passou de `bool`/`Option` para `PromptPresence` e
`PromptHashResolution`. `ParsedFile` e `HasPromptFilesystem` transportam o estado fechado.
Rust, TypeScript, Python, C, C++, Zig, Go, Java e Elixir usam a mesma função
`resolve_prompt`, consultam o reader somente com header e só calculam hash quando o prompt
está presente.

V1 aceita apenas `Missing`. `InfrastructureFailure` é projetado como V0 Fatal e falhas
estruturais idênticas da raiz são deduplicadas antes da apresentação.

## Matriz observada

| Caso | V1 | Infraestrutura | Exit |
|---|---:|---:|---:|
| raiz absoluta canônica | 0 | 0 | 0 |
| alias absoluto com ancestral simbólico | 0 | 1 V0 Fatal | 1 |
| prompt realmente ausente sob ancestral válido | 1 | 0 | 1 |
| ancestral do prompt ausente | 0 | falha tipada | 1 |
| prompt simbólico quebrado | 0 | falha tipada | 1 |
| arquivo acima de 10 MiB | 0 | falha tipada | 1 |

O teste black-box `v1_absolute_path_assessment` executa o binário real em caminho absoluto
canônico e alias temporário, verificando text e SARIF. A reprodução local real retornou um
único V0 Fatal no alias e nenhuma ocorrência V1 ou mensagem “prompt inexistente”.

## Mutação e ataques

A primeira auditoria independente rejeitou a implementação. Ela encontrou classificação
incorreta de caminho inseguro ausente, caches separados de presença/hash, apagamento de
falha no `fix-hashes`, ausência de preflight global e deduplicação excessiva. A segunda
auditoria encontrou ainda a janela `Present → remoção → Missing`. O cache passou a congelar
a resolução completa na primeira consulta, e um teste cobre explicitamente essa direção.
O código foi devolvido aos mesmos revisores; ambas as rejeições fazem parte da evidência.

Recibos após a remediação:

- `03_infra/prompt_io.rs`: 17 gerados, 17 válidos mortos, zero sobreviventes;
- contrato, regras V0/V1 e `03_infra/prompt_reader.rs`: 23 gerados, 10 válidos mortos,
  13 inviáveis em compilação e zero sobreviventes válidos;
- `02_shell/fix_hashes.rs`: 35 gerados, 11 mortos, 4 inviáveis e 20 sobreviventes.

Na terceira campanha, os mutantes de `FixHashesUseCase::plan` que poderiam apagar ou
alterar o novo resultado foram mortos. Os 20 sobreviventes estão em apresentação,
`FixBatchError`, `FixBatchResult`, `plan_bijective`, `validate_bijective` e
`format_results`; portanto não se declara score total para `fix_hashes.rs`. Eles são dívida
de oráculo preexistente e não evidência da obrigação P0117.

As APIs `is_some`, `is_none`, `unwrap` e `into_option` de `PromptHashResolution` foram
removidas: cada consumidor agora precisa distinguir explicitamente `Hash`, `Missing` e
`InfrastructureFailure`.

## Gates e limitações

- `cargo fmt --all --check`: passou;
- `cargo test --all-targets`: passou (650 testes unitários e todos os testes de integração);
- ratchet de complexidade P0109: 2/2 passou;
- `git diff --check`: passou;
- black-box text/SARIF de P0117: 6/6 passou;
- reprodução canônica: exit 0, zero violações;
- reprodução pelo alias absoluto: exit 1, exatamente um V0 Fatal e zero V1;
- `--fix-hashes --dry-run`: `Nothing to fix`;
- `cargo build --release`: passou;
- identidade e smoke test do binário instalado: pendentes do merge.

Os comandos e denominadores das três campanhas estão versionados em
`lab/p0117_mutation_receipt.md`.

A autoria ocorreu numa única sessão com fases e artefatos separados; portanto o resultado
é executado sem atestação de isolamento, não “segregado e atestado”. O protocolo comprova
somente a classificação observável de acesso a prompts descrita aqui, não equivalência
funcional geral do linter.

Reauditoria final: adversário e verificador aprovaram o diff após duas rejeições materiais
e respectivas remediações. A aprovação autoriza commit e merge, mas não altera o limite de
isolamento histórico acima.
