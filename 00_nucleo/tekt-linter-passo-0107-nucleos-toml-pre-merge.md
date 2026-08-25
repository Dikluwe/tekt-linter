# Passo operacional 0107 — migrar Núcleos Tekt de `.tekt` para `.toml`

> **Estado:** ESCRITO — não executado  
> **Data:** 2026-08-25  
> **Branch:** `codex/tekt-nucleus-artifact`  
> **Baseline:** `b4587141775f5e5876eafbe55c471a72f33ce4ef`  
> **Dependência:** P0105 fechado e P0106 `READY WITH RESIDUAL AUDIT`  
> **Objetivo terminal:** deixar o branch apto ao fechamento pré-merge

## 1. Decisão

Preservar **Núcleo Tekt** como conceito arquitetural e substituir apenas sua identidade
física: arquivos sob o namespace canônico passam de `*.tekt` para `*.toml`.

```text
00_nucleo/prompts/_nuclei/**/*.toml
```

O discriminador interno continua obrigatório:

```toml
tekt = 1
kind = "nucleus"
```

Não adotar `.tekt.toml`, alias duplo ou autodetecção por conteúdo. A extensão conhecida
reduz manutenção/tooling; o diretório `_nuclei` e `kind = "nucleus"` preservam a distinção
semântica em relação a outros TOML do projeto e aos prompts Markdown.

## 2. Escopo fechado

Este passo deve:

1. revisar ADR-0022 para registrar `.toml` como representação normativa;
2. atualizar contrato, parser, walker, V26, hashing, resselo e diagnósticos;
3. migrar fixtures, vetores L0, prompts, assessments e documentação vigente;
4. renomear todo Núcleo Tekt real/fixture de `.tekt` para `.toml`;
5. atualizar paths e pins transitivos afetados;
6. provar que a semântica, os bytes TOML e o algoritmo de hash não mudaram além do path;
7. fechar regressões e produzir parecer pré-merge.

Este passo não deve:

- mudar o conceito de claim, modalidade, dependência ou DAG;
- mudar V15, a bijeção prompt↔código ou permitir código→núcleo;
- criar certificação, DSL, macros, execução ou descoberta fora de `_nuclei`;
- aceitar qualquer `.toml` como Núcleo Tekt;
- manter `.tekt` como formato legado silencioso;
- modificar Tekt, Bateia ou tekt-cargo-dsm;
- fazer merge antes do fechamento segregado.

## 3. L0 hash-pinned

Congelar no Assessment 0035, antes de writes, os bytes e hashes abaixo:

| Insumo | SHA-256 |
|---|---|
| `00_nucleo/adr/0022-nucleos-tekt-compartilhamento-l0.md` | `5652cc708edbd5bf943904dc20cd3caad4129418a909f72e8d31f241808db53e` |
| `00_nucleo/prompts/nucleus-artifact.md` | `0419750faf2b6e5b58ee879b6acb6d3e391550320cd1fb46bcde68529b3f4650` |
| `00_nucleo/prompts/rules/nucleus-integrity.md` | `39b39f80e620b480827ec7606daec31b260338e14eecb0866c939ad9b8667525` |
| `01_core/rules/nucleus_integrity.rs` | `039c2797b2073d880a278b2a9d5ac968bc1e49dbeadc7ad75288bc93df7da2c6` |
| `03_infra/nucleus.rs` | `2d3fc6f3e724b7dc2b91656273b8001a97223d81bd213a53dda3a4ff537c9e0b` |
| `03_infra/prompt_walker.rs` | `8ba02452a363a6b495eae35581b745a4a9b0d91c3360a0fd350bdba19b779c3b` |
| `02_shell/fix_hashes.rs` | `c36c43b16a1d84f4ee2d10ab84f6843904cb5f65f1570d694bd0b083078e45cd` |
| `00_nucleo/assessments/0033-nucleus-artifact.md` | `3618e3ee1da168c0b99f12328871e59cad9b66b75b4e0f571febf3358bc67e33` |
| `00_nucleo/tekt-linter-passo-0105-artefato-nucleo-tekt.md` | `0af7879e212715042351bd7bd45071f5486bb7bc8f2dccce5be10493f93ebf22` |

