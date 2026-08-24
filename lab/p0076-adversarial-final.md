# P0076 — revisão adversarial final (Agente C)

## Veredito

**NÃO REABRIR.** O RED de ordem pública da V11 foi fechado sem mudança observável na semântica de pertinência. O probe independente passou em 6/6 propriedades.

Escopo lido: passo P0076 integral, prompt causal final, `dangling_contract.rs` e entidades necessárias. Não foi lido `tests/mechanical_rule_classifiers_assessment.rs`, nem mensagens ou artefatos do Agente B. Produção e gate não foram alterados.

## Ataques executados

| # | Ataque | Critério mecânico | Resultado |
|---|---|---|---|
| P1 | 512 construções equivalentes com `HashSet` novos e inserções variadas. | O vetor completo de `Violation`, não apenas o conjunto de nomes, é byte a byte igual ao baseline em todas as construções. | **PASS** |
| P2 | Tabela hostil de identidades: `A`, `AA`, `Aa`, `a`, `aA`, `e◌́`, `É`, `é`, inserida ao contrário. | Resultado segue exatamente a ordem nativa de `str`; caixa, prefixos e NFC/NFD permanecem distintos. | **PASS** |
| P3 | Implementação concreta e blanket sobrepostos, entradas repetidas e nomes externos. | O conjunto pendente é exatamente `declared - (implemented ∪ blanket)`; overlap satisfaz uma vez e duplicatas são neutras. | **PASS** |
| P4 | Implementações unrelated e nomes que apenas compartilham prefixo/sufixo com declarações. | Não há satisfação por substring: `A` e `AA` continuam ambos pendentes. | **PASS** |
| P5 | Dois pendentes Unicode sob cada nível `Info`, `Warning`, `Error` e `Fatal`. | Cardinalidade 2, `rule_id=V11`, nível injetado, location `01_core/contracts:(0,0)` e evidência da mensagem são preservados. | **PASS** |
| P6 | Índice vazio e conjunto declarado integralmente satisfeito. | Ambos retornam vetor vazio. | **PASS** |

## Evidência da correção

A implementação agora separa classificação e apresentação:

1. filtra as referências segundo a diferença congelada;
2. copia as referências pendentes para um `Vec<&str>`;
3. aplica `sort_unstable()` diretamente sobre `&str`;
4. somente então constrói as violações.

Isso remove a semente e a ordem de inserção do `HashSet` da saída pública. A ordenação não passa por mensagem, locale, lowercase ou normalização, preservando representações visualmente próximas como identidades distintas.

## Reprodução

Probe preservado fora do auto-lint: `lab/p0076_adversarial_final_probe.rs.txt`.

```sh
cargo build --lib
cp lab/p0076_adversarial_final_probe.rs.txt /tmp/p0076_adversarial_final_probe.rs
rustc --edition=2021 /tmp/p0076_adversarial_final_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/p0076_adversarial_final_probe
/tmp/p0076_adversarial_final_probe
```

Saída observada:

```text
PASS P1 512 equivalent constructions are byte-identical
PASS P2 native str order/case/prefix/NFC-NFD
PASS P3 concrete+blanket overlap and duplicate neutrality
PASS P4 unrelated/superset names do not satisfy by substring
PASS P5 level/location/message/cardinality preserved
PASS P6 empty and fully-satisfied identities
```

## Limite residual

A identidade continua sendo o nome simples da trait, conforme limitação explícita do prompt causal. Colisões entre traits homônimas de módulos distintos permanecem fora do P0076 e não constituem regressão desta correção.
