# ADR-0016: Regras de decisão mecânica (match) no tronco do crystalline-lint, com nível `info` e diagnósticos na nomenclatura da linguagem

**Status**: PROPOSTO — bloqueado pelas questões empíricas em «Questões em aberto»
**Data**: 2026-08-14
**Documentos relacionados**: série de medições AST do typst-crystalline (varredura syn de 2026-08, regras R1–R7); ADR-0001 (tree-sitter como IR); ADR-0009 (suporte TypeScript/Python)

> Nota de numeração: este ADR é o 0016 — o README do repo estava
> desactualizado (listava ADRs só até 0009). A Fase E do passo de
> implementação inclui auditoria documental para saneamento e prevenção.

## Contexto

A investigação sobre cobertura de decisões no typst-crystalline (2026-08) produziu, por medição directa com parser `syn`:

1. A árvore de decisão de domínio do Crystalline é preservada face ao Vanilla (1.425 vs 1.423 `match` de domínio, Δ = 0,14%), mas **291 braços wildcard executam lógica de fallback não-erro**, dos quais 26 são saturação arbitrária silenciosa em enums fechados de domínio (`_ => Unit::Percent`, `_ => Assoc::Left`, …) — uma variante nova do enum é adoptada sem erro de compilação e gera output errado.
2. A triagem desses 291 casos demonstrou que a classificação correcta (isento / DENY / WARN / INFO) é decidível por **proxies sintácticos** — forma do scrutinee, ligação do wildcard, forma do corpo do braço — sem informação de tipos.
3. O crystalline-lint é multi-linguagem (Rust, TypeScript, Python, com Go/Zig no horizonte) e usa tree-sitter como IR (ADR-0001). O seu modelo de severidades tem apenas `fatal | error | warning`; não existe nível para métricas/advisories.

Forças em tensão: (a) as regras R1–R5 são sobre decisões exaustivas, um conceito com análogo em todas as linguagens-alvo (`match`/`case _`, `switch`/`default`); (b) o linter não pode depender de análise de tipos específica de Rust sem trair o IR neutro; (c) regras-métrica (R2, R5) não cabem em `warning` sem gerar fadiga.

## Decisão

> 1. **Nível `info` entra no núcleo**: `Level::Info` em `violation.rs`, mapeado para `note` no SARIF (que suporta o nível nativamente), nunca falha `--fail-on`, configurável em `[rules]` como os restantes níveis não-fatais. É um passo independente e anterior às regras de match.
>
> 2. **Família de regras de decisão mecânica no tronco**, escopo inicial Rust:
>    - V16 `WildcardSaturation` (warning → error por ratchet) — R1: wildcard que descarta informação em enum fechado de domínio;
>    - V17 `CompoundGuard` (warning) — R3: guard com `&&`/`||`;
>    - V18 `RangePatternInMatch` (warning) — R4: range numérico em match de domínio;
>    - V19 `OrPatternAlternatives` (info) — R2: reporta nº de alternativas por braço;
>    - V20 `DeepPatternNesting` (info) — R5: profundidade > 2 fora de contexto-tabela.
>
> 3. **IR neutro, null por omissão**: novo trait `HasDecisionArms` em `rule_traits.rs`. Só o `rs_parser` o preenche nesta fase; os restantes parsers devolvem colecção vazia e as regras produzem zero violações. Extensão futura a Python/TS/Go é preenchimento de parser, nunca reescrita de regra.
>
> 4. **Proxies sintácticos em vez de tipos** (a regra decide sobre a forma, não sobre o tipo):
>    - scrutinee aberto: chamada de método (`.kind()`, `.get(i)`), indexação ou literal → isento;
>    - reincorporação: braço liga identificador (`other =>`) e o identificador ocorre no corpo → isento;
>    - saturação arbitrária (DENY): corpo é path qualificado de enum (`Unit::Percent`, `Assoc::Left`) ou literal não-neutro (`1.0`, `vec![]`);
>    - default neutro (WARN): corpo é `false`/`true`/`0`/`0.0`/tupla de zeros/`Default::default()`;
>    - delegação (INFO): corpo é call expression;
>    - walker parcial (WARN arquitectural): corpo é bloco vazio ou `continue` em módulos de exportação/pipeline.
>
> 5. **Diagnósticos na nomenclatura da linguagem**: a mensagem nunca diz «wildcard» a um utilizador TypeScript. O IR guarda o snippet verbatim do padrão (`_ =>`, `case _`, `default:`) e a regra consulta uma tabela de termos por linguagem (`decision_arm_term_for(language)`); a mensagem cita o snippet real do código e o termo nativo (ex.: Rust — «wildcard `_ =>` descarta informação em enum de domínio»; Python — «`case _` descarta…»; TS/Go — «`default:` descarta…»).
>
> 6. **Excepções declaradas, não comentários mágicos**: tabela `[wildcard_exceptions]` em `crystalline.toml` com `ficheiro:linha = "justificativa"`, seguindo o precedente de `[orphan_exceptions]`. Hubs intencionais (ex.: normalização de cores via `to_rgba_f32()`) ficam auditáveis num único sítio.
>
> 7. **Ratchet de severidade**: V16–V18 nascem `warning`; promoção a `error` só quando o worklist do typst-crystalline (26 DENY + ~18 neutros + 131 walkers) estiver fechado. Invariante: nenhuma release do linter quebra o CI de um projecto aderente no dia da adopção.

