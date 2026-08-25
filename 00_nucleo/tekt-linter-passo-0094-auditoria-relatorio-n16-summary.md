# Passo operacional 0094 — auditoria segregada do relatório N16 por módulo

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/audit-n16-summary`
> **Pré-condição:** P0093 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0093

## Objetivo

Auditar a seam L2 pura que reconhece tags `N16[α/β/γ]`, associa localizações a módulos,
agrega fontes e exceções e apresenta a tabela `n16-summary`, hoje materializada em
`02_shell/n16_summary.rs`.

O lote cobre extração textual da tag, identidade de localização, agrupamento,
deduplicação, totais, ordenação, percentuais, avisos de amostra pequena e o consumidor
real `--format n16-summary`. Não reabre a regra V16, parsers, descoberta de arquivos,
carregamento TOML, `path_encoding` já fechado no P0072 nem a taxonomia arquitetural do
ADR-0017.

## Hipótese e risco

O componente parece de baixo risco porque transforma coleções finitas sem I/O. O risco
residual é epistemológico: ordem de `HashMap`, normalização de paths ou regras de
apresentação implícitas podem alterar contagens e prioridades sem produzir violação.

As fontes históricas também deixam pontos potencialmente incompatíveis: o P0069 exige
ordem primária por γ absoluto, descreve percentual como secundário e formula aviso para
`total < min_sample_size`, mas sua tabela de referência não avisa módulos pequenos sem
γ. Nenhum gate pode escolher silenciosamente uma interpretação.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| taxonomia N16 | `00_nucleo/adr/0017-v16-v21-diferenca-categorica.md` | `79f406654aacf3693616232a4fdbb911e359486d089ffde841af5375625104dd` |
| especificação histórica do relatório | `00_nucleo/tekt-linter-passo-0069-relatorio-n16-por-modulo.md` | `1cbd96c5d4c7ca085406c7689733c2c1ef5380af59e3e19ec88594180424f808` |
| contrato V16/exceções | `00_nucleo/prompts/rules/wildcard-saturation.md` | `19f79428f1e7c9740ae7f2466f03bc82c22a5632a2388e5b2c587a3fa2588609` |
| arquitetura do pipeline | `00_nucleo/prompts/linter-core.md` | `908a00fd7e4eaa985b755682fb73984cbb886496ce988070f176ad307ec24446` |
| apresentação pública | `00_nucleo/prompts/sarif-formatter.md` | `959d6e56785e6c32087fcae361300304d4a8197a2669f9df7f2b4809a4842605` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |

Qualquer alteração invalida o Assessment 0023 até resselamento.

## Alegações candidatas

### Extração e identidade

1. Somente formas autorizadas de `N16[α/β/γ]` ou A/B/C, com case ASCII permitido pelo
   L0, produzem tag; decoys, tag truncada, múltiplas tags e Unicode hostil exigem decisão
   normativa explícita.
2. Source path e chave TOML que designam a mesma localização têm uma identidade única e
   determinística, sem equivalência inventada por basename, substring ou remoção de
   componentes semanticamente relevantes.
3. Chaves TOML malformadas, múltiplos `:`, path vazio, linha zero, overflow e path não
   UTF-8 possuem política nominal ou são `SPEC-GAP`; não podem virar linha zero por
   fallback implícito sem autorização.

### Agregação

4. Cada ocorrência autorizada é contabilizada exatamente uma vez; duplicação entre
   fonte e exceção na mesma localização não aumenta o total.
5. Ordem de fontes e ordem de inserção do `HashMap` de exceções não alteram o mapa final
   nem os bytes apresentados.
6. Agrupamento por módulo preserva as categorias especiais decididas (`math/layout/`,
   `export/` e camadas) e não usa substring capaz de confundir paths próximos.
7. Módulos sem anotação não aparecem; totais são soma exata de α, β e γ, sem overflow
   silencioso dentro do domínio testável.

### Apresentação e consumidor

8. A tabela possui colunas e linha total normativas, com arredondamento e representação
   de zero/ausência decididos antes do gate.
9. γ absoluto é a chave primária de ordenação. Empates devem ter desempate total e
   determinístico publicado; percentual ou total não podem ganhar precedência por
   acidente.
10. A condição exata de aviso de amostra pequena — inclusive módulos com γ zero,
    `min_sample_size` zero/um e total zero — deve ser fechada pelo L0 antes do gate.
11. O cálculo aproximado de quantos pontos percentuais um caso altera deve ter regra de
    arredondamento explícita e não produzir NaN/infinito.
12. O wiring seleciona `n16-summary` somente quando solicitado, usa fontes/exceções já
    obtidas por L3/L4 e não converte o relatório em regra de gate ou severidade V nova.

### Arquitetura Tekt

13. L2 contém taxonomia de apresentação, agregação e formatação puras; não lê filesystem,
    configuração, ambiente, relógio, rede ou processo.
14. L1 fornece somente entidades/contratos de entrada e não conhece o formato de saída.
15. L3 descobre/lê fontes e configuração; L4 instancia, injeta e escolhe o formato, sem
    duplicar classificação, deduplicação, ordenação ou percentuais.

## Preflight arquitetural e normativo obrigatório

O Assessment 0023 e o adversário A devem decidir antes dos gates:

- gramática exata de tag, incluindo primeira/múltiplas ocorrências e decoys;
- parser total da chave `path:line`, especialmente `:` dentro do path e linha inválida;
- identidade de localização e grau autorizado de normalização entre fonte e TOML;
- agrupamento por componentes de path, sem heurística por substring;
- precedência fonte/exceção quando a mesma localização traz tags divergentes;
- desempate total após γ absoluto;
- formato de `% γ`, caso α-only e arredondamento;
- condição exata de amostra pequena e cálculo de `~pp`;
- comportamento de mapa vazio e limites de `min_sample_size`;
- seleção/exit status do formato no consumidor real.

Ausência de decisão é `SPEC-GAP`. Contradição entre P0069, ADR e prompt V16 também é
`SPEC-GAP`; nenhuma implementação vigente serve como oráculo para sanear o L0.

## Protocolo segregado

### A — Assessment e adversário L0

1. Após integrar P0093, criar branch nova e
   `00_nucleo/assessments/0023-n16-summary.md` com baseline/hash de integração.
2. O adversário A lê somente Assessment e insumos L0 hash-pinned, classifica cada ponto
   como decidido, contraditório ou ausente.
3. Sanear a autoridade normativa mínima e resselar todos os hashes antes de qualquer
   gate. Se a política exigir informação que a API não recebe, registrar `SPEC-GAP` em
   vez de inventar leitura em L2.

### B1 — Gate cego de extração e agregação

Um verificador novo, sem produção, cria exclusivamente
`tests/n16_summary_collection_assessment.rs`. Deve cobrir gramática, paths hostis,
chaves TOML, duplicatas, conflito fonte/exceção, ordem de input/HashMap, agrupamentos e
contagens. Fixtures são valores em memória; nenhum filesystem ou parser real.

### B2 — Gate cego de apresentação e wiring observável

Outro verificador cria exclusivamente `tests/n16_summary_presentation_assessment.rs`.
Cobre ordenação total, tabela vazia, totais, percentuais, arredondamento, avisos e
payloads hostis. O comportamento do consumidor CLI deve ser provado por API pública ou
fixture confinada separada; o gate não lê B1 nem produção.

### C — Confronto e correção

Somente depois de B1/B2 congelados, confrontar `02_shell/n16_summary.rs` e a menor seam
necessária de `04_wiring/main.rs`. Correção funcional exige RED causal.

A solução deve respeitar:

- L2 decide e transforma dados já fornecidos, sem efeitos externos;
- L3 mantém leitura/parse de arquivos e configuração;
- L4 apenas seleciona o formato e injeta dados;
- L1 não recebe lógica de relatório;
- `path_encoding` não é reaberto salvo RED novo e específico.

V16, parsers, config loader e CLI geral ficam fora do delta salvo RED causal impossível
de fechar na seam autorizada. Nenhuma limpeza oportunista é permitida.

### D — Adversário final

Verificar hashes, causalidade RED→GREEN, independência dos gates, determinismo por
permutação, consumidor real, arquitetura Tekt, delta escondido e regressão dos
assessments 0001–0022.

## Classificações e fechamento

- `RED`: produção contradiz alegação congelada;
- `SPEC-GAP`: autoridade não decide ou se contradiz;
- `GATE-DEFECT`: gate inventa política, usa produção como oráculo ou contamina camadas;
- `PASS`: alegação confrontada sem divergência.

Fechar somente como `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. B1/B2 em identidades e arquivos separados;
2. hashes dos gates registrados antes do confronto;
3. testes dirigidos de coleção e apresentação;
4. propriedades de permutação de fontes e ordem de inserção de exceções;
5. fixture confinada do formato real, se o L0 exigir observação end-to-end;
6. `cargo test --workspace --quiet`;
7. `cargo run --quiet -- --checks v16 --format n16-summary .`;
8. auto-lint V5/V6/V7/V12;
9. `rustfmt --check` dirigido e `git diff --check`;
10. busca por I/O/import L3 em L2;
11. reparador de hashes em dry-run;
12. adversário final e worktree limpo.

## Saídas esperadas

- Assessment 0023;
- gates B1/B2 segregados;
- L0 saneado se necessário;
- correção mínima somente após RED;
- relatório `00_nucleo/relatorio-p0094-auditoria-n16-summary.md`;
- matriz L0→L1/L2→L3/L4→gates;
- veredito final.

P0094 não autoriza merge, push, instalação ou release. Sem integração prévia do P0093,
a execução deve parar antes de criar branch concorrente.
