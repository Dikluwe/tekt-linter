# Assessment 0004 — inventário e registro de crates

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvos:** `crate_registry` e `provenance_inventory`

## Hipótese

As duas unidades apenas normalizam, agregam e consultam dados finitos. Esperamos zero
achados. Um RED reclassifica registries e inventários que parecem mecânicos, sobretudo
quando recebem entradas em ordem variável do filesystem ou do pipeline paralelo.

## Alegações sob teste

1. Permutar membros ou arquivos não altera nenhuma resposta nem os bytes observáveis
   dos diagnósticos produzidos.
2. `owner_of` escolhe sempre o ancestral de maior profundidade; empates e membros
   duplicados são rejeitados ou resolvidos de forma canônica, nunca pela ordem de input.
3. Normalização `-` → `_` é aplicada de forma consistente a pacote, dependências e
   renames; colisões após normalização não são silenciosamente reinterpretadas.
4. O inventário agrega exatamente constantes elegíveis por módulo, exclui testes,
   tabelas, triviais e linguagens não Rust, e escolhe location canônica.

## Gate curto

Até quatro propriedades, sem alterar produção. Comportamento não decidido pelos prompts
é `SPEC-GAP`. Falha dependente apenas da ordem de uma coleção logicamente equivalente é
RED, pois ambas as unidades alimentam saída determinística do linter.
