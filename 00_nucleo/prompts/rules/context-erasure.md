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
