# P0075 — revisão adversarial final (Agente C)

## Veredito

**NÃO REABRIR.** Os dois REDs congelados foram fechados e não encontrei regressão equivalente no escopo das sete decisões do passo. O probe independente passou em 6/6 propriedades.

Escopo lido: passo P0075 integral, prompt causal `prompt-stale.md`, produção final de `prompt_stale.rs` e entidades necessárias. Não foi lido `tests/provenance_rules_assessment.rs`, nem mensagens ou artefatos do Agente B. Produção e testes não foram alterados.

## Ataques executados

| # | Ataque | Critério mecânico | Resultado |
|---|---|---|---|
| P1 | Multiconjunto nas três famílias, nos dois sentidos: uma duplicata adicionada e uma removida em funções, tipos e reexports. | Cada caso deixa exatamente uma ocorrência no vetor correto e zero nos outros cinco. | **PASS** |
| P2 | Permutações com duplicatas de mesma multiplicidade nas três famílias; comparação nos dois sentidos. | `compute_delta(a,b)` e `compute_delta(b,a)` são vazios. | **PASS** |
| P3 | Empates de função por todos os campos: mesmo nome, params prefixais/diferentes e `return_type` `None`/`Some`, com entrada invertida. | Deltas das duas enumerações são estruturalmente iguais e seguem `(name, params, return_type)`. | **PASS** |
| P4 | Empates de tipo por nome, todas as seis variantes de kind e members prefixais/diferentes, com entrada invertida. | Ordem observada: `Struct, Enum, Trait, Class, Interface, TypeAlias`; dentro de `Struct`, members fazem o desempate lexicográfico. | **PASS** |
| P5 | Permutar internamente `params` e `members`. | A permutação não é normalizada: gera uma adição e uma remoção em cada família afetada. | **PASS** |
| P6 | Permutar simultaneamente os seis grupos do delta: funções/tipos/reexports adicionados e removidos. | A mensagem completa de V6 é byte a byte idêntica nas duas enumerações. | **PASS** |

## Evidência da implementação

O cancelamento usa um bitmap `consumed` por lado: uma ocorrência estruturalmente igual só cancela uma posição ainda não consumida. Isso implementa diferença de multiconjuntos, inclusive quando há duplicatas idênticas.

Depois do cancelamento, cada vetor é ordenado antes da construção de `InterfaceDelta`:

- funções por `(name, params, return_type)`;
- tipos por `(name, kind_rank, members)`;
- reexports por texto.

O `kind_rank` é uma correspondência explícita das seis variantes e não depende do discriminante do enum. `InterfaceDelta::describe` percorre os seis grupos na ordem pública congelada, portanto recebe dados já canônicos.

## Reprodução

Probe preservado como arquivo não escaneável: `lab/p0075_adversarial_final_probe.rs.txt`.

```sh
cargo build --lib
cp lab/p0075_adversarial_final_probe.rs.txt /tmp/p0075_adversarial_final_probe.rs
rustc --edition=2021 /tmp/p0075_adversarial_final_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/p0075_adversarial_final_probe
/tmp/p0075_adversarial_final_probe
```

Saída observada:

```text
PASS P1 multiset +/- one occurrence in all three families
PASS P2 permutation with duplicates is empty in both directions
PASS P3 function total order covers all fields/ties
PASS P4 explicit kind rank and full members tie-break
PASS P5 params/members order remains semantic
PASS P6 stable V6 message across all six permuted groups
```

## Limite residual

`describe()` exibe nomes de funções e nomes/kinds de tipos, não todos os campos usados no desempate. Isso reduz detalhe diagnóstico para assinaturas homônimas, mas não viola as decisões congeladas do P0075: a representação estrutural do delta é completa, os seis vetores são canônicos e a mensagem é estável. Alterar o formato público da mensagem exigiria decisão fora deste passo.