Inventariar adicionalmente, com hash e classificação, toda ocorrência produtiva ou fixture
de `.tekt`, `tekt` e `_nuclei`. Ocorrências históricas podem permanecer somente quando
explicitamente marcadas como descrição superseded; documentação normativa deve convergir.

## 4. ADR-0022 Rev. 1

Atualizar a ADR, sem criar uma ADR concorrente:

- estado `aceito — Rev. 1` e vínculo P0107;
- localização `00_nucleo/prompts/_nuclei/**/*.toml`;
- TOML 1.0 estrito como formato e extensão;
- identidade por namespace + schema fechado + `tekt = 1` + `kind = "nucleus"`;
- `.tekt` rejeitado como extensão legada não suportada;
- justificativa: não havia linguagem própria, apenas TOML com extensão proprietária;
- consequência: tooling conhecido e menor custo, sem alteração da semântica arquitetural.

P0105 permanece registro histórico da decisão original e deve receber uma nota curta de
supersessão para a extensão, apontando ADR-0022 Rev. 1/P0107; não reescrever sua cronologia.

## 5. Contrato normativo após a migração

1. Apenas arquivos regulares UTF-8 `*.toml` sob `_nuclei` entram no inventário de núcleos.
2. Todo arquivo inventariado deve possuir schema fechado, `tekt = 1` e
   `kind = "nucleus"`; TOML desconhecido no namespace falha V26.
3. `.toml` fora de `_nuclei` não é Núcleo Tekt e não entra no walker de núcleos.
4. `.tekt` sob `_nuclei` é V26 com diagnóstico de extensão legada; não é ignorado.
5. `.tekt` fora de `_nuclei` continua erro de inventário conforme a política vigente,
   com mensagem de migração, para evitar ghost artifacts.
6. `@prompt .../*.toml` continua inválido quando o alvo é `_nuclei`; código nunca aponta
   diretamente a Núcleo Tekt.
7. Referências em prompts usam path lógico canônico terminado em `.toml` e SHA-256 completo.
8. O SHA-256 do núcleo continua cobrindo os bytes integrais do TOML; o digest efetivo
   continua incluindo dependências transitivas e identidade de path.
9. Renomear `.tekt`→`.toml` muda intencionalmente a identidade lógica/path e portanto
   invalida pins consumers, mas não altera a claim ou seu schema.
10. Prompt sem núcleo mantém hash bit a bit idêntico ao baseline.

## 6. Protocolo segregado

### A — inventário e classificação

Criar `00_nucleo/assessments/0035-nucleus-toml-migration.md` com:

- baseline e L0 hash-pinned;
- lista exata de arquivos/fixtures/strings afetados;
- separação `PRODUCTION`, `FIXTURE`, `NORMATIVE-DOC`, `HISTORICAL-DOC`;
- mapa old path→new path;
- previsão da superfície do resselo;
- classificação RED/gate/SPEC-GAP.

Nenhum write produtivo antes de A congelado.

### B1 — gate de formato

Atualizar/criar gate cego que prove:

- TOML mínimo válido com `.toml` é aceito;
- schema desconhecido, versão/kind inválidos e limites continuam fail-closed;
- `.tekt`, `.tekt.toml`, `.md` e extensão ausente são rejeitados;
- TOML fora de `_nuclei` não é descoberto como núcleo;
- arquivo `.toml` genérico dentro de `_nuclei` não passa sem o discriminador.

### B2 — gate de grafo e identidade

Provar in-memory que:

