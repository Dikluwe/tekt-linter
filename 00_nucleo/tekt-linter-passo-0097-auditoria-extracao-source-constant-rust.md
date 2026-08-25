# Passo operacional 0097 — auditoria da extração estrutural Rust de `SourceConstant`

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; `READY WITH RESIDUAL AUDIT`
> **Branch prevista:** `codex/audit-rust-source-constant-extraction`
> **Pré-condição:** P0096 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0096

## Objetivo

Auditar a seam L3 que transforma fonte Rust em ocorrências estruturais
`SourceConstant` dentro de `ParsedFile.constants`, materializada por `RustParser` e
`extract_constants` em `03_infra/rs_parser.rs`.

O lote cobre somente fatos estruturais observáveis: kind, snippet, linha/coluna,
`is_test_origin`, `function_return_type`, `is_in_binary_scaling`, `context_var`,
`geometric_sink`, `is_in_data_table`, ordem e multiplicidade. V21 e V22 são consumidores
obrigatórios de regressão, mas nenhum deles pode servir como oráculo dos gates.

Após o parecer A, o recorte executável foi reduzido: somente kind, snippet, linha/coluna,
ordem e multiplicidade de literais numéricos dentro de `function_item` permanecem no
oráculo. Os demais campos listados acima estão formalmente fora, ainda que B2 histórico
tenha sido planejado antes do saneamento.

## Exclusões normativas

Ficam fora, mesmo que hoje compartilhem a mesma função de produção:

- descoberta, parsing, janela, precedência ou semântica de `citation`;
- frescura de referências e qualquer acesso ao filesystem;
- decisão, severidade, mensagem ou localização de violações V21/V22;
- denominador, agrupamento, percentual ou apresentação do inventário V22;
- configuração global, wiring, seleção de checks, SARIF e exit status;
- parsers de outras linguagens e integração V16/V23–V25.

Os gates podem ignorar `citation`, mas não podem alegar que ela está correta. Necessidade
de decidir ou alterar qualquer item excluído é `SPEC-GAP` e interrompe o lote.

## Hipótese e risco

O recorte tem risco médio: a extração em memória é pura e possui fronteira L3→L1, mas
recebe bytes/AST Rust hostis e alimenta dois consumidores diretos. Gates históricos de
V21/V22 usam IR sintético; fixtures e CLI exercitam o parser apenas transitivamente.

Pontuação P0096: `camadas/efeitos/entrada/consumidores/L0/gates/regressão =
1/0/3/2/0/2/2 = 10`.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| contrato V21 | `00_nucleo/prompts/unsourced-constant.md` | `9560ecbcdc3a5f5eec14e0cabe96062b274504f92e1f009188c6dbc2f59fa174` |
| traits/IR de regras | `00_nucleo/prompts/contracts/rule-traits.md` | `cdba18365badfb56288480f683451914d88b0df07201acc43ee8334d22289ba3` |
| parser Rust | `00_nucleo/prompts/parsers/rust.md` | `f9b620ae1a377a9deca44a1a9ba80437097dbd254eb8664cf597d2a85e8ae0d3` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| Assessment V21 | `00_nucleo/assessments/0017-hardcoded-contextual-value-v21.md` | `fb3024c255789d409b73d2d8e5e138c753c8a01e9c986190ff727147081e584b` |
| fechamento V21 | `00_nucleo/relatorio-p0088-triagem-v21.md` | `2b5d0c09078dddb9d7dcab43b14e6585c40ea89253cdf020d438e88435524f40` |
| Assessment de risco | `00_nucleo/assessments/0025-inventario-risco-residual.md` | `4d9a7fa75def17dfcd5f5e552210b825d8b64ea98e64f8e9fdd430eb0fc74e2a` |
| reconciliação P0096 | `00_nucleo/assessments/0025-c-reconciliacao-risco.md` | `f713ec185c8c4e878da8c5cc609846271a6a1af3bd3a58e69b45e6667b1c7ede` |
| fechamento P0096 | `00_nucleo/relatorio-p0096-inventario-risco-residual.md` | `b653185723e46790ac32098cc8781787c8220247114d39301453be9c42750037` |

Na execução, os hashes devem ser recalculados depois da integração P0096. Qualquer
divergência interrompe o Assessment 0026 até resselamento.

## Alegações candidatas

### Identidade e localização

1. Cada literal elegível produz uma ocorrência, preservando ordem de fonte e
   multiplicidade; repetição textual não deduplica fatos.
2. `snippet` corresponde exatamente ao token/expressão autorizado pelo L0, sem trim ou
   normalização incidental.
3. linha é 1-based e coluna segue a convenção publicada para o parser Rust; Unicode antes
   do literal exige decisão explícita entre byte e caractere.
4. literals negativos, floats com sufixo, strings/chars/bytes, constantes nomeadas,
   macros, ranges e expressões compostas só entram conforme taxonomia fechada de
   `ConstantKind`; ausência de regra é `SPEC-GAP`.

### Contexto estrutural

5. `is_test_origin` deriva apenas de contextos Rust normativos (`#[test]`, módulos de
   teste e equivalentes decididos), sem heurística por nome de path.
6. `function_return_type` pertence à função envolvente correta e preserva o texto
   autorizado; closures, impl methods, async/unsafe/extern e retorno implícito exigem
   política L0 antes do gate.
7. `is_in_binary_scaling` identifica somente o papel estrutural definido para `*` e `/`,
   incluindo nesting e ordem dos operandos sem promover aritmética genérica.
8. `context_var` e `geometric_sink` preservam identidade de paths/campos e não usam
   substring aproximada; múltiplos candidatos exigem precedência explícita.
9. `is_in_data_table` depende de estrutura de match e limiar normativo, não de contagem
   textual de literais ou comentários.
