# Passo operacional 0095 — auditoria segregada do loader de refinamento

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/audit-refinement-snapshot-loader`
> **Pré-condição:** P0094 integrado em `master`, worktree limpo e branch nova criada a
> partir do merge
> **Predecessor:** P0094

## Objetivo

Auditar a fronteira L3 que desserializa snapshots JSON e contratos TOML explícitos de
refinamento para os tipos fechados L1, hoje materializada em
`03_infra/refinement_snapshot.rs`.

O lote cobre schema, versionamento, estados de observável, relações, validação estrutural,
determinismo e leitura explícita dos dois artefatos. Não reabre comparação semântica L1,
extração Rust, Git imutável, selos/recibos, execução entre revisões ou regras V0–V25.
`citation_freshness` também fica fora: já foi fechado pelo Assessment 0017 com gates
L1/L3 9/9.

## Hipótese e risco

O loader parece de baixo risco porque somente transforma bytes fornecidos pelo usuário em
estruturas finitas. O risco é aceitar estados ambíguos que mais tarde aparentem
`PRESERVED`: campos desconhecidos, chaves duplicadas, identificadores vazios, relações
contraditórias, listas degeneradas ou diferenças de ordem absorvidas silenciosamente.

## Insumos L0 iniciais hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| ADR de refinamento | `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `c2607ff2feb044487b454b3dc3115c9613d8124faebc415dc889eb717038e376` |
| contrato/CLI de refinamento | `00_nucleo/prompts/refinement-validator.md` | `a3a1eb935f5c79e698e0b4a792f36ec70f67c53c9db65c345b27e347c2bcba7d` |
| arquitetura do pipeline | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| tipos de diagnóstico | `00_nucleo/prompts/violation-types.md` | `147afa0d8f3f3e6e30e050590dad0b99c7da8486d3565e3f6c42f7fa883ea4dc` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| Git refinement já fechado | `00_nucleo/assessments/0001-git-refinement.md` | `5a38f20563a865a12dc0c052a2b7a5dd0d46cb17452600c183c8781bce8a5d17` |
| fechamento anterior | `00_nucleo/assessments/0013-fechamento-pre-merge.md` | `3d8d4d1aba216f03d3384ade951a9a46015411c6d4edb53e075e7f29813c5dd9` |
| passo histórico Etapa A | `00_nucleo/tekt-linter-passo-validacao-de-refinamento.md` | `0364668e0adfa53d01f6d17cab9a7298839e9935c4286b4a412f04df559db9bd` |

Qualquer alteração invalida o Assessment 0024 até resselamento.

## Alegações candidatas

### Snapshot JSON

1. Somente `format_version == 1` é aceito; número ausente, negativo, fracionário,
   overflow, string ou versão futura falha explicitamente.
2. `artifact_id` e `extractor_version` são obrigatórios e não vazios após `trim`, mas os
   bytes originais autorizados são preservados ou normalizados somente se o L0 decidir.
3. `observables` é obrigatório e cada chave possui identidade exata. Chave vazia,
   whitespace, duplicata JSON e ordem de objeto exigem política normativa explícita.
4. Cada observável materializa exatamente `Known(value)`, `Absent` ou
   `Unknown(reason)`. Estado/campo desconhecido, payload ausente ou campos incompatíveis
   não são reinterpretados.
5. Todas as razões `UnknownReason` normativas fazem round-trip; reason desconhecida não
   vira uma razão aproximada.
6. Valor `Known` vazio, whitespace, Unicode, controles e payload grande são preservados
   ou rejeitados somente conforme decisão L0, nunca por coerção implícita.

### Contrato TOML

7. `id` é obrigatório e não vazio após `trim`; pelo menos uma `[[relation]]` é exigida.
8. `kind` aceita somente `preserve`, `may-normalize` e `must-not-invent`, com case e
   hífen exatos.
9. `preserve` e `may-normalize` exigem source/target não vazios;
   `must-not-invent` exige target e proíbe ou ignora source somente por decisão explícita.
10. `may-normalize` exige `accepted_targets` não vazio; valores vazios, duplicados,
    repetição do source/target e ordem da lista têm política congelada antes do gate.
11. Campos proibidos por variante, campos TOML desconhecidos, tabelas/relations
    duplicadas e chaves duplicadas falham fechados ou seguem regra L0 explícita.
12. Ordem e multiplicidade de relações são preservadas no loader; canonicalização ou
    rejeição de relações semanticamente duplicadas pertence ao contrato, não ao parser
    incidental.

### Leitura, erros e arquitetura

13. `load_contract_from_bytes` distingue UTF-8 inválido de TOML inválido e preserva o
    identificador `source` hostil na mensagem sem executar ou reinterpretar conteúdo.
