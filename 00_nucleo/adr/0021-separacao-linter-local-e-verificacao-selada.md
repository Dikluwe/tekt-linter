# ADR-0021 — separar o linter local da verificação selada

**Estado:** aceito  
**Data:** 2026-08-25  
**Decisor:** mantenedor humano  
**Contexto:** P0100–P0102 / F05

## Contexto

A auditoria de `refine-revisions` ampliou incrementalmente o modelo de ameaça até exigir
resistência a executável Git hostil, troca transitória sincronizada do object database,
descendente que executa `setsid` e contenção por Job Object. A soma dessas garantias
transformaria o linter arquitetural em sandbox/certificador multiplataforma.

Os gates provaram que a implementação vigente não oferece essa certificação. Também
provaram que fechá-la exige decisões próprias de produto: cgroup/namespace/subreaper,
staging/sandbox/backend Git, privilégios, budgets, cleanup e matriz por plataforma.

## Decisão

Adotar a opção C:

1. `crystalline-lint` continua sendo um linter para melhoria e preservação arquitetural.
2. `refine-revisions` permanece conveniência local defensiva.
3. O Git instalado, o usuário local e a estabilidade do repositório durante a operação
   pertencem à base confiável desse comando.
4. Configuração externa, hooks, alternates, protocolos externos, paths, framing,
   budgets e symlinks persistentes continuam tratados defensivamente e falham fechados.
5. Adversário local ativo, Git adulterado, corrida transitória sincronizada e fuga
   deliberada de processo ficam fora da garantia do modo local.
6. Um modo selado/certificador poderá ser criado futuramente como projeto ou comando
   separado, somente diante de caso de uso real. Ele terá L0, sandbox, gates por
   plataforma, distribuição e release próprios.

## Consequências

Os avanços P0100/P0101 são integráveis: seam Git única, ambiente mínimo, validação de
framing, budgets incrementais, timeout, proteção persistente do object database e
projeção única para o extrator.

Os gates P0102 contra adversário local ativo são preservados como pesquisa futura e
ignorados na suíte padrão. `PASS` do linter não deve ser descrito como certificação
contra esse adversário.

F05 fecha para o modelo local. R2/R4/R5 deixam de ser REDs do produto atual e passam a
requisitos candidatos do futuro modo selado. Isso é reclassificação de autoridade pelo
decisor humano, não alegação de que a implementação satisfez os gates hostis.

## Alternativas rejeitadas

- incorporar agora sandbox multiplataforma: custo e autoridade desproporcionais ao
  propósito atual;
- remover `refine-revisions`: descartaria uma conveniência útil e melhorias já verdes;
- manter contrato ambíguo: faria limites locais parecerem garantias formais.
