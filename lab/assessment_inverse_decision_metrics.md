# Assessment P0114 — contrapesos experimentais de V19/V20

> **Estado:** LAB CLOSED — PROMOTE ONE SPEC
> **Commit-base:** `3fd957745d1b3dec264d2924c0b35a4804f6e4c2`
> **Data:** 2026-08-26
> **Decisão histórica:** promover V27 `MergeableDecisionArms` para um passo próprio de
> especificação; a hipótese então chamada V28 foi posteriormente descartada e o ID está
> vago

> **Superado parcialmente por P0115:** o refino taxonômico posterior encontrou dois
> falsos positivos no fingerprint, separou tabelas declarativas e terminou
> `KEEP V27 EXPERIMENTAL`. A decisão vigente está em
> `lab/assessment_v27_taxonomy.md`.

> **Fechamento V28 posterior:** V20 é uma métrica unidirecional de teto. Complexidade
> mínima não é objetivo de qualidade; especificidade e segurança de tipos são dimensões
> diferentes. `FragmentedPatternProjection` não é o inverso semântico de V20, seu probe e
> fixtures foram removidos e `V28 = VACANT`. As seções abaixo são somente histórico.

## 1. Baseline e colisão de identidade

O congelamento encontrou V19=68 e V20=17 no auto-lint. Hashes L0/L1:

| Insumo | SHA-256 |
|---|---|
| `or_pattern_alternatives.rs` | `2579227c98b7b4274a08b65844a8f66f4166d650bb320f9972cf14929ccebb99` |
| `deep_pattern_nesting.rs` | `9b32c9892f59ab319495868cd0968c63f90ab147d343853886827f127db923aa` |
| prompt V19 | `91c409539ea603c2e4ae1aa4932e6bedddb209991652b4a345dbbe7e3b159620` |
| prompt V20 | `b4a4acabc362920561bec12f95ddcb99a0fcf68c5d53759b9ff3e6ab1d5060e5` |
| ADR-0016 | `abdf38e2c75b7f3a113a50db61913320d08f510e5c7f377efde020af1c19ffd2` |
| `Cargo.lock` | `91b07d6f70b8d00ef216a6fdc3d8db24d3e8977539055430317ff593b6fa02cb` |

O nome V26 proposto inicialmente colidia com a regra de produção `NucleusIntegrity`,
registrada em `04_wiring/main.rs` e na CLI. O README principal não a mostrava na tabela
consultada. O lab corrigiu os aliases antes da medição: V27 é o contraponto de V19 e V28
é o contraponto de V20. Nenhum dos dois foi registrado em produção.

Ferramentas: `rustc 1.92.0`, `cargo 1.92.0`, `tree-sitter 0.23` e
`tree-sitter-rust 0.23` provenientes do `Cargo.lock` vigente. O probe foi compilado
diretamente como crate isolado, ligando somente os artefatos já construídos; nenhum
módulo de L1–L4 o importou.

## 2. Artefatos e reprodução

| Artefato | SHA-256 inicial executado |
|---|---|
| `lab/inverse_decision_metrics_probe.rs.txt` | `ffd5277ed690c7894f317dbbc92d3b2969e6bec796b3f316e7f0bfc6ad1786f3` |
| `lab/inverse_decision_metrics_fixtures.rs.txt` | `00992cfe6c300778c7bea2c458e8a1851cf1bf189f1aacb7690154907c538516` |

Comando de reprodução do probe:

```bash
rustc --crate-name p0114_inverse_probe --edition=2021 \
  lab/inverse_decision_metrics_probe.rs.txt \
  -L dependency=target/debug/deps \
  --extern tree_sitter=target/debug/deps/libtree_sitter-3f225a9764d2bd9d.rlib \
  --extern tree_sitter_rust=target/debug/deps/libtree_sitter_rust-541d59d46810700f.rlib \
  -o /tmp/p0114-inverse-probe
```

Os hashes acima identificam a versão confrontada antes da redação final deste assessment;
o fechamento deve registrar também os hashes finais após qualquer ajuste documental.

## 3. Normalização executada

V27 construiu fingerprint recursivo por `node kind` e folhas textuais. Ignorou whitespace,
comentários e `parenthesized_expression` transparentes, mas preservou paths, callees,
literais, macros, controle e identificadores. Agrupou somente braços da mesma
`match_expression`, sem guard.

`PROVEN-SYNTACTIC` foi restrito a padrões unitários/literais sem bindings, wildcard,
desestruturação, range, `@` ou `or-pattern` prévio. Corpo vazio ou macro e padrões fora
desse subconjunto foram `UNKNOWN`. Essa restrição eliminou alegações indevidas de
subsunção como `(Rust, ItemDefinition)` versus `(_, ItemDefinition)`.

V28 aceitou somente cadeias lineares de `if let`, sem `else`, cujo bloco de sucesso
continha exatamente o próximo `if let`. O RHS do nível seguinte precisava ser exatamente
o único binding produzido pelo padrão anterior. Chamadas como
`name_node.utf8_text(source)` foram bloqueadas: são pipelines, não padrões componíveis.

## 4. Matriz adversarial

### V27 — largura fragmentada

| Fixture | Esperado | Obtido |
|---|---|---|
| dois braços unitários, literal igual | positivo | `PROVEN-SYNTACTIC` |
| três braços, chamada/argumentos iguais | positivo | `PROVEN-SYNTACTIC` |
| layout/comentário diferente, AST igual | positivo | `PROVEN-SYNTACTIC` |
| argumento diferente | silêncio | silêncio |
| guard diferente | silêncio | silêncio |
| binding/resultados diferentes | silêncio | silêncio |
| corpos vazios iguais | incerto | `UNKNOWN` |
| macros iguais | incerto | `UNKNOWN` |

