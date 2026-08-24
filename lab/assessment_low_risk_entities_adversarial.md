# Triagem adversarial — entidades puras de baixo risco

## Escopo e forma de comparação

Quatro propriedades de alto sinal para `Layer`, `Language`, `ViolationLevel`,
`Location`, `Violation`, `LocalIndex` e `ProjectIndex`. Nenhuma depende de parser,
filesystem ou ordem de iteração de `HashSet`.

Para comparar índices, usar uma projeção observável completa:

- `referenced_prompts`, `all_declared_traits`, `all_implemented_traits` e
  `all_blanket_impl_traits` comparados como conjuntos;
- `alien_files` comparado conforme a alegação congelada de merge comutativo e
  duplicatas neutras: conjunto canônico de paths, e também registrar o `Vec` bruto para
  localizar ordem ou multiplicidade introduzida.

Se a intenção real for preservar multiplicidade ou ordem de aliens, há `SPEC-GAP`: a
alegação 5 de comutatividade integral e a alegação 6 de neutralidade de duplicatas
precisam ser restringidas antes de chamar o resultado de `PASS`.

## P1 — lei de monóide sob todos os campos e particionamentos

Construir três contribuições A, B e C, cada uma com valores sentinela distintos em
**todos** os campos: prompt, alien, trait declarada, implementada e blanket. Reduzir:

```text
((A merge B) merge C)
(A merge (B merge C))
((C merge A) merge B)
fold([A, B, C])
fold([A]) merge fold([B, C])
empty merge fold([A, B, C])
fold([A, B, C]) merge empty
```

Repetir as permutações dos três elementos. Exercitar tanto `merge_local` quanto
`ProjectIndex::merge`, para impedir que uma das duas rotas omita um campo.

Mutação que a propriedade deve matar: remover, sobrescrever ou direcionar ao campo
errado qualquer um dos cinco `extend`/`insert`/`push`; inverter somente a ordem de
concatenação de aliens.

**RED exato:** qualquer projeção final perde ou inventa um sentinela; identidade vazia
altera qualquer campo; ou duas parentizações, partições ou permutações produzem
projeções semanticamente diferentes. Se os quatro sets coincidirem mas o `Vec` bruto
de aliens variar apenas em ordem, isso já contradiz comutatividade estrutural; classificar
como `RED` sob o assessment congelado, ou `SPEC-GAP` se a igualdade pretendida for
explicitamente redefinida como conjunto.

## P2 — duplicatas são idempotentes em cada família, inclusive aliens

Criar um `LocalIndex` D com o mesmo prompt, o mesmo alien e o mesmo nome repetido duas
vezes dentro de cada vetor de traits. Comparar:

```text
reduce([D])
reduce([D, D])
reduce([D, empty, D])
reduce([split(D), split(D)])
```

`split(D)` distribui os cinco campos por contribuições diferentes, mantendo os mesmos
valores. Os quatro campos `HashSet` devem possuir cardinalidade 1. Sob a alegação 6,
aliens também devem conter exatamente um path semanticamente observável.

Mutação que a propriedade deve matar: trocar qualquer set por vetor, omitir dedup de
alien, ou deduplicar uma família usando acidentalmente a chave de outra.

**RED exato:** `reduce([D, D])` difere de `reduce([D])` em qualquer campo, qualquer
cardinalidade de set é diferente de 1, ou o mesmo alien aparece duas vezes no resultado
observável. A implementação atual usa `Vec::push/extend` para aliens; portanto a
multiplicação de um path duplicado é o candidato RED mais direto.

## P3 — transporte e fusão possuem cobertura de campo completa

Construir um `ParsedFile` sentinela com header `prompt-P`, layer `Unknown`, path
`alien-P`, e três listas disjuntas:

```text
declared_traits       = ["declared-only"]
implemented_traits    = ["implemented-only"]
blanket_impl_traits   = ["blanket-only"]
```

Aplicar `LocalIndex::from_parsed`, depois `merge_local`, e exigir o mapeamento exato de
cada origem ao seu único destino. Repetir com layer conhecida, que deve mudar somente
`alien_file` de `Some(path)` para `None`; todos os demais sentinelas permanecem.
Confirmar ainda que `from_parse_error`, `from_source_error` e `LocalIndex::empty` são
identidades em todos os cinco campos.

Mutação que a propriedade deve matar: esquecer `blanket_impl_traits` em `empty`,
`from_parsed`, `merge_local` ou `merge`; cruzar declared/implemented/blanket; perder o
prompt; classificar layer conhecida como alien; deixar construtores de erro contribuírem.

**RED exato:** após a cadeia, qualquer sentinela está ausente, aparece em mais de uma
família, aparece na família errada, ou um campo não sentinela é inventado. Na repetição
com layer conhecida, qualquer diferença além da ausência de `alien-P` também é RED.

## P4 — variantes públicas, ordem de severidade e clone integral

Manter listas explícitas congeladas e exigir distinção par a par:

```text
Layer    = L0, L1, L2, L3, L4, Lab, Unknown
Language = Rust, TypeScript, Python, C, Cpp, Zig, Go, Java, Elixir, Unknown
Level    = Info, Warning, Error, Fatal
```

Para `Level`, exigir exatamente `Info < Warning < Error < Fatal` e nenhuma igualdade
entre variantes. Uma função auxiliar com `match` exaustivo, sem wildcard, transforma
adição futura de variante pública em falha de compilação até a propriedade ser revista.

Construir duas violações que diferem em exatamente um campo por vez (`rule_id`, level,
message, path, line, column) e exigir desigualdade. Para clone, testar paths
`Cow::Borrowed` e `Cow::Owned`; o clone deve ser igual ao original e preservar a
variante do `Cow`, não apenas os bytes do path.

Mutação que a propriedade deve matar: omitir `Go`, `Java`, `Elixir`, `L0` ou `Info` da
matriz pública; reordenar severidades; igualdade parcial de violação/location; converter
path borrowed em owned (ou o inverso) durante clone.

**RED exato:** variante listada não é construtível ou colide com outra; a cadeia de
ordem falha; duas violações com um campo divergente comparam iguais; clone diverge em
qualquer campo; ou o discriminante `Borrowed`/`Owned` muda. Variante pública nova causa
falha de compilação do `match` e deve ser classificada como necessidade de atualizar a
especificação, nunca aceita silenciosamente como cobertura completa.

## Prioridade de execução

1. **P2 duplicatas/aliens** — maior probabilidade de RED imediato e observação mínima.
2. **P1 monóide completo** — valida a segurança da redução paralela em todos os campos.
3. **P3 transporte completo** — protege especialmente a família blanket, ausente no
   prompt histórico mas presente no alvo público atual.
4. **P4 variantes/clone** — fecha completude pública e semântica das entidades simples.
