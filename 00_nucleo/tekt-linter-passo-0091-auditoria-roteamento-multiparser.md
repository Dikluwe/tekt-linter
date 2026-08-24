# Passo operacional 0091 — auditoria segregada do roteamento MultiParser

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/audit-multiparser-routing`
> **Pré-condição:** P0090 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0090

## Objetivo

Auditar a seam de composição que seleciona um `LanguageParser` a partir de
`SourceFile.language`, hoje materializada por `MultiParser` em `04_wiring/main.rs`.

O lote verifica somente roteamento e fallback. Não audita gramáticas, AST, FQN, aliases,
imports, cobertura, prompts ou snapshots dos parsers concretos.

## Escopo de baixo risco

Matriz fechada esperada:

| Language | Destino |
|---|---|
| Rust | RustParser |
| TypeScript | TsParser |
| Python | PyParser |
| C | CParser |
| Cpp | CppParser |
| Zig | ZigParser |
| Go | GoParser |
| Java | JavaParser |
| Elixir | ElixirParser |
| Unknown | `ParseError::UnsupportedLanguage` |

O risco principal não é sintático, mas causal: a escolha de slot é política pura; a
instanciação e chamada do adapter são composição L4. O passo deve separar essas duas
alegações sem mover parser concreto para L1 nem tornar `MultiParser` público apenas para
facilitar teste.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| sistema/composição | `00_nucleo/prompts/linter-core.md` | `2e5da1cfb9d1f66e5015cf67bf2f6fb9e8992ad29c59d05d5acef4ea2705f8d3` |
| contrato parser | `00_nucleo/prompts/contracts/language-parser.md` | `203ed423a5149331525a6c7bc1662e74b87e6bcab7a5ee5337cda7581523791b` |
| ParseError | `00_nucleo/prompts/contracts/parse-error.md` | `1f8c47cb5d0001c356c71e2df8ec0619d76dd5a439a5ba9e9b8f8d7285282645` |
| SourceFile/Language | `00_nucleo/prompts/contracts/file-provider.md` | `1574ce788513573901376fc80933464cca5e7b6bc17acf5af8bfcd28e4d7335d` |
| isolamento multilíngue | `00_nucleo/adr/0009-isolamento- de-parsers-por-linguagem.md` | `fbfeb007115f2464ece7e1f0e2a5615bb06b459e7bb7446bbd2957a06ee67452` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |

Qualquer alteração invalida o Assessment 0020 até resselamento.

## Alegações candidatas

1. Cada uma das nove linguagens suportadas seleciona exatamente um slot distinto e
   nominalmente correto.
2. `Language::Unknown` não chama parser algum e retorna
   `ParseError::UnsupportedLanguage` preservando path e linguagem.
3. Roteamento depende apenas de `file.language`; path, conteúdo, layer e
   `has_adjacent_test` são irrelevantes à escolha.
4. O `SourceFile` original é emprestado ao parser selecionado, sem clone, reconstrução,
   mutação ou troca de lifetime.
5. Resultado `Ok` ou `Err` do parser selecionado é propagado sem tradução, fallback ou
   segunda tentativa.
6. Exatamente uma chamada ocorre; parsers não selecionados permanecem intocados.
7. Ordem dos campos/adapters, configuração e disponibilidade de outros parsers não
   alteram a decisão.
8. A seam não acessa filesystem, ambiente, relógio, rede ou processo.

## Preflight arquitetural obrigatório

Assessment 0020 e adversário cego devem decidir antes do gate:

- tipo público fechado que representa os nove slots e o fallback;
- localização causal da decisão `Language → slot` em L1;
- API pura black-box para a matriz, sem tipos L3;
- seam de composição testável com spies sem tornar adapters concretos dependências L1;
- representação exata de `Unknown` e precedência diante de parser ausente;
- se `MultiParser` pode permanecer struct privada L4 após extração da política.

Ausência de decisão é `SPEC-GAP`. É proibido usar os nove parsers reais como oráculo do
gate ou expor `MultiParser` por conveniência.

## Protocolo segregado

### A — Assessment e adversário L0

1. Criar `00_nucleo/assessments/0020-multiparser-routing.md` com baseline pós-merge e
   hashes autorizados.
2. Adversário A lê somente Assessment/L0; produção, testes, histórico e relatórios são
   proibidos.
3. Congelar matriz executável e `SPEC-GAPs`.
4. Sanear L0 e resselar antes de qualquer gate ou código.

### B1 — Gate cego da política de slot

Verificador novo cobre as dez variantes, mutação sistemática dos campos irrelevantes,
determinismo, igualdade e ausência de tipos/I/O L3. O oráculo vem exclusivamente da
matriz L0 hash-pinned.

### B2 — Gate segregado da composição

Outro verificador usa parsers-spy independentes para provar chamada única, propagação
exata de Ok/Err, empréstimo do mesmo `SourceFile` e zero chamadas nos demais slots. O
gate não executa tree-sitter, readers, snapshots ou filesystem.

Se o L0 não oferecer seam genérica segura para B2, classificar `SPEC-GAP`; não substituir
por teste interno contaminado no binário.

### C — Confronto e correção

Somente após B1/B2 congelados, confrontar `MultiParser`. Correção funcional exige RED.
A solução deve respeitar:

- L1 decide slot por enum de domínio puro;
- L3 implementa `LanguageParser` concreto;
- L4 instancia adapters e encaminha a chamada conforme o slot;
- L2 permanece fora do roteamento.

Nenhum parser concreto, config, walker, IR extraída ou regra V1–V25 pode mudar salvo RED
pertencente ao lote.

### D — Adversário final

Verificar causalidade, gravidade, matriz completa, fallback fail-closed, spies
independentes, ausência de delta escondido e regressão dos assessments 0001–0019.

## Classificações e fechamento

- `RED`: produção contradiz alegação congelada;
- `SPEC-GAP`: L0 não decide API/camada/seam;
- `GATE-DEFECT`: teste compartilha implementação ou parser real como oráculo;
- `PASS`: alegação confrontada sem divergência.

Fechar somente como `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. gates B1 e B2;
2. testes do contrato `language_parser` e `parse_error`;
3. regressão V0/PARSE e partição do walker;
4. smoke de cada parser existente apenas como regressão, não oráculo;
5. `cargo test --workspace --quiet`;
6. auto-lint V4/V5/V11/V12;
7. `cargo run --quiet -- . --fix-hashes --dry-run`;
8. `rustfmt --check` dirigido e `git diff --check`;
9. busca mecânica por I/O e imports L3 na política L1;
10. adversário final e worktree limpo.

## Saídas esperadas

- Assessment 0020;
- gates cegos B1/B2;
- L0 saneado se necessário;
- correção mínima após RED;
- relatório `00_nucleo/relatorio-p0091-auditoria-multiparser.md`;
- matriz L0→política L1→composição L4→gates;
- veredito final.

P0091 não autoriza merge, push, instalação ou release. Sem integração prévia do P0090,
a execução deve parar antes de criar branch concorrente.