14. `load_snapshot`/`load_contract` são read-only e não seguem política implícita de raiz
    confinada para paths explicitamente escolhidos pelo usuário, salvo decisão contrária
    do L0. Symlink, FIFO, diretório, tamanho máximo e troca concorrente são
    `SPEC-GAP` se a autoridade não os classificar.
15. Mesmo input produz o mesmo valor ou mesma classe de erro independentemente de ordem
    de mapas e locale; mensagens não alegam ausência quando houve erro de schema/I/O.
16. L1 possui tipos e comparação; L3 lê/desserializa/valida schema; L2 possui comando e
    apresentação; L4 seleciona e injeta. L3 não compara refinamento nem decide
    `PRESERVED/VIOLATED/UNKNOWN`.

## Preflight normativo obrigatório

O Assessment 0024 e o adversário A devem decidir antes dos gates:

- schema fechado versus tolerância a campos desconhecidos em JSON/TOML;
- política para chaves duplicadas de JSON/TOML;
- vazio/whitespace em chaves de observável e valores `Known`;
- preservação versus trim de ids/versões/source/target;
- campos permitidos/proibidos por variante de relação;
- duplicatas e ordem de `accepted_targets` e relações;
- relação semanticamente duplicada ou contraditória;
- limites de bytes/profundidade/cardinalidade;
- semântica de paths explícitos, symlink, arquivo não regular e troca concorrente;
- estabilidade exigida de mensagens versus somente classe de erro.

Ausência ou contradição é `SPEC-GAP`. O loader vigente e o comportamento padrão de
Serde/TOML não são autoridade normativa.

## Protocolo segregado

### A — Assessment e adversário L0

1. Após integrar P0094, criar branch nova e
   `00_nucleo/assessments/0024-refinement-snapshot-loader.md` com baseline/hash de
   integração.
2. A lê somente Assessment e os nove insumos hash-pinned; classifica todos os gaps.
3. Sanear L0 e publicar assinaturas/tipos mínimos para os gates. Resselar antes de B1/B2.

### B1 — Gate cego de snapshot JSON

Verificador novo cria exclusivamente `tests/refinement_snapshot_loader_assessment.rs`.
Usa diretório temporário confinado e cobre versão, metadata, observáveis, razões, campos
e chaves duplicadas/desconhecidas, Unicode, controles, ordem e erros. Não lê contrato
TOML, produção, B2 ou comparação L1.

### B2 — Gate cego de contrato TOML

Outro verificador cria exclusivamente `tests/refinement_contract_loader_assessment.rs`.
Prioriza `load_contract_from_bytes` para isolar schema de I/O e cobre todas as variantes,
campos condicionais, listas, duplicatas, ordem, UTF-8 e `source` hostil. Uma matriz curta
confirma equivalência com `load_contract` em arquivo regular explícito.

### C — Confronto e correção

Somente após B1/B2 congelados, confrontar `03_infra/refinement_snapshot.rs` e os tipos L1
estritamente necessários. Correção funcional exige RED causal.

A solução deve respeitar:

- L1 mantém `ArtifactFacts`, `ObservableValue`, `UnknownReason`,
  `RefinementContract` e `RefinementRelation`;
- L3 implementa leitura e validação de schema, sem emitir veredito;
- L2/L4 não duplicam parser ou política de campos;
- Git, extractor, seal e comparação não mudam salvo RED causal impossível de fechar na
  seam autorizada.

### D — Adversário final

Verificar hashes, causalidade RED→GREEN, gates independentes, schema fail-closed,
consumidor `refine`, arquitetura Tekt, delta escondido e regressão dos assessments
0001–0023.

## Classificações e fechamento

- `RED`: produção contradiz alegação congelada;
- `SPEC-GAP`: L0 não decide schema, identidade, limites ou erro;
- `GATE-DEFECT`: gate inventa API/política ou usa comparação/produção como oráculo;
- `PASS`: alegação confrontada sem divergência.

Fechar somente como `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

## Validação mínima

1. B1/B2 em identidades e arquivos separados;
2. hashes dos gates registrados antes do confronto;
3. testes dirigidos JSON e TOML;
4. fixtures hostis e propriedades de permutação;
5. `cargo test --workspace --quiet`;
6. smoke do comando `refine` com snapshots/contrato explícitos;
7. auto-lint V5/V6/V7/V12;
8. reparador de hashes em dry-run;
9. `rustfmt --check` dirigido e `git diff --check`;
10. busca por comparação/veredito dentro do loader L3;
11. adversário final e worktree limpo.

## Saídas esperadas

- Assessment 0024;
- gates B1/B2 segregados;
- L0 saneado se necessário;
- correção mínima somente após RED;
- relatório `00_nucleo/relatorio-p0095-auditoria-refinement-snapshot-loader.md`;
- matriz L0→L1/L3→L2/L4→gates;
- veredito final.

P0095 não autoriza merge, push, instalação ou release. Sem integração prévia do P0094,
a execução deve parar antes de criar branch concorrente.
