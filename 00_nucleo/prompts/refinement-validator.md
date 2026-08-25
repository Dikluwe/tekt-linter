# Prompt: Validador direcional de refinamento
Hash do Código: ac51c390

> **Estado:** VIGENTE — ADR-0019 aprovado pelo humano em 2026-08-23
> **Camadas:** L1–L4
> **Materialização:** Etapas A, B1 e B2 no branch `codex/refinement-validator`

## Intenção

Comparar dois conjuntos de fatos observáveis extraídos de artefatos e decidir, de modo
determinístico e direcional, se o alvo preserva um contrato declarado pela fonte.

O validador certifica somente a transformação concreta e o fragmento modelado. Não
prova o transformador, não demonstra equivalência funcional geral e não interpreta
silêncio como prova.

## Vocabulário obrigatório

- `ArtifactFacts`: snapshot canônico e versionado de observáveis de um artefato.
- `Observable`: chave estável, valor e proveniência da evidência.
- `RefinementContract`: relações declaradas entre observáveis fonte/alvo.
- `RefinementVerdict`: `Preserved`, `Violated` ou `Unknown`.
- `Witness`: contraexemplo estruturado ao contrato sobre fatos observados.
- `UnknownReason`: causa fechada e acionável da insuficiência de prova.

Não chamar `Witness` de entrada executável do programa, salvo se uma versão futura
realmente executar ou simbolizar o domínio.

## Relações mínimas

```text
preserve(source, target)
may_normalize(source, target, accepted_targets)
must_not_invent(target)
```

`preserve` exige o mesmo valor conhecido. `may_normalize` aceita igualdade ou valor
alvo explicitamente enumerado. `must_not_invent` exige ausência conhecida do fato no
alvo. Relações adicionais exigem evidência e revisão do ADR.

## Resultado

```text
Preserved
Violated(Witness {
  contract_id,
  relation,
  source_artifact,
  target_artifact,
  source_observable,
  target_observable,
  evidence,
})
Unknown {
  contract_id,
  reason,
  affected_observables,
}
```

Ordem de precedência para múltiplas relações:

1. qualquer `Violated` demonstrado vence;
2. sem violação, qualquer `Unknown` impede `Preserved`;
3. somente todas as relações demonstradas produzem `Preserved`.

A implementação não pode retornar cedo em `Unknown` se outra relação independente já
possuir evidência suficiente para `Violated`. Deve agregar resultados determinística e
puramente.

## Invariantes

- Fonte e alvo não são intercambiáveis.
- Ausência de evidência não equivale a ausência conhecida de um fato.
- Identidade de observável não é inferida apenas por nome semelhante.
- Normalização é opt-in, enumerada e auditável.
- Comparação independe da ordem de entrada dos mapas e relações.
- Versão do formato e do extrator participa da proveniência.
- L1 não conhece arquivos, Git, tree-sitter, TOML, JSON, SARIF ou relógio.
- Nenhuma camada principal importa de `lab/`.
- Snapshot inválido produz erro de entrada ou `Unknown` tipado, nunca `Preserved`.

## Primeira entrega proposta

Comparar dois snapshots explícitos:

```bash
crystalline-lint refine \
  --before before.refinement.json \
  --after after.refinement.json \
  --contract refinement.toml
```

Formato `text` é o padrão; `sarif` também é suportado. Exit codes: `0` para
`Preserved`, `1` quando houver `Violated`, `2` para `Unknown` sem violação ou erro de
entrada. Não implementar leitura de commits nem execução de comandos nessa entrega.

## Contrato do loader explícito — saneamento P0095

O loader L3 publica, em `crystalline_lint::infra::refinement_snapshot`:

```rust
pub fn load_snapshot(path: &Path) -> Result<ArtifactFacts, String>;
pub fn load_snapshot_from_bytes(bytes: &[u8], source: &str) -> Result<ArtifactFacts, String>;
pub fn load_contract(path: &Path) -> Result<RefinementContract, String>;
pub fn load_contract_from_bytes(bytes: &[u8], source: &str)
    -> Result<RefinementContract, String>;
```

Erros continuam strings para não ampliar a API, mas começam por uma classe estável:
`io:`, `invalid-utf8:`, `json-syntax:`, `toml-syntax:`, `unsupported-version:`,
`schema:`, `limit:` ou `concurrent-modification:`. Somente o prefixo é contrato; texto de
I/O, posição e restante da mensagem são informativos. `source` é rótulo opaco preservado
na mensagem e nunca é aberto, executado ou reinterpretado por APIs `from_bytes`.

