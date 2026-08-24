# Passo operacional 0088 — saneamento arquitetural e auditoria segregada V21

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** fechado — `READY WITH RESIDUAL AUDIT`
> **Branch:** `codex/plan-v21-segregated-audit`
> **Base:** `master@f27433d`
> **Predecessor:** P0087

## Objetivo

Auditar V21 `HardcodedContextualValue` sem ratificar uma violação da arquitetura Tekt.
O classificador deve permanecer em L1 puro; leitura de arquivo para verificar frescura de
`// ref:` pertence a L3 e só pode alcançar L1 por contrato causal explícito.

O passo separa duas alegações que hoje aparecem fundidas:

1. classificação pura do predicado contextual, filtros, citações e severidade;
2. resolução externa da frescura de uma referência de path/linha.

V22 permanece fechado pelo assessment 0004 e entra apenas como regressão.

## Preflight arquitetural obrigatório

Antes de qualquer novo teste ou alteração L1:

1. congelar Assessment 0017 com baseline, alegações e insumos L0 por SHA-256;
2. registrar como `SPEC-GAP` qualquer contrato que exija filesystem em L1 sem seam;
3. verificar os headers causais de `unsourced_constant.rs`, `rule_traits.rs` e de toda
   porta/adapter eventualmente necessária;
4. atualizar primeiro o L0 causal; somente depois materializar L1/L3;
5. proibir imports de filesystem, config, shell, wiring ou lab no classificador L1.

O estado atual não autoriza mover `std::fs` mecanicamente. A seam precisa nascer em L0,
com semântica de `valid`, `stale` e `unknown` suficiente para impedir falso silêncio.

## Insumos normativos iniciais

| Unidade | Caminho | SHA-256 atual |
|---|---|---|
| regra V21 | `00_nucleo/prompts/unsourced-constant.md` | `e1bc92fa1c56585ed0b15cc22762a7538ea13676010c41875ab5f1db3f662fd3` |
| IR/trait | `00_nucleo/prompts/contracts/rule-traits.md` | `aeced5c851ac21a6214c1c4ca2cdd12e011926af9ae64898b95fcda0690ac4df` |
| referência arquitetural | `00_nucleo/adr/0017-v16-v21-diferenca-categorica.md` | hash a registrar no assessment |

Se a seam exigir novo contrato de porta, criar primeiro seu prompt L0 e incluir caminho
e SHA-256 no Assessment 0017 antes de gerar o arquivo Rust correspondente.

## Fronteiras do lote

### Incluído

- `V21RuleConfig` e defaults normativos;
- `SourceConstant`, `Citation` e `HasConstants` apenas nos campos consumidos por V21;
- classificador puro de escopo, predicado relacional, triviais, tabelas/testes, citações,
  strict modules, mensagens, ordem e multiplicidade;
- contrato puro de frescura e adapter L3 mínimo, se legitimados pelo novo L0;
- gates segregados distintos para L1 e para o adapter;
- regressão V22 e validação arquitetural Tekt.

### Excluído

- ampliar a heurística ou listas por conveniência sem RED congelado;
- auditoria integral do parser Rust;
- inventário V22, relatórios agregados e N16;
- CLI/wiring além do encaixe mínimo exigido pela seam;
- alterações no Typst Cristalino, instalação, release, push ou merge.

Achado que exija parser completo, configuração transversal ou consumidor externo vira
residual explícito ou passo próprio; não amplia silenciosamente o P0088.

## Protocolo segregado

### A — Contrato e adversário normativo

O orquestrador cria Assessment 0017 antes de ler conclusões do gate. Um adversário recebe
somente assessment e L0 autorizados e procura:

- lista/configuração tratada como substring em vez de identidade declarada;
- triviais equivalentes ou quase iguais;
- citação `Spec`/`Rationale` vazia, opaca ou malformada;
- `Ref` válida, stale e unknown;
- ausência de `project_root`, path absoluto, escape, symlink e linha extrema;
- strict module e format module com paths parecidos;
- silêncio indevido por erro externo;
- ordem, duplicatas e campos irrelevantes;
- dependência de filesystem em L1 ou gravidade invertida.

Todos os `SPEC-GAP` são congelados antes de editar L0.

### B1 — Gate cego do classificador L1