## Prompts Afetados

| Prompt | Natureza da mudança |
| :---- | :---- |
| prompts/rules/wildcard-saturation.md | **Novo**: especificação de V16 (e partilhada por V17–V20) — filtros, proxies, matriz de severidade, mensagens por linguagem, teste de mutação |
| prompts/violation-types.md | **Alterado**: nível `info`, mapeamento SARIF `note`, semântica de `--fail-on` |
| prompts/rules/*.md (V1–V15) | Mantidos |
| readme_prompt.md | **Alterado**: tabela de verificações V16–V20, campo `languages` em `[rules]` |

## Consequências

**Positivas**: o modo de falha «variante nova adoptada silenciosamente» passa a ser detectável em CI; o nível `info` desbloqueia futuras regras-métrica sem poluir warnings; o IR de braços de decisão beneficia o tronco (todos os parsers podem vir a preenchê-lo); o typst-crystalline ganha ratchet automático sobre os 26 casos DENY já medidos.

**Negativas**: os proxies sintácticos podem divergir da classificação com tipos em casos não observados na amostra (mitigado pelo gate de precisão em «Questões em aberto»); o núcleo (`violation.rs`, `sarif_formatter.rs`, `config.rs`) é tocado — blast radius partilhado por todas as regras.

**Neutras**: projectos não-Rust vêem zero mudança de comportamento (colecção vazia); utilizadores Zig nunca verão V16 (switch exaustivo por construção na linguagem).

## Alternativas Consideradas

| Alternativa | Prós | Contras |
| :---- | :---- | :---- |
| Ramo/fork do linter para regras Rust | Isolamento total de risco | Divergência permanente do IR; duplo merge de `rs_parser.rs`; auto-validação deixa de cobrir as regras novas; a validação empírica já aconteceu — não é investigação. **Rejeitada.** |
| Implementar com `syn` em vez de tree-sitter | Padrões Rust exactos | Quebra o IR neutro (ADR-0001); torna a regra inextensível a outras linguagens; cria segundo parser de Rust para manter. **Rejeitada.** |
| Emitir R2/R5 como `warning` | Sem mudança ao núcleo | Métricas não são defeitos; fadiga de avisos; degrada o canal. **Rejeitada.** |
| Modo `--metrics` separado em vez de nível `info` | Núcleo intocado | Segundo pipeline de relatório; resultados não chegam ao SARIF/Code Scanning. **Rejeitada.** |
| **Tronco + nível `info` + proxies sintácticos (escolhida)** | Regras são cidadãos normais do tronco; SARIF `note` nativo; extensão por parser | Toca o núcleo partilhado; exige gate de precisão empírica |

## Questões em aberto (bloqueiam o status ACEITO)

1. **Precisão dos proxies**: a implementação tree-sitter de V16, corrida sobre o typst-crystalline, deve reproduzir a classificação de referência (varredura `syn` de 2026-08: 26 DENY / ~18 WARN-neutro / 131 WARN-walker / 1 INFO) com concordância ≥ 95% por categoria; divergências são analisadas caso a caso e o proxy ajustado ou a excepção registada.
2. **Auto-validação**: `crystalline-lint .` sobre o próprio repo do tekt-linter fica verde com V16–V20 activos (código adaptado ou excepções justificadas em `[wildcard_exceptions]`).
3. **Não-regressão multi-linguagem**: corrida sobre um projecto TypeScript e um Python de referência, V16–V20 produzem exactamente zero violações.

## Referências

[^1^]: Varredura AST (syn) do typst-crystalline, 2026-08 — contagens R1–R7 por camada, triagem dos 291 wildcards, amostra de 29 casos classificada manualmente (documentos da sessão de medição, repo typst-crystalline).
[^2^]: SARIF v2.1.0 — nível `note` para resultados informativos: https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html
[^3^]: Formalização de MC/DC para pattern matching (padrão refutável = decisão; sub-padrões = condições): Zaeske et al., «Towards MC/DC of Rust», arXiv:2409.08708: https://arxiv.org/abs/2409.08708
