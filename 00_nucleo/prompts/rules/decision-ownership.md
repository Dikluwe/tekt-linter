# Prompt — V25 `DecisionOwnership`
Hash do Código: 181d2563

**Status:** VIGENTE; ADR-0018 aceito em 2026-08-23
**Camada futura:** L1 local e agregação global, com fatos AST produzidos por L3
**Idioma inicial:** Rust; contrato e IR agnósticos de linguagem

## Intenção

Garantir que uma decisão semântica nomeada possua um único owner e que consumidores não
recomponham a decisão por proxy nem reexecutem canonicalizadores depois do marco em que
o valor foi declarado resolvido.

V25 compara identidades explícitas de `[[semantic.decision]]`; nunca usa similaridade
textual entre funções.

## Contrato

Cada entrada declara `id`, `language`, `owner`, `consumers`, proxies proibidos opcionais,
canonicalizadores e marcos `resolved_after`. Um owner é símbolo normalizado único.

São violações:

- mais de um owner efetivo para o mesmo `id`;
- consumidor combinar resultado explícito com proxy declarado;
- consumidor implementar diretamente uma decisão reservada ao owner;
- canonicalizador chamado em escopo posterior ao marco resolvido.

Consumidor chamar o owner é o caminho legítimo. Uma heurística pode ser owner de outra
identidade sem conflito.

## Diagnóstico

`V25 DecisionOwnership`: informar identidade, owner esperado, consumidor e modalidade
(`duplicate-owner`, `proxy-reentry`, `canonicalizer-reentry` ou
`direct-reimplementation`). Nível padrão `warning`.

## Cenários

```text
Dado owner explicit_math e proxy family_name.contains("math")
Quando consumidor calcula explicit || proxy
Então emitir V25 proxy-reentry

Dado dois owners declarados para a mesma identidade
Então emitir V25 duplicate-owner

Dado map_glyph como canonicalizador e marco shaped_glyph como resolvido
Quando consumidor posterior chama map_glyph novamente
Então emitir V25 canonicalizer-reentry

Dado consumidor que chama o owner
Então não emitir

Dado contains("math") como owner de outra identidade
Então não emitir

Dado canonicalização dentro do owner e antes do marco
Então não emitir
```

## Restrições

- Proibido detectar duplicação por texto parecido, nome parecido ou hash de corpo.
- O índice global agrega declarações por `id`; ordem do rayon não altera o resultado.
- Macros opacas e dispatch dinâmico são não analisáveis.
- Seleção `v25` não executa V23/V24.

## Fronteira executável L3 → L1

Owner efetivo, identidade, composição com proxy e posição relativa a `resolved_after`
são decididos por L3/agregação. Repetir o mesmo fato não é assumido idempotente no
classificador: `SemanticObservation` representa ocorrência diagnóstica já decidida.

L1 consome a API pública `SemanticObservationKind`, `SemanticObservation` e
`HasSemanticObservations` definida integralmente no L0 de V23. A variante
`DirectDecisionReimplementation` representa a quarta modalidade exigida pela intenção.

V25 expõe
`check<'a, T: HasSemanticObservations<'a>>(file: &T, level: ViolationLevel)
-> Vec<Violation<'a>>` e mapeia, em ordem:

| Kind | Modalidade textual |
|---|---|
| `DuplicateDecisionOwner` | `duplicate-owner` |
| `DecisionProxyReentry` | `proxy-reentry` |
| `CanonicalizerReentry` | `canonicalizer-reentry` |
| `DirectDecisionReimplementation` | `direct-reimplementation` |

As três variantes V23/V24 são ignoradas. Cada ocorrência gera uma violação V25; nível
preserva o parâmetro; mensagem contém `contract_id`, modalidade e `detail` verbatim;
location usa path do arquivo e linha/coluna da observação. `language()` é irrelevante em
L1 porque a aplicabilidade já foi decidida upstream.

O módulo é importável por `crystalline_lint::rules::decision_ownership`; tipos por
`crystalline_lint::entities::rule_traits`, `layer` e `violation`. O gate L1 não prova a
correção do extrator/agregador; owners, duplicatas de entrada, temporalidade e contratos
cruzados permanecem para o lote posterior.