10. Ordem, nesting e multiplicidade permanecem determinísticos sob whitespace,
    comentários e formatação equivalentes.

### Arquitetura e consumidores

11. L3 observa AST e materializa fatos; L1 define `SourceConstant`/`HasConstants` e decide
    diagnósticos; L4 apenas coordena o parser e as regras.
12. O extractor não chama V21/V22, não consulta config global e não lê filesystem.
13. V21 `unsourced_constant` e V22 `provenance_inventory` são os dois consumidores diretos;
    regressões de ambos permanecem verdes, mas seus resultados não validam a extração.
14. Campos excluídos, especialmente `citation`, não recebem alegação por passarem
    incidentalmente em fixtures.

## Preflight normativo obrigatório

O Assessment 0026 e o adversário A devem decidir antes dos gates:

- taxonomia exata de literals/constantes elegíveis e significado de `snippet`;
- convenção de coluna e Unicode;
- escopo de origem de teste;
- associação de função/return type;
- significado estrutural de scaling e tratamento de operandos/nesting;
- gramática de `context_var` e `geometric_sink` e desempate;
- definição e limiar de data-table;
- ordem/multiplicidade e comportamento diante de erro sintático;
- API pública mínima pela qual gates cegos obtêm `ParsedFile.constants`.

O comportamento atual de tree-sitter ou de `extract_constants` não é autoridade. Gaps não
podem ser preenchidos copiando fixtures existentes. Se o L0 não decidir uma alegação, ela
deve ser removida do lote ou classificada `SPEC-GAP`; saneamento que altere semântica de
citação é proibido por este passo.

## Protocolo segregado

### A — Assessment e adversário L0

1. Após integrar P0096, criar branch nova e
   `00_nucleo/assessments/0026-extracao-source-constant-rust.md`.
2. A lê somente Assessment e insumos hash-pinned, classifica cada preflight e confirma que
   citações/V21/V22 semânticos estão fora.
3. Se necessário, sanear apenas o L0 estrutural e resselar antes de B1/B2.

### B1 — gate cego de identidade/localização

B1 cria exclusivamente `tests/rust_source_constant_identity_assessment.rs`. Deve passar
fonte Rust pela API pública do `RustParser` e inspecionar apenas `ParsedFile.constants`.
Cobre kind, snippet, linha/coluna, ordem, multiplicidade, nesting, Unicode e casos
negativos autorizados. Não importa ou chama regras V21/V22, não lê produção para derivar
expected e não inspeciona `citation`.

### B2 — gate cego de contexto estrutural

B2 cria exclusivamente `tests/rust_source_constant_context_assessment.rs`. Também usa
fonte→IR e confronta casos próximos negativos do recorte: literais fora de função,
strings/chars/bytes, constantes nomeadas, macros, ranges, patterns, whitespace e erro
sintático sem IR parcial. Não chama V21/V22, não compartilha helpers/fixtures com B1 e não
inspeciona nenhum campo fora de kind/snippet/linha/coluna/ordem/multiplicidade.

### C — confronto e correção

Somente após B1/B2 congelados, confrontar a seam em `03_infra/rs_parser.rs` e os tipos L1
estritamente necessários. Produção só muda mediante RED causal. Mudança de `citation`,
configuração, wiring ou regras é proibida.

### D — adversário final

D verifica hashes, independência dos gates, causalidade RED→GREEN, matriz de literals e
contextos, determinismo, ausência de oráculo circular, os dois consumidores, arquitetura
Tekt e regressões 0001–0025. Deve buscar campos populados incidentalmente e alegações
sobre citações escondidas em nomes/asserts.

## Consumidores e regressões obrigatórias

Além de B1/B2, executar separadamente:

- testes fechados do Assessment 0017/V21;
- testes/fixtures de `provenance_inventory` V22 e Assessment 0004;
- testes dirigidos do parser Rust;
- suíte completa do workspace.

Falha de consumidor causada pelo extractor é `RED`. Alterar V21/V22 para acomodar o gate
é proibido. Falha histórica não causada pelo lote deve ser registrada e não corrigida por
expansão silenciosa.

## Classificações e fechamento

- `RED`: fonte→IR contradiz alegação estrutural congelada;
- `SPEC-GAP`: L0 não decide o fato ou a fronteira encosta em citações/agregação;
- `GATE-DEFECT`: gate usa V21/V22/produção como oráculo, compartilha material proibido ou
  exige campo não publicado;
- `PASS`: alegação confrontada sem divergência.

Fechar somente como `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. hashes L0 e Assessment 0026 congelados;
2. B1/B2 em arquivos e identidades separados;
3. RED inicial registrado antes de qualquer correção;
4. `cargo test --test rust_source_constant_identity_assessment --quiet`;
5. `cargo test --test rust_source_constant_context_assessment --quiet`;
6. regressões V21 e V22 separadas;
7. `cargo test --workspace --quiet`;
8. busca por chamadas a V21/V22 e asserts de `citation` dentro dos gates;
9. auto-lint V5/V6/V7/V12;
10. reparador de hashes V5 em dry-run;
11. `rustfmt --check` dirigido e `git diff --check`;
12. adversário D, relatório e worktree limpo.

## Saídas esperadas

- Assessment 0026;
- gates B1/B2 hash-frozen;
- saneamento L0 estrutural, se necessário;
- correção mínima apenas após RED;
- matriz fonte→AST→`SourceConstant`→V21/V22;
- `00_nucleo/relatorio-p0097-auditoria-extracao-source-constant-rust.md`;
- veredito final.

P0097 não autoriza merge, push, instalação, release nem alteração da semântica de
citações. Sem integração prévia do P0096, a execução deve parar antes de criar branch
concorrente.
