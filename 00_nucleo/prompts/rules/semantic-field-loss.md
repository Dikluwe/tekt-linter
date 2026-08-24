# Prompt — V24 `SemanticFieldLoss`
Hash do Código: 877d1741

**Status:** VIGENTE; ADR-0018 aceito em 2026-08-23
**Camada futura:** L1, com projeções AST produzidas por L3
**Idioma inicial:** Rust; contrato e IR agnósticos de linguagem

## Intenção

Detectar uma fronteira declarada de projeção, DTO, chave ou identidade na qual um campo
obrigatório da origem é abandonado ou substituído por uma forma neutra.

V24 não procura `default()` globalmente. A relação `origem → destino` deve existir em
`[[semantic.projection]]`.

## Contrato

Cada entrada declara `id`, `language`, `scope`, `source`, `destination`, `neutral_forms`
e `normalization`. O destino pode ser campo nomeado, argumento ou slot de retorno.

`normalization = "preserve"` exige que a expressão do destino dependa da origem. Um
default usado depois de ler um campo opcional (`source.unwrap_or_default()`) preserva a
dependência e é legítimo. Um default independente no slot obrigatório é perda.
`normalization = "drop-to-default"` autoriza explicitamente a normalização.

## Diagnóstico

`V24 SemanticFieldLoss`: informar contrato, campo de origem, destino e forma neutra.
Nível padrão `warning`.

## Cenários

```text
Dado style.variations → return.2 com normalization preserve
Quando return.2 é FontVariations::default() sem depender de style.variations
Então emitir V24

Dado o mesmo contrato
Quando return.2 é style.variations.clone().unwrap_or_default()
Então não emitir

Dado origem opcional ausente no caminho analisado
Então não emitir

Dado normalization drop-to-default
Quando o destino usa default
Então não emitir

Dado Default::default() fora de scope/destination registrados
Então não emitir
```

## Restrições

- Proibido inferir identidade por nome de tipo ou função.
- Proibido assumir que `None`, zero ou `default()` são perda sem contrato.
- Dependência sintática suportada é intraprocedural; caso opaco é não analisável.
- Seleção `v24` não executa V23/V25.

## Fronteira executável L3 → L1

Normalização, dependência sintática, ausência comprovada, opacidade, scope e destination
são decididos por L3. L1 consome a API pública `SemanticObservationKind`,
`SemanticObservation` e `HasSemanticObservations` definida integralmente no L0 de V23,
SHA-256 autorizado pelo assessment vigente. A taxonomia inclui as sete variantes ali
enumeradas.

V24 expõe
`check<'a, T: HasSemanticObservations<'a>>(file: &T, level: ViolationLevel)
-> Vec<Violation<'a>>` e mapeia, em ordem de entrada, somente
`NeutralProjectionDestination`. Cada ocorrência, inclusive duplicata, gera exatamente
uma violação. `rule_id = "V24"`; nível preserva o parâmetro; mensagem contém
`contract_id` e `detail` verbatim; location usa `file.path()` e linha/coluna da
observação. `language()` não participa do classificador porque L3 já decidiu
aplicabilidade.

O módulo é importável por `crystalline_lint::rules::semantic_field_loss`; tipos por
`crystalline_lint::entities::rule_traits`, `layer` e `violation`. Este L0 não transforma
o gate puro de L1 em evidência para a extração L3; dependência, ausência e contratos
cruzados permanecem obrigatórios no lote de integração.