### Snapshot JSON v1

O objeto raiz é fechado e contém exatamente `format_version`, `artifact_id`,
`extractor_version` e `observables`. Campos desconhecidos e campos/chaves duplicados em
qualquer objeto falham com `schema:`. `format_version` é inteiro JSON e deve ser 1.
`artifact_id` e `extractor_version` são strings com `trim` não vazio; os bytes originais
são preservados, sem normalização.

`observables` é objeto obrigatório com chaves de `trim` não vazio, preservadas
byte-a-byte. Cada valor é objeto fechado:

- `{"state":"known","value":<string>}`; valor vazio/whitespace é válido e preservado;
- `{"state":"absent"}`;
- `{"state":"unknown","reason":...}` com razão exata entre
  `missing-observable`, `ambiguous-identity`, `unsupported-parser`,
  `opaque-construction`, `partial-contract`, `budget-exhausted`.

Campos proibidos, estado/razão desconhecidos e payload ausente falham com `schema:`.
Ordem de propriedades não altera `ArtifactFacts`.

### Contrato TOML

O documento raiz é fechado: `id`, zero ou mais `[[observable]]` pertencentes ao extrator
L3 e uma ou mais `[[relation]]`. O loader de relações reconhece `observable`, mas não o
materializa nem substitui sua validação pelo loader de extração. `id`, `source`, `target`
e valores de `accepted_targets` são preservados e exigem `trim` não vazio quando
presentes. Kinds são case-sensitive:

- `preserve`: exatamente `kind`, `source`, `target`;
- `may-normalize`: exatamente `kind`, `source`, `target`, `accepted_targets`; lista não
  vazia e sem valores duplicados;
- `must-not-invent`: exatamente `kind`, `target`; `source` e `accepted_targets` são
  proibidos.

Campos/tabelas/chaves duplicadas ou desconhecidas falham fechados. Ordem de relações e
de `accepted_targets` é preservada. Relação estruturalmente duplicada é rejeitada. Para o
mesmo target, `must-not-invent` conflita com qualquer outra relação. Para o mesmo par
source/target, coexistência de `preserve` e `may-normalize`, ou duas `may-normalize`, é
rejeitada. Igualdade textual entre source, target e um accepted target não é, sozinha,
erro estrutural.

### Limites e leitura

Antes do parse: máximo 4 MiB por artefato. Depois do parse: no máximo 4096 observáveis,
4096 relações, 4096 accepted targets por relação, 16384 accepted targets no total e
64 KiB por string. Excesso é `limit:`, nunca fato `Unknown`.

As APIs por path aceitam path explicitamente escolhido pelo usuário, sem confinamento a
uma raiz, mas exigem arquivo regular e rejeitam symlink em qualquer componente,
diretório, FIFO, socket e device antes de ler. Abrem somente para leitura, limitam bytes
durante a leitura e comparam tamanho e modificação do mesmo handle antes/depois; mudança
observada é `concurrent-modification:`. Não escrevem, criam ou executam nada. As APIs por
bytes são o oráculo de schema; arquivo regular estável deve produzir o mesmo valor/classe.

L1 mantém tipos e comparação; L3 somente lê/desserializa/valida; L2 apresenta e decide
exit; L4 coordena. O loader nunca produz `PRESERVED`, `VIOLATED` ou `UNKNOWN`.

## Etapa B1 — geração de snapshots Rust

O subcomando vigente é:

```bash
crystalline-lint snapshot \
  --contract refinement.toml \
  --artifact-id working-tree \
  --output working-tree.refinement.json \
  <project-root>
```

Contrato de extração:

```toml
[[observable]]
key = "refinement.verdict.variants"
language = "rust"
file = "01_core/entities/refinement.rs"
query = '''
(enum_item
  name: (type_identifier) @_name
  body: (enum_variant_list) @value
  (#eq? @_name "RefinementVerdict"))
'''
capture = "value"
cardinality = "one"
on_missing = "unknown"
```

`cardinality` aceita `one` ou `many`; `on_missing` aceita `unknown` ou `absent`.
Em `one`, múltiplas capturas viram `Unknown(AmbiguousIdentity)`. Em `many`, textos são
normalizados, ordenados e codificados deterministicamente como lista JSON compacta.
Zero capturas respeita `on_missing`. Query inválida e path inseguro são erros de
entrada; erro sintático do artefato vira `Unknown(OpaqueConstruction)`.

