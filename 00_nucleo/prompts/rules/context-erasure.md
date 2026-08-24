# Prompt — V23 `ContextErasure`
Hash do Código: 5087e2a9

**Status:** VIGENTE; ADR-0018 aceito em 2026-08-23
**Camada futura:** L1, com fatos AST produzidos por L3
**Idioma inicial:** Rust; contrato e IR agnósticos de linguagem

## Intenção

Detectar quando um valor declarado como contextual chega a um sumidouro depois que o
contexto requerido foi substituído por neutro ou removido por uma projeção parcial.

V23 só opera quando existe `[[semantic.context]]` aplicável. Nomes, literais e métodos
não têm significado implícito.

## Contrato

Cada entrada declara `id`, `language`, `scopes`, `sources`, `resolvers` com posição do
argumento contextual, `erasing_projections`, `sinks` e `absolute_sources` auditáveis.
O resolvedor é violação apenas quando recebe neutro e a origem não é absolute-only.
Uma projeção apagadora é violação apenas se seu resultado alcançar um sumidouro do
mesmo contrato pelo fluxo local suportado.

O fluxo suportado inclui expressão direta e cadeia por `let` imutável dentro da mesma
função. Mutação, retorno interprocedural, macro opaca ou alias não resolvido resultam em
caso não analisável, nunca em acusação presumida.

## Diagnóstico

`V23 ContextErasure`: informar contrato, fonte, operação apagadora, sumidouro e linha.
Nível padrão `warning`.

## Cenários

```text
Dado contrato que marca contextual_radius como contextual, resolve_pt arg 0 como
contexto e rounded_rect_path como sumidouro
Quando contextual_radius.resolve_pt(0.0) alimenta rounded_rect_path
Então emitir V23

Dado o mesmo contrato e abs como projeção apagadora
Quando contextual_radius.abs.0 alimenta o sumidouro
Então emitir V23

Dado absolute_radius em absolute_sources
Quando absolute_radius.resolve_pt(0.0) alimenta o sumidouro
Então não emitir

Dado tracking.resolve_pt(style.size.val())
Quando style.size é contexto declarado
Então não emitir

Dado let zero = 0.0 fora de resolvedor registrado
Então não emitir
```

## Restrições

- Proibido codificar `Length`, `abs`, `resolve_pt`, raio ou paths do Typst no núcleo.
- Proibido interpretar todo zero como contexto neutro fora da posição registrada.
- Seleção `v23` não executa V24/V25.
- Configuração inválida falha explicitamente antes da análise.

## Fronteira executável L3 → L1

Neutralidade, correspondência de contrato, fluxo ao sink, `absolute-only` e opacidade são
decididos pelo extrator L3. O classificador L1 não recebe AST nem contrato e não pode
recomputar essas decisões. Ele recebe uma `SemanticObservation` já classificada:

```rust
pub enum SemanticObservationKind {
    ContextNeutralArgument,
    ContextErasingProjection,
    NeutralProjectionDestination,
    DuplicateDecisionOwner,
    DecisionProxyReentry,
    CanonicalizerReentry,
    DirectDecisionReimplementation,
}
pub struct SemanticObservation {
    pub contract_id: String,
    pub kind: SemanticObservationKind,
    pub detail: String,
    pub line: usize,
    pub column: usize,
}
pub trait HasSemanticObservations<'a> {
    fn semantic_observations(&self) -> &[SemanticObservation];
    fn path(&self) -> &'a std::path::Path;
    fn language(&self) -> &Language;
}
```

V23 expõe
`check<'a, T: HasSemanticObservations<'a>>(file: &T, level: ViolationLevel)
-> Vec<Violation<'a>>` e mapeia, em ordem de entrada, somente
`ContextNeutralArgument` e `ContextErasingProjection`. Cada ocorrência gera uma
violação; duplicatas não são deduplicadas em L1. `rule_id = "V23"`, o nível é exatamente
o parâmetro recebido, a mensagem contém `contract_id` e `detail` verbatim, e a location
usa `file.path()` mais linha/coluna da observação. `language()` é metadado irrelevante
nesta camada: aplicabilidade linguística já foi decidida por L3.

As regras são importáveis por `crystalline_lint::rules::context_erasure`; tipos por
`crystalline_lint::entities::rule_traits`, `layer` e `violation`. A auditoria da decisão
L3 (formas neutras, fluxo, identidade e cardinalidade de extração) é obrigatória em lote
separado e não pode ser alegada a partir do gate puro de L1.