Um verificador novo, proibido de ler produção/testes existentes, recebe API pública
completa e hash-pinned. O gate deve cobrir:

1. todas as linguagens e coleção vazia;
2. produto `is_in_binary_scaling × context_var × geometric_sink`;
3. escopo, format modules, test-origin e data-table;
4. triviais normativos e controles próximos;
5. `None`, `Spec`, `Rationale` e `Ref` sob estados de frescura injetados;
6. Warning versus Error em strict modules;
7. literal/context/sink/detail/location preservados;
8. múltiplas constantes, ordem, multiplicidade e mutação de campos irrelevantes;
9. ausência de I/O: gate executa com mock puro e sem filesystem real;
10. somente `rule_id == V21`.

### B2 — Gate cego do adapter L3

Outro verificador recebe somente o contrato da porta e testa em sandbox temporário:

- arquivo/linha existente e não vazia → `valid`;
- arquivo ausente, linha zero, além de EOF ou vazia → `stale`;
- erro de leitura, encoding/metadata não suportado e entrada fora da raiz → `unknown` ou
  erro explícito conforme L0, nunca `valid` nem silêncio;
- confinamento de path e symlink;
- Unicode, arquivos grandes, orçamento e determinismo;
- zero escrita, rede, hooks ou mutação do projeto.

O gate L3 não importa o classificador V21 como oráculo.

### C — Confronto e correção

Somente após B1/B2 congelados, o orquestrador confronta L0, gates e produção. Cada falha
é `RED`, `SPEC-GAP` ou `GATE-DEFECT`. Correção funcional exige RED prévio; mudança causal
começa no L0 e termina com headers/hashes resselados pelo reparador oficial.

### D — Adversário final

Um papel independente verifica:

- causalidade L0→L1/L3 no histórico;
- L1 sem I/O e dependências invertidas;
- adapter L3 implementando porta, sem lógica de classificação V21;
- gate que não compartilha expectativas com produção;
- RED/SPEC-GAP histórico encerrado;
- ausência de alteração escondida em parser/config/CLI;
- regressão V22 e demais assessments fechados.

Achados são fechados antes de qualquer recomendação de merge.

## Classificações e política de unknown

- `RED`: comportamento contradiz alegação executável congelada;
- `SPEC-GAP`: L0 não decide política necessária ou não publica API suficiente;
- `GATE-DEFECT`: teste não cobre a alegação ou deriva oráculo da produção;
- `UNKNOWN` externo nunca pode ser convertido em citação válida ou silêncio.

A política exata de V21 diante de `UNKNOWN` deve ser decidida no L0 antes da
materialização; este passo recomenda diagnóstico explícito, não falso negativo.

## Validação de fechamento

Executar, no mínimo:

1. gates B1 e B2;
2. regressão V22;
3. `cargo test --workspace --quiet`;
4. `cargo run --quiet -- . --fix-hashes --dry-run`;
5. auto-lint V21 no próprio repositório;
6. `rustfmt --check` somente nos arquivos funcionais novos/alterados;
7. `git diff --check` contra a base;
8. busca mecânica confirmando ausência de `std::fs`/I/O no L1 tocado;
9. smoke V21 no consumidor apenas em modo leitura, com fingerprint antes/depois;
10. worktree limpo após o commit de fechamento.

O drift rustfmt global legado e candidatos históricos de hash do consumidor continuam
residuais conhecidos; não autorizam refactor ou escrita transversal neste lote.

## Saídas esperadas

- `00_nucleo/assessments/0017-hardcoded-contextual-value-v21.md`;
- L0 V21 saneado e, se necessário, novo L0 de porta de frescura;
- gate black-box L1;
- gate do adapter L3;
- relatório adversarial em `lab/`;
- `00_nucleo/relatorio-p0088-triagem-v21.md`;
- matriz de rastreabilidade L0 → L1/porta → L3 → gates;
- veredito final `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

Nenhum merge, instalação, release ou push pertence ao P0088. Após fechamento, eventual
merge deve ocorrer em ação separada.

## Fechamento

Executado em 2026-08-24. Assessment 0017, gates B1/B2, saneamento L0, separação
L1/L3/L4, RED adversarial de confinamento atômico e respectivos gate-defects foram
fechados. Evidência e residuais estão em `relatorio-p0088-triagem-v21.md`.