O extrator inicial aceita apenas `language = "rust"`. O snapshot usa
`format_version = 1`, `extractor_version = "crystalline-rust-query-v1"`, não contém
timestamp e é escrito atomicamente. L1 recebe capturas já normalizadas e decide apenas
cardinalidade/ausência; tree-sitter, TOML, JSON e filesystem permanecem em L3.

## Cenários RED

```text
Dado variations=wght=650 na fonte e variations=wght=650 no alvo
E relação preserve
Quando comparar
Então Preserved

Dado variations=wght=650 na fonte e variations=default no alvo
Quando comparar
Então Violated com Witness contendo ambos os valores

Dado weight=bold na fonte e weight=700 no alvo
E may_normalize aceita 700
Quando comparar
Então Preserved

Dado a mesma transformação sem normalização declarada
Quando comparar
Então Violated

Dado radius.state=contextual na fonte e radius.state=erased no alvo
Quando comparar
Então Violated

Dado nenhum proxy-owner na fonte e proxy-owner no alvo
E must_not_invent(proxy-owner)
Quando comparar
Então Violated

Dado fonte proveniente de macro opaca
Quando comparar
Então Unknown(MacroOpaque), nunca Preserved

Dadas duas relações, uma Unknown e outra Violated
Quando agregar
Então Violated vence e a incerteza permanece nos detalhes

Dadas as mesmas relações e mapas em ordens diferentes
Quando comparar
Então veredito e serialização são idênticos
```

## Etapa B2 — comparação de revisões Git imutáveis

O subcomando aprovado é:

```bash
crystalline-lint refine-revisions <repository-root> \
  --before-ref <sha-ou-ref> --after-ref <sha-ou-ref> \
  --contract refinement.toml
```

Cada ref é resolvida uma única vez para um commit OID. A partir daí somente o OID
imutável participa da enumeração de trees, extração, `artifact_id` e testemunhas. A
fonte Git fornece bytes por path lógico ao mesmo extrator B1; não existe comparador,
normalizador ou semântica de query paralela.

O adapter usa processos Git locais com argumentos separados, nunca shell. Deve
desabilitar prompts, lazy fetch, replace objects, locks opcionais, protocolos externos,
hooks, configuração global/sistema, filtros, textconv e seguimento de symlinks. Não
executa checkout, worktree, stash, build, LFS ou submódulo. Working tree, índice, HEAD,
branch e stash são ignorados e não podem ser alterados.

Aceitam-se apenas blobs regulares. Arquivo ausente respeita `on_missing`; objeto
esperado ausente, symlink, submódulo, framing inválido e erro de leitura são
inconclusivos ou erro de entrada, nunca ausência conhecida. Limites iniciais: 512
paths, 4 MiB por blob e 32 MiB por revisão. Excesso produz
`Unknown(BudgetExhausted)` sem truncamento. O backend não adiciona biblioteca Git e
mantém `snapshot + refine` como fallback sem autoridade de subprocesso.

## Limites da primeira versão

- fatos finitos previamente extraídos;
- sem execução simbólica ou concreta;
- sem memória, aliases interprocedurais ou macros expandidas;
- sem manipulação do worktree;
- sem solver SMT;
- suporte inicial de extração pode ser apenas Rust, mas L1 permanece neutro.

## Relação com diagnósticos existentes

V6 continua verificando interface atual contra snapshot do prompt. V23–V25 continuam
verificando um estado local sob contratos semânticos. O modo `refine` compara dois
estados. Se compartilhar observações com essas regras, a configuração deve possuir uma
fonte única e a apresentação deve evitar duplicidade.

## Critérios de aceitação futuros

1. ADR-0019 aprovado antes de L1–L4.
2. Fixtures RED existem antes do comparador de produto.
3. `Unknown` não é convertido em sucesso na saída ou exit code por omissão silenciosa.
4. Toda violação contém testemunha estável e serializável.
5. Testes provam direcionalidade, precedência e determinismo.
6. Oráculos históricos do `typst-crystalline` são reduzidos a fixtures locais.
7. V6 e V23–V25 não sofrem regressão ou duplicação.
8. Auto-lint e hashes passam após a materialização autorizada.

## Histórico de revisões

| Data | Estado | Motivo |
|---|---|---|
| 2026-08-23 | Proposto | Hipótese confirmada pelo experimento descartável e ADR-0019 |
| 2026-08-23 | Vigente | Humano aprovou materialização segura da Etapa A em branch dedicado |
| 2026-08-24 | Vigente | Humano aprovou Etapa B1: snapshot Rust por queries explícitas, sem Git |
| 2026-08-24 | Vigente | Humano aprovou Etapa B2: leitura imutável por Git batch, sem checkout ou rede |