- dependências canônicas terminam em `.toml`;
- missing, cycle e orphan mantêm ordenação/diagnóstico determinísticos;
- path textual participa da identidade e `a.toml` ≠ `A.toml`;
- substituir somente `.tekt` por `.toml` preserva claims/dependencies parseadas.

### B3 — gate de wiring e V26

Provar em fixture real:

- dois prompts 1:1 podem consumir o mesmo núcleo `.toml` sem V15;
- código→núcleo `.toml` falha;
- `.tekt` legado produz exatamente V26 e não some do inventário;
- prompt walker não classifica `_nuclei/*.toml` como prompt;
- V1/V5/V7/V15 mantêm suas responsabilidades.

### B4 — gate de hash e transação

Fixar vetores antes de produção:

- prompt sem núcleo: digest idêntico ao baseline;
- conteúdo TOML idêntico com path antigo/novo: raw hash idêntico, identidade/digest efetivo
  diferente somente onde o path é normativo;
- alteração de um núcleo invalida todos os consumers transitivos;
- dry-run e execução usam o mesmo plano;
- falha de preflight/rename/pin escreve zero bytes;
- resselo toca apenas paths/pins/metadados autorizados.

Congelar B1–B4 em commit próprio antes da implementação.

## 7. Implementação

Executar em um lote produtivo único, após B1–B4:

1. trocar constantes, filtros de extensão, validação de path e mensagens para `.toml`;
2. preservar o parser TOML/schema/limites atuais;
3. renomear fixtures e artefatos reais com operação rastreável no Git;
4. atualizar todas as referências de dependência e pins de prompt;
5. atualizar ADR-0022, prompts proprietários, Assessment 0033 e relatório P0105;
6. deixar P0105 histórico com nota de supersessão, não apagar evidência;
7. executar `fix-hashes --dry-run`; exigir superfície igual à previsão de A;
8. executar o resselo real uma vez;
9. repetir dry-run e exigir `Nothing to fix`.

Não executar `fix-hashes` antes de o inventário V26 ficar íntegro.

## 8. RED, gate e SPEC-GAP

Classificar todo achado:

| Classe | Condição |
|---|---|
| RED | `.tekt` ignorado, `.toml` genérico aceito, código→núcleo permitido, hash/transação divergente |
| gate | teste/documento desatualizado sem mudança de contrato |
| SPEC-GAP | extensão/namespace/compatibilidade não decididos por ADR-0022 Rev. 1 |

RED exige correção e reteste. SPEC-GAP exige decisão explícita na ADR antes do write.
Nenhum item pode ser rebaixado apenas para permitir merge.

## 9. Fechamento pré-merge

Executar e registrar:

```bash
cargo fmt --check
cargo test
cargo run -- . --fix-hashes --dry-run
cargo run -- .
git diff --check
git status --short
```

Critérios terminais:

- B1–B4 verdes;
- V1/V5/V7/V15/V26 sem regressão estrutural;
- zero `.tekt` produtivo ou fixture não classificado;
- zero referência normativa ativa a `**/*.tekt`;
- resselo idempotente;
- diff funcional limitado à migração declarada;
- Assessment 0035 fechado como `READY WITH RESIDUAL AUDIT` ou mais forte;
- worktree limpo e commits segregados;
- parecer explícito de merge, sem realizar o merge no mesmo commit de implementação.

Se `cargo fmt --check` falhar apenas por dívida histórica já presente no baseline, registrar
o delta exato e exigir que nenhum arquivo tocado por P0107 introduza nova divergência.

## 10. Commits previstos

1. `audit(p0107): freeze nucleus toml migration surface`
2. `test(p0107): freeze toml nucleus migration gates`
3. `refactor(p0107): migrate tekt nuclei to toml`
4. `docs(p0107): revise nucleus adr and migration records`
5. `chore(p0107): reseal toml nucleus graph`
6. `docs(p0107): close pre-merge nucleus migration`

Cada commit deve fechar seu próprio gate; não misturar ADR, implementação e resselo.