Resultado: 3 positivos fortes, 2 `UNKNOWN`, zero falso positivo nos negativos. O probe
é deliberadamente incompleto para bindings e padrões sobrepostos; não alegou ausência de
falsos negativos fora do subconjunto fechado.

### V28 — profundidade fragmentada

| Fixture | Esperado | Obtido |
|---|---|---|
| cadeia pura de dois níveis | positivo | `PROVEN-SYNTACTIC` |
| cadeia pura de três níveis | positivo | `PROVEN-SYNTACTIC` |
| efeito entre níveis | silêncio | silêncio |
| reutilização intermediária | silêncio | silêncio |
| `else` com ação distinta | silêncio | silêncio |
| nesting lexical sem cadeia de projeção | silêncio | silêncio |

Resultado: 2 positivos fortes e zero emissão nos negativos. A prova é sintática e cobre
somente `if let` diretamente componível; `match`, `let else`, ownership e drop order
continuam fora do oráculo.

As fixtures completas compilaram como crate Rust. O probe rejeitou entrada com erro de
parse em vez de produzir candidato parcial.

## 5. Confronto por transformação

Uma cópia temporária combinou os dois positivos. Antes:

```rust
match value { Kind::A => 7, Kind::B => 7, Kind::C => 0 }

if let Some(middle) = value {
    if let Middle::Leaf(number) = middle { result = number; }
}
```

Depois:

```rust
match value { Kind::A | Kind::B => 7, Kind::C => 0 }

match value {
    Some(Middle::Leaf(number)) => result = number,
    _ => {}
}
```

Ambas as versões compilaram. A matriz de resultados foi idêntica:

```text
[7, 7, 0, 9, 0, 0]
```

O linter de produção, executado somente com V19/V20 no projeto temporário, confirmou:

| Versão | V19 | V20 |
|---|---:|---:|
| antes | 0 | 0 |
| depois | 1 | 1 |

Logo, os contrapesos não cancelam as métricas: a composição indicada por V27/V28 move a
representação para o lado observado por V19/V20. A redução foi de três linhas no pequeno
programa, mas linhas não participaram do veredito.

## 6. Piloto no código real

Universo: todos os `.rs` versionáveis em `01_core`, `02_shell`, `03_infra`, `tests`,
`lab` e `oraculo`. Tempo observado: 0,96 s; RSS máximo: 8.144 KiB.

### V27

Foram encontrados 13 grupos: 5 `PROVEN-SYNTACTIC` e 8 `UNKNOWN`. Revisão integral:

| Path/owner | Classe | Revisão |
|---|---|---|
| `02_shell/cli.rs::sarif_level` | forte | Fatal/Error retornam `"error"`; fundível |
| `01_core/rules/wiring_logic_leak.rs::is_forbidden` | forte | quatro variantes `true`; fundíveis |
| mesmo owner | forte | Struct/Class compartilham expressão configurável; fundíveis |
| `01_core/entities/l1_allowed_external.rs::for_language` | forte | Rust/Unknown retornam a mesma referência; fundíveis, embora o comentário de fallback deva ser preservado |
| `01_core/entities/rule_traits.rs::decision_arm_term_for` | forte | TypeScript/Go retornam o mesmo termo; fundíveis |
| oito grupos restantes | `UNKNOWN` | bindings, wildcard, tuplas, literais textuais, corpo vazio ou macro; sem alegação automática |

Os cinco fortes são verdadeiros positivos sintáticos em quatro arquivos e quatro módulos
funcionais. Nenhuma transformação foi aplicada. O caso com comentário de fallback mostra
que futura mensagem/autofix deve preservar evidência; por isso a promoção é apenas da
especificação e deve nascer sem autofix.

### V28

O detector inicial permissivo encontrou 11 pipelines aninhados, mas o confronto mostrou
que nenhum era uma projeção componível: o segundo RHS chamava método sobre o binding.
Depois de congelado esse falso pressuposto, o detector foi endurecido e produziu zero
candidato real. Essa redução 11→0 é calibração do gate, não saneamento de produção.

A ausência de sinal real impede promoção mesmo com fixtures verdes. Cobrir formas úteis
exigiria IR entre expressões, efeitos e ownership superior ao `HasDecisionArms` atual.

## 7. Decisão

### V27 — `PROMOTE-SPEC`

Há sinal real suficiente, implementação sintática pequena e subconjunto seguro
falsificável. O passo posterior deve:

1. reservar V27 somente depois de conferir novamente o registro;
2. especificar equivalência estrutural, compatibilidade de bindings e sobreposição;
3. emitir `info`, sem autofix;
4. manter `UNKNOWN` fora da contagem principal;
5. preservar comentários/evidência e não sugerir reordenação sem prova;
6. testar explicitamente a transição esperada para V19.

### V28 — decisão histórica superada

O experimento demonstrou apenas uma transformação sintática possível, não uma métrica de
qualidade inversa. A hipótese foi descartada posteriormente; não deve ser reaberta como
contrapeso de V20. O ID V28 está vago para outra regra sem relação causal com esta ideia.

Estado final: **LAB CLOSED — PROMOTE ONE SPEC**. Nenhum score foi criado, V19/V20 não
foram alteradas e nenhuma regra experimental entrou no linter.
