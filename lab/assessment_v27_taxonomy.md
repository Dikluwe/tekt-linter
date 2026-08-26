# Assessment P0115 — refino taxonômico de V27

> **Estado:** LAB CLOSED — KEEP V27 EXPERIMENTAL
> **Regime:** protocolo completo executado como ensaio segregado por ordem e artefatos;
> sem atestação forte de isolamento de autoridades
> **Baseline tekt-linter:** `3fd957745d1b3dec264d2924c0b35a4804f6e4c2`
> **Corpus Typst Crystalline:** revisão observada
> `7fb5bb6d9ee73298af4fd09bb858cb26e5fdd568`, fixada por manifesto de bytes
> **V28:** `VACANT — inverse-complexity hypothesis discarded by later human decision`

## 1. Regime, autoridades e limitação de atestação

P0115 alterou a semântica do classificador experimental e exigiu contrato, ataques e
veredito. Foi usado o protocolo completo da skill `tekt-materializacao-segregada` e a
ADR-0020.

| Papel | Entrada | Saída | Capacidade efetiva |
|---|---|---|---|
| intenção | P0115 e P0114 | taxonomia congelada | leitura integral; P0115 já escrito |
| contrato/oráculo | taxonomia, corpus e fixtures anteriores | expected TSV e ataques | escrita somente em `lab/` durante a fase |
| implementador | P0115 e expected congelado | probe refinado | escreveu o probe após o baseline |
| adversário | contrato, baseline e corpus | ataques de operador e escopo de comentário | executado depois de cada candidato |
| verificador | hashes, outputs e gates | este assessment | mesma autoridade humana/modelo da sessão |

Uma única autoridade executora teve acesso ao conjunto completo. Logo, os artefatos são
ordenados e reproduzíveis, mas não independentes em sentido forte. Strings de papel e
hashes não suprem isolamento de capacidade. O veredito correto é **executado sem
atestação de isolamento**, não “segregado e atestado”.

## 2. Entradas congeladas

| Artefato | SHA-256 |
|---|---|
| manifesto de 450 fontes Rust | `cd828484621321e9bd6e466dbc822621e51027939fd1c5988dcbb8dfc49de862` |
| saída bruta P0114 | `629fd65936dccac1e8996c9b6514b77592f4c5db526353ebc54304b5acec657c` |
| probe P0114 | `ffd5277ed690c7894f317dbbc92d3b2969e6bec796b3f316e7f0bfc6ad1786f3` |
| fixtures P0114 | `00992cfe6c300778c7bea2c458e8a1851cf1bf189f1aacb7690154907c538516` |

O manifesto contém 450 linhas `sha256 + path` para `01_core`–`04_wiring`. Todos os
arquivos foram parseados. Mudanças documentais já existentes no checkout consumidor não
entraram na identidade das fontes Rust.

Baseline observado:

```text
V19=349
V20=600
V27 P0114: 40 PROVEN-SYNTACTIC + 190 UNKNOWN = 230 grupos
```

## 3. Contrato taxonômico e política de Unknown

O classificador final distingue:

- `ALIAS-EQUIVALENCE`;
- `DECISION-EQUIVALENCE`;
- `CONFIGURED-EQUIVALENCE`;
- `DECLARATIVE-TABLE`;
- `EVIDENCE-PRESERVING-SEPARATION`;
- `EMPTY-EQUIVALENCE`;
- `MACRO-EQUIVALENCE`;
- `OVERLAP-OR-SUBSUMPTION`;
- `BINDING-DEPENDENT`;
- `UNKNOWN`.

Somente alias e decisão fechada entram na contagem forte. Bindings, macros, padrões
sobrepostos e conhecimento insuficiente nunca são promovidos por default.

Para alias, um mesmo comentário precisa nomear explicitamente todos os literais e declarar
alias/sinônimo. Evidência genérica no mesmo `match` é insuficiente. Essa política causa um
falso negativo deliberado em `cyan/aqua`, preferível a propagar evidência para o grupo
errado.

## 4. Gate discriminatório

As fixtures compilam como crate Rust e o expected TSV fixa owner+classe. O gate passou
antes e depois de `rustfmt`.

Ataques válidos rejeitados:

1. corpo com `==` versus `!=`;
2. guard distinto;
3. tabela extensa com corpos repetidos;
4. evidência individual em somente um braço;
5. binding/desestruturação incompatível com o subconjunto forte;
6. comentário de alias aplicável a um grupo vizinho, mas não ao outro.

Resultado:

```text
mutation_score = 6/6 = 1.0
reordenação por formatação = invariante
Unknown convertido em Preserved = 0
```

O gate encontrou e fechou dois defeitos durante a execução:

### GATE-DEFECT 1 — operadores apagados

O fingerprint P0114 percorria somente filhos nomeados da AST. Operadores como `==` e `!=`
eram tokens não nomeados e desapareciam. Isso marcou incorretamente dois pares de
`ordering.rs` como equivalentes. A normalização passou a preservar todos os filhos,
ignorando somente comentários e whitespace não materializado.

### GATE-DEFECT 2 — escopo amplo de alias

Procurar `alias` em todo o texto do `match` permitia atribuir o comentário de um grupo a
outro. O proxy final exige que o mesmo comentário nomeie todos os padrões do grupo.

