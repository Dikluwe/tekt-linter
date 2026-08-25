# Passo operacional 0093 — auditoria segregada do planejamento fix-hashes

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/audit-fix-hashes-planning`
> **Pré-condição:** P0092 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0092

## Objetivo

Auditar a seam L2 que transforma violações V5 em plano de dupla paridade e executa o
plano pelo port `HashRewriter`, hoje materializada em `02_shell/fix_hashes.rs`.

O lote cobre somente planejamento, orquestração das duas escritas, dry-run e
apresentação. Não reabre confinamento, hashing byte-exato, permissões ou atomicidade
interna dos writers L3, já auditados no P0074; não audita V5, CLI ou o reparador como
ferramenta de desenvolvimento fora desse caso de uso.

## Hipótese e risco

O planejamento parece mecânico, mas a execução coordena dois efeitos dependentes:

1. gravar Hash A no header do código;
2. gravar Hash B na metadata do prompt.

O risco principal é relatar sucesso parcial como sucesso total, descartar estados não
corrigíveis, alterar cardinalidade ou apresentar dry-run como escrita realizada.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| fluxo fix/update | `00_nucleo/prompts/fix-hashes.md` | `dd987d35beced5bd7fb6a0961f1e2cfa08d85d4c6f8a3702f797f7a6f32e8024` |
| apresentação/CLI | `00_nucleo/prompts/sarif-formatter.md` | `959d6e56785e6c32087fcae361300304d4a8197a2669f9df7f2b4809a4842605` |
| arquitetura do pipeline | `00_nucleo/prompts/linter-core.md` | `908a00fd7e4eaa985b755682fb73984cbb886496ce988070f176ad307ec24446` |
| tipos V5 | `00_nucleo/prompts/violation-types.md` | `147afa0d8f3f3e6e30e050590dad0b99c7da8486d3565e3f6c42f7fa883ea4dc` |
| fechamento writer L3 | `00_nucleo/assessments/0006-prompt-io-and-hashes.md` | `df3b8bcf1f14f1989c978efe620a55a822512a8ccdbf6e5ea35d3d918d636567` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |

Qualquer alteração invalida o Assessment 0022 até resselamento.

## Alegações candidatas

### Planejamento

1. Somente `rule_id == "V5"` produz entrada; ordem, duplicatas e uma entrada por
   ocorrência são preservadas.
2. `read_header` é chamado exatamente uma vez por V5, com o path integral da violação.
3. Header ilegível permanece estado distinto e não chama nenhum cálculo de hash.
4. Header legível preserva prompt path e old hash integralmente; calcula Hash A uma vez
   pelo prompt path e Hash B uma vez pelo source path.
5. Falha de Hash A e falha de Hash B são estados distintos, sem strings/`Option`s
   paralelos capazes de representar combinações contraditórias.
6. Nenhum path é normalizado, canonicalizado, deduplicado ou associado por basename.

### Execução

7. Cada entrada planejada produz exatamente um resultado na mesma ordem, inclusive
   estados não corrigíveis.
8. Dry-run não chama escrita e apresenta source path, prompt path, old hash, Hash A e
   Hash B suficientes para tornar a dupla paridade proposta observável.
9. Execução real de entrada pronta chama `write_hash(source, Hash A)` antes de
   `write_prompt_meta(prompt, Hash B)`, exatamente uma vez cada.
10. Se a primeira escrita falha, a segunda não ocorre e a fase/razão exatas são
    preservadas.
11. Se a primeira passa e a segunda falha, o resultado nunca é sucesso. O L0 deve decidir
    se exige rollback, estado explícito de sucesso parcial ou transação única no port;
    ausência dessa decisão é `SPEC-GAP`.
12. Uma falha não interrompe nem altera entradas posteriores.

### Apresentação e arquitetura

13. `DryRun`, sucesso integral, falha na escrita do código, falha na metadata e estados
    não corrigíveis têm apresentação distinguível e não enganosa.
14. Payloads de hash/path e razões hostis permanecem observáveis sem reconstrução.
15. L2 não acessa filesystem, ambiente, relógio, rede ou processo; somente o port chama
    L3. L4 apenas instancia/injeta e coordena a reanálise.

## Preflight arquitetural obrigatório

Assessment 0022 e adversário A devem decidir antes dos gates:

- API única e pública de `HashRewriter`;
- tipos fechados e comparáveis de plano/resultado;
- estados exatos para header ilegível, prompt ausente e source hash indisponível;
- cardinalidade, ordem e política de duplicatas;
- semântica transacional das duas escritas e precedência de falhas;
- resultado observável de dry-run;
- responsabilidade pela compensação se a segunda escrita falhar;
- apresentação do sucesso parcial sem alegar dupla paridade concluída.

Ausência de decisão é `SPEC-GAP`. O gate não pode ler writer L3, filesystem ou produção.
P0074 serve apenas como evidência de que cada chamada L3 isolada é atômica; não prova
atomicidade entre as duas chamadas.

## Protocolo segregado

### A — Assessment e adversário L0

1. Criar `00_nucleo/assessments/0022-fix-hashes-planning.md` com baseline pós-merge e
   hashes autorizados.
2. Adversário A lê somente Assessment/L0 e classifica gaps de estado/transação.
3. Sanear L0 e resselar antes de qualquer gate ou confronto.

### B1 — Gate cego de planejamento

Verificador novo cobre filtro, cardinalidade, ordem, duplicatas, paths hostis, todas as
falhas de leitura/cálculo e contagem/argumentos de cada método do spy.

### B2 — Gate cego de execução

Outro verificador cobre dry-run, sequência das duas escritas, falhas em cada fase,
continuação, duplicatas e resultado por entrada. Não usa writer real.

### B3 — Gate cego de apresentação consumida

Terceiro verificador cobre `format_plan` e `format_results`, incluindo payloads hostis,
estados não corrigíveis, dry-run e sucesso parcial. Esse gate é obrigatório antes do
confronto para evitar repetir o defeito descoberto no P0092.

### C — Confronto e correção

Somente após B1–B3 congelados, confrontar `02_shell/fix_hashes.rs` e o consumidor L4
estritamente necessário. Correção funcional exige RED.

A solução deve respeitar:

- L2 possui caso de uso, estados e port;
- L3 executa primitivas externas, sem decidir sucesso do caso de uso;
- L4 injeta adapter e reanalisa, sem reimplementar política;
- L1 permanece fora da mutação, exceto tipos de diagnóstico já existentes.

Nenhum writer L3, regra V5, parser ou CLI pode mudar salvo RED causal ou reparo mecânico
de hash.

### D — Adversário final

Verificar causalidade, transação/compensação, gates independentes, arquitetura Tekt,
delta escondido, apresentação real e regressão dos assessments 0001–0021.

## Classificações e fechamento

- `RED`: produção contradiz alegação congelada;
- `SPEC-GAP`: L0 não decide estado, transação ou compensação;
- `GATE-DEFECT`: gate compartilha implementação/L3 ou ignora o consumidor real;
- `PASS`: alegação confrontada sem divergência.

Fechar somente como `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. B1, B2 e B3 em identidades/arquivos separados;
2. testes dirigidos de `fix_hashes` e regressão P0074;
3. dry-run CLI em fixture confinada, sem mutação;
4. falhas determinísticas de ambas as fases via spies;
5. `cargo test --workspace --quiet`;
6. auto-lint V5/V6/V7/V12;
7. `cargo run --quiet -- . --fix-hashes --dry-run`;
8. `rustfmt --check` dirigido e `git diff --check`;
9. busca por I/O/import L3 em L2;
10. adversário final e worktree limpo.

## Saídas esperadas

- Assessment 0022;
- gates B1/B2/B3 segregados;
- L0 saneado se necessário;
- correção mínima após RED;
- relatório `00_nucleo/relatorio-p0093-auditoria-fix-hashes.md`;
- matriz L0→L2→port→L3/L4→gates;
- veredito final.

P0093 não autoriza merge, push, instalação ou release. Sem integração prévia do P0092,
a execução deve parar antes de criar branch concorrente.
