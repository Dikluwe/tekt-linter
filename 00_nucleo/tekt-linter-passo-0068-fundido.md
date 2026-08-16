# Passo 0068 (tekt-linter) — Estender predicado V21 (campos profundos) + materializar ADR V16/V21

**Repositório**: `tekt-linter`.
**Fusão de dois passos relacionados**, ambos originados na mesma investigação
(P1053→P1055, `typst-crystalline`), ambos a tocar `wildcard_saturation.rs`/regras
V16-V21:
1. Achado do P1055, Fase 0: `layouter.regions.current.cursor_y += style.size * 0.6`
   escapou ao predicado de V21 por ter 3 níveis de campo no alvo da atribuição composta.
2. ADR "V16 não é silenciável por citação; V21 é" — guardado no projecto do linter,
   precisa de ser reflectido na ferramenta, não só existir como documento.
**Pré-condição**: `git status` limpo no `tekt-linter`.

---

## Parte 1 — Estender o predicado de V21 para atribuições compostas com campos profundos

### Diagnóstico (já confirmado no P1055)

O parser reconhece `variavel = literal × contextual` mas não
`objeto.campo.subcampo += literal × contextual` — a profundidade de acesso a campo no
alvo da atribuição (`layouter.regions.current.cursor_y`, 3 níveis) não era reconhecida
como sumidouro geométrico válido.

### Correcção

1. Alargar o reconhecimento de "sumidouro" para aceitar qualquer cadeia de acesso a
   campo (`a.b.c.d`) terminada num nome que já bate na lista de sumidouros geométricos
   (`cursor_y`, `cursor_x`, `gap`, `inset`, etc. — lista já definida no Passo 0066),
   não só identificadores simples.
2. Cobrir explicitamente operadores de atribuição composta (`+=`, `-=`, `*=`, `/=`),
   não só `=`.
3. Testar contra o caso real já conhecido:
   ```
   layouter.regions.current.cursor_y += layouter.style.size * 0.6;
   ```
   Confirmar que a regra estendida apanha isto antes de dar como resolvido.

---

## Parte 2 — Referência ao ADR no código-fonte da regra V16

`wildcard_saturation.rs` não deve depender de ninguém ter lido o ADR à parte.

1. Adicionar ao topo do ficheiro um comentário/rustdoc citando o ADR pelo caminho exacto
   (`00_nucleo/adr/<nome-real>.md`), com a frase central: *"V16 nunca silencia por
   citação — um wildcard vigia todas as variantes futuras de um enum, não um valor
   fixo. Ver ADR para a distinção com V21."*
2. Remover qualquer comentário existente que sugira o oposto (ex.: intenção de suportar
   silenciamento por citação como V21) — o ADR fecha essa dúvida.

## Parte 3 — Validação de formato da taxonomia (`N16[α/β/γ]`)

**Não** exigir a tag ainda — a reclassificação dos 204 casos existentes é passo próprio,
no `typst-crystalline`, fora deste. Mas, quando a tag **estiver presente**, validar
formato:

1. Reconhecer `N16[α]`, `N16[β]`, `N16[γ]` (confirmar com o dono se prefere letras
   gregas literais ou `N16[A]`/`N16[B]`/`N16[C]` por segurança de encoding).
2. Tag malformada (`N16[` seguido de algo fora das três válidas): aviso próprio de
   formato, não erro, não silenciamento.
3. Não muda a severidade base de V16 — só valida forma, quando a tag existir.

## Parte 4 — Métrica agregada de distribuição (adiada)

Proposta: relatório por módulo com a proporção `α/β/γ`, mesmo padrão de V22. **Adiada**
até existirem dados reais (a reclassificação dos 204 casos ainda não aconteceu) —
implementar métrica sobre dados inexistentes é especulativo.

---

## Validação

```
cargo test --workspace
crystalline-lint --checks v21 <repo-alvo>   # confirmar que divider.rs (P1055) é apanhado
crystalline-lint --checks v16 <repo-alvo>   # confirmar que a severidade de V16 não mudou
```

---

## Resultado esperado

V21 apanha atribuições compostas com campos profundos (fecha a lacuna do P1055). V16
com a razão do "nunca silencia" documentada no próprio código-fonte da regra, não só no
ADR à parte. Taxonomia `N16[α/β/γ]` reconhecida e validada em formato, pronta para a
reclassificação dos 204 casos no `typst-crystalline`. Métrica agregada fica para depois.