## 5. Autoridade dos 40 fortes originais

Os 40 casos foram classificados integralmente em
`lab/v27_typst_strong_authority.tsv`:

| Classe humana | Quantidade |
|---|---:|
| `DECLARATIVE-TABLE` | 35 |
| `FALSE-EQUIVALENCE` | 2 |
| `DECISION-EQUIVALENCE` | 2 |
| `ALIAS-EQUIVALENCE` | 1 |

Os 35 casos declarativos incluem 29 grupos de `03_infra/src/fonts.rs`; o restante são
tabelas de símbolos, keywords, lexer, classes matemáticas e parâmetros. Eles não entram
na contagem forte.

Os dois falsos do P0114 eram:

```rust
BinOp::Lt  => ord == Ordering::Less
BinOp::Geq => ord != Ordering::Less
```

e o par simétrico. O apagamento do operador explicava ambos.

Os dois fortes detectáveis são:

- `SyntaxKind::Underscore | SyntaxKind::Hat` em `math_op`;
- `Value::Auto | Value::None` em `extract_usize_or_none_min`.

O terceiro forte humano é `"cyan"/"aqua"`. O comentário P477 declara duas aliases CSS;
`gray/grey` já está condensado, deixando `cyan/aqua` como o segundo par. Como o comentário
não nomeia diretamente ambos, o detector final classifica o match como tabela e preserva
o caso como falso negativo conhecido.

## 6. Amostra dos 190 Unknown

Foram confrontados 30 casos estratificados em `lab/v27_typst_unknown_sample.tsv`:

| Destino refinado | Quantidade na amostra |
|---|---:|
| `OVERLAP-OR-SUBSUMPTION` | 14 |
| `BINDING-DEPENDENT` | 10 |
| `MACRO-EQUIVALENCE` | 3 |
| `EMPTY-EQUIVALENCE` | 2 |
| deixou de ser equivalente após preservar operadores | 1 |

Nenhum caso da amostra foi convertido implicitamente em forte.

## 7. Resultado final no Typst Crystalline

O probe refinado produziu 209 grupos:

| Classe | Quantidade |
|---|---:|
| `OVERLAP-OR-SUBSUMPTION` | 91 |
| `BINDING-DEPENDENT` | 69 |
| `DECLARATIVE-TABLE` | 36 |
| `EMPTY-EQUIVALENCE` | 6 |
| `MACRO-EQUIVALENCE` | 5 |
| `DECISION-EQUIVALENCE` | 2 |
| `ALIAS-EQUIVALENCE` | 0 |

Comparação:

```text
P0114 bruto:       230 grupos, 40 fortes alegados
P0115 refinado:    209 grupos,  2 fortes automáticos
autoridade humana:              3 fortes reais
```

Tabela de confusão da contagem forte contra a autoridade dos 40:

```text
TP=2  FP=0  FN=1  TN/não-forte=37
precision=100%
recall=66,7%
```

A precisão exigida foi alcançada, mas o critério de pelo menos três fortes detectáveis em
dois módulos não foi: restaram dois fortes automáticos. O alias real requer conhecimento
contextual que o proxy deliberadamente não inventa.

## 8. Hashes de saída

| Artefato | SHA-256 |
|---|---|
| probe refinado V27, após remoção do experimento V28 | `69a122d37e57bb3c1fae176dd42534c53996639ce4c4b834ac2a90c3360ffd69` |
| fixtures taxonômicas | `5a8949d812f3a92131f9738b6870d7d15252182c7365576977addd4fa6008b8a` |
| expected TSV | `52857e38342f2021d9dec0bd0ebe854cf46225f201896d9428c6c50705b77cf6` |
| saída taxonômica | `ca218a134825b528cf327800ce8ba715aa77af2474a3529f57f50891de3cb5e9` |
| autoridade dos 40 | `a13fe945239edc5a37c8aa63ce8e7ed1bc83153dd54298a36a46874e6a1d2914` |
| amostra Unknown | `da3283310cde0ef16bbf6ec2b5b575962680ecb9d59c38023f0928055f86d370` |

Os hashes acima correspondem aos artefatos finais; o hash do probe foi atualizado depois
da remoção integral do experimento V28.

## 9. Veredito

**LAB CLOSED — KEEP V27 EXPERIMENTAL.**

V27 tem uma taxonomia útil e precisão forte sem falsos positivos no corpus revisado, mas
o sinal automático ficou abaixo do mínimo e a classe alias exige evidência de domínio que
o parser não pode atribuir genericamente. Não registrar V27, não reservar definitivamente
o ID e não implementar autofix.

Uma reabertura pode investigar contratos explícitos de alias ou um modo puramente métrico
que reporte tabelas separadamente, mas precisa de novo passo e novo corpus.

Uma decisão humana posterior substituiu o estado suspenso original. V20 é métrica
unidirecional de teto; especificidade e segurança de tipos não justificam complexidade
mínima. O estado vigente é:

```text
V28 = VACANT — no inverse-complexity rule
```

Ela não participou do veredito de V27. Seu código experimental e suas fixtures foram
removidos; o ID pode ser usado futuramente por outra obrigação independente.
