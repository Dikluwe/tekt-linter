# P0080 / Assessment 0011 — revisão adversarial independente (Agente C)

## Veredito

**PASS funcional; SPEC-GAP documental no gate cego.** Não encontrei RED em V12 ou V13: o probe independente passou em 6/6 propriedades. Contudo, a alegação 5 do assessment exige cobertura dos “18 tokens” sem enumerá-los nem apontar normativamente para a lista do prompt causal. Um gate escrito apenas a partir do assessment consegue verificar a cardinalidade, mas não distinguir a tabela correta de outra tabela qualquer com 18 entradas.

O prompt causal enumera os 18 tokens e a produção coincide com ele; usando essa fonte, a implementação passou integralmente. O SPEC-GAP não é defeito de execução e não justifica alterar L1 durante esta triagem.

Escopo lido: P0080 e assessment 0011 integrais, prompts causais V12/V13, produção dos dois alvos e entidades/traits necessárias. Não foi lido `tests/declaration_state_classifiers_assessment.rs`, nem mensagens ou artefatos do Agente B. Produção e gate não foram alterados.

## Propriedades e mutações

| # | Prioridade | Ataque | Observação mecânica | Resultado |
|---|---:|---|---|---|
| P1 | P1 | V12: sete camadas × seis `DeclarationKind` × dois estados de `allow_adapter_structs`. | Fora de L4 tudo é vazio. Em L4, Enum/Impl/Interface/TypeAlias sempre violam; Struct/Class seguem somente a configuração. | **PASS** — alegação 1. |
| P2 | P1 | V12: todas as kinds, duplicata Enum, nomes NFC/NFD, linhas fora de ordem; executar config false/true e comparar input. | Config remove apenas Struct/Class; input não muda; ordem, duplicata, kind/nome, source path, linha, rule id e `Warning` permanecem. | **PASS** — alegações 2 e 3. |
| P3 | P1 | V13: sete camadas × `is_mut`, usando tipo inofensivo `u32`. | Somente `L1 && is_mut` viola. | **PASS** — alegação 4 para mutabilidade direta. |
| P4 | P0 | V13: tabela dos 18 tokens enumerados no prompt/produção. | Exatamente 18 `Error`, na ordem de entrada; cada mensagem contém nome/token e cada linha sentinela é preservada. | **PASS** funcional; **SPEC-GAP** da fonte cega da tabela. |
| P5 | P1 | V13: `mutex`, `RWLock`, `Once_Lock`, `AtomicU128`, `AtomicPointer`, `ReferenceCell`, `Unsafe_Cell`, `Μutex` e outros próximos sem substring causal exata. | Todos permanecem isentos. | **PASS** — alegação 6. |
| P6 | P1 | V13: `is_mut` junto de tokens, múltiplos tokens em ordem textual contrária à tabela, duplicata, Unicode e duas execuções. | `mut` tem precedência; sem mut vence o primeiro token da lista causal (`Mutex` antes de `RwLock`, `RefCell` antes de `UnsafeCell`); vetor completo é determinístico e preserva ordem/multiplicidade/evidência. | **PASS** — alegações 5 e 6. |

## Tabela causal efetivamente verificada

Derivada do prompt `mutable-state-core.md` e confirmada na constante de produção:

```text
Mutex, RwLock, OnceLock, LazyLock,
AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize,
AtomicU8, AtomicU16, AtomicU32, AtomicU64, AtomicUsize, AtomicPtr,
RefCell, UnsafeCell
```

São 18 entradas. A comparação observada é `type_text.contains(token)`, byte-sensitive e sem normalização.

## Natureza do SPEC-GAP

O assessment diz “tokens públicos congelados” e “os 18 tokens”, mas não contém a lista nem uma referência explícita do tipo “a lista normativa é a seção X de `mutable-state-core.md`”. Sob segregação estrita, um autor de gate que não lê produção precisa adivinhar ou importar contexto não declarado.

Critério documental de fechamento: enumerar a lista no assessment ou referenciar explicitamente a seção normativa do prompt causal. Isso torna a propriedade falsificável sem acoplá-la à produção sob teste.

## Reprodução

Probe preservado fora do auto-lint: `lab/assessment_declaration_state_classifiers_probe.rs.txt`.

```sh
cargo build --lib
cp lab/assessment_declaration_state_classifiers_probe.rs.txt \
  /tmp/assessment_declaration_state_classifiers_probe.rs
rustc --edition=2021 /tmp/assessment_declaration_state_classifiers_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/assessment_declaration_state_classifiers_probe
/tmp/assessment_declaration_state_classifiers_probe
```

Saída observada:

```text
PASS P1 V12 exhaustive 7x6x2
PASS P2 V12 config isolation/order/multiplicity/evidence/no mutation
PASS P3 V13 exhaustive layers x is_mut
PASS P4 V13 all 18 causal tokens
PASS P5 V13 legitimate near-substrings
PASS P6 V13 precedence/order/multiplicity/evidence/determinism
```

## Matriz priorizada

| Ordem | Ataque | Falha procurada | Estado |
|---:|---|---|---|
| 1 | Conferir cada token causal individualmente | Token omitido/substituído | PASS funcional / SPEC-GAP documental |
| 2 | V12 7×6×2 | Kind/camada/config assimétrico | PASS |
| 3 | `mut` + múltiplos tokens | Precedência instável | PASS |
| 4 | Duplicatas e ordem sentinela | Agregação ou reordenação | PASS |
| 5 | Substrings visualmente próximas | Falso positivo por representação | PASS |
| 6 | Alternar config com mesmo input | Efeito fora de Struct/Class ou mutação | PASS |
