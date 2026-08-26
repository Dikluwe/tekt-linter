# Passo operacional 0116 — implementar e validar V27 MergeableDecisionArms

> **Estado:** EXECUTADO — fechado por `lab/assessment_v27_implementation.md`
> **Predecessor:** P0115, fechado por `lab/assessment_v27_taxonomy.md`
> **Regra:** V27 `MergeableDecisionArms`
> **Categoria:** redução de duplicação sintática em decisões
> **Escopo inicial:** Rust `match`; diagnóstico `Info`; sem autofix de produção
> **V28:** permanece `VACANT`
> **Resultado:** `LAB CLOSED — V27 DETECTOR ONLY`; V27 registrada como `Info` opt-in,
> sem autofix e com bindings preservados como `Unknown` por ausência de autoridade de tipos

## 1. Decisão e pergunta fechada

V27 deixa de ser tratada como “contrapeso” de V19. Ela materializa uma regra clássica:

> braços de uma mesma decisão com consequência estruturalmente idêntica podem indicar
> erro de copiar/colar ou padrões que deveriam ser unidos por `|`.

P0116 responde somente:

> O tekt-linter consegue detectar braços consolidáveis com precisão suficiente e emitir
> sugestão determinística, sem alterar bindings, guards, ordem, expansão de macro ou
> configuração condicional?

O passo implementa o diagnóstico V27 e testa uma transformação em laboratório. Ele não
adiciona correção de código-fonte ao `--fix-hashes`, não cria um `--fix` genérico e não
altera V19/V20.

## 2. Contrato normativo

Para dois braços `a` e `b` da mesma expressão, definir:

```text
mergeable(a, b) =
    body_equivalent(a, b)
    AND binding_compatible(a, b)
    AND guard_equivalent(a, b)
    AND order_safe(a, b)
    AND expansion_compatible(a, b)
    AND attributes_compatible(a, b)
    AND NOT placeholder_body(a)
```

V27 reporta um grupo maximal de dois ou mais braços quando todos podem ser consolidados
no primeiro braço do grupo.

O diagnóstico deve oferecer duas interpretações, sem decidir intenção:

1. possível erro de copiar/colar no corpo;
2. repetição intencional representável por um `or-pattern`.

## 3. Igualdade observável do corpo

Não comparar texto bruto nem hash sozinho. Implementar duas etapas:

1. fingerprint estrutural para agrupar candidatos;
2. comparação estrutural integral para confirmar o grupo.

A representação canônica deve:

- preservar todos os nós nomeados e tokens operadores, inclusive `==`, `!=`, `?`, `!`,
  `return`, `break`, `continue`, `await` e delimitadores semanticamente relevantes;
- ignorar whitespace, comentários e coordenadas;
- preservar caminhos, literais, argumentos, tipos, labels e ordem dos filhos;
- distinguir chamada comum de invocação de macro;
- normalizar somente bindings cuja correspondência tenha sido provada pela seção 4;
- nunca usar `BodyForm` ou `body_snippet` isoladamente como prova.

Colisão de fingerprint não é achado: o comparador integral precisa confirmar igualdade.

## 4. Compatibilidade de bindings

Estender a IR de decisões com uma assinatura por binding:

```text
BindingSignature {
    canonical_slot,
    source_name,
    mode: move | ref | ref_mut,
    mutable,
    pattern_path,
    uses_in_guard,
    uses_in_body
}
```

Duas alternativas são compatíveis somente quando introduzem a mesma quantidade de
bindings e existe bijeção por `canonical_slot` com mesmo modo, mutabilidade e posições de
uso equivalentes.

P0116 não possui resolução de tipos do compilador. Logo:

- padrões sem binding podem ser provados sintaticamente;
- bindings textualmente correspondentes podem ser diagnosticados como candidatos;
- renomeação, inferência de tipo ou ergonomia de binding que dependa de `rustc` fica
  `UNPROVEN` e não recebe sugestão automática;
- nenhum erro de binding pode ser delegado à compilação posterior como substituto da
  prova do linter.

Testar explicitamente `move`, `mut`, `ref`, `ref mut`, nested references e diferenças da
edição 2024.

## 5. Guards, ordem, atributos e macros

### 5.1 Guards

Ausência é igual somente a ausência. Guards presentes exigem igualdade estrutural após a
mesma normalização de bindings do corpo. Guard diferente bloqueia V27, mesmo que ambos
sejam constantes ou atualmente produzam o mesmo valor.

### 5.2 Ordem

Na primeira versão, `order_safe` significa **braços adjacentes**. Não tentar provar
disjunção atravessando braço intermediário. Casos não adjacentes podem ser contabilizados
em canal experimental `NON_ADJACENT`, mas não geram V27 de produção.

Um grupo adjacente conserva a posição do primeiro braço e a ordem textual das
alternativas. Wildcard, range, binding total ou padrão já subsumido bloqueia sugestão se
a segurança não for sintaticamente demonstrável.

### 5.3 Atributos

Braços com atributos são compatíveis somente se suas listas de atributos forem
estruturalmente idênticas. Qualquer `cfg`/`cfg_attr` bloqueia sugestão nesta versão, pois a
compilação pode remover alternativas separadamente.

### 5.4 Macros

Corpo ou padrão contendo macro fica fora do achado acionável. Registrar separadamente
`MACRO_UNPROVEN`; não comparar apenas o texto de duas invocações. Em particular,
`todo!()`, `unimplemented!()`, `unreachable!()` e placeholders configurados não geram
V27.

## 6. Mudanças de modelo e camadas

### L1 — contrato e regra pura

1. ampliar `DecisionArm` em `01_core/entities/rule_traits.rs` com:
   - guard estrutural/canônico, não apenas os booleanos atuais;
   - bindings e modos;
   - fingerprint e forma estrutural confirmável do corpo;
   - atributos;
   - proveniência de macro/placeholder;
   - índice do braço dentro da decisão;
2. implementar `01_core/rules/mergeable_decision_arms.rs` sem I/O;
3. exportar a regra em `01_core/rules/mod.rs`;
4. manter V27 independente da contagem de V19 e V20.

Evitar tornar L1 dependente de `tree-sitter`. L3 projeta uma IR explícita; L1 decide
somente sobre essa IR.

### L3 — extração Rust

Em `03_infra/rs_parser.rs`:

1. extrair a árvore canônica de corpo e guard preservando operadores sem nome;
2. extrair bindings por alternativa de padrão e seus modos;
3. mapear usos dos bindings em guard e corpo;
4. marcar macros, placeholders e atributos condicionais;
5. preservar spans necessários ao diagnóstico e à sugestão de laboratório.

Adicionar testes de parser antes dos testes da regra. Um mock L1 não prova que L3
preserva operadores ou modos de binding.

### L2/L4 — registro e apresentação

1. registrar V27 no catálogo textual e SARIF;
2. atualizar a cardinalidade V0–V27 e testes que hoje esperam 27 regras;
3. emitir `ViolationLevel::Info` durante o piloto;
4. apresentar grupo, linhas envolvidas e os padrões sugeridos;
5. não alterar código-fonte nem política de exit code;
6. adicionar configuração de exclusão de placeholders somente se a lista padrão precisar
   ser extensível; não criar threshold numérico sem evidência.

## 7. Diagnóstico e sugestão

Formato mínimo:

```text
V27: braços nas linhas 10 e 12 possuem consequência estruturalmente idêntica;
verifique possível copiar/colar ou una os padrões como `Kind::A | Kind::B`
```

O span principal aponta para o segundo braço; a mensagem cita o primeiro. A ordem dos
grupos é `path, match line, first arm index`. Um braço participa de no máximo um grupo
maximal.

A sugestão textual não deve alegar `MachineApplicable`. O laboratório pode gerar um
patch separado somente para casos `PROVEN_ADJACENT`, aplicar em cópia temporária e exigir:

```text
parse(before) = PASS
parse(after)  = PASS
V27(after)    = V27(before) - 1 grupo
V19(after)    = V19(before) + alternativas introduzidas
```

A variação esperada de V19 é observação da transformação, não compensação ou score.

## 8. Matriz mínima de fixtures

### Positivos obrigatórios

1. variantes adjacentes sem binding e mesmo corpo;
2. três braços adjacentes formando grupo maximal;
3. braços com guard estruturalmente idêntico;
4. binding idêntico por nome, modo e posição de uso;
5. corpo em bloco com chamadas, operadores e efeitos na mesma ordem;
6. `or-pattern` preexistente unido a um novo padrão compatível.

### Negativos obrigatórios

1. corpos que diferem somente por `==` versus `!=`;
2. chamadas iguais com argumento ou ordem diferente;
3. `x` usado em posições diferentes no corpo;
4. `move`, `ref`, `ref mut` ou mutabilidade divergentes;
5. um binding ausente em uma alternativa;
6. guard ausente, diferente ou com usos diferentes;
7. braços iguais mas não adjacentes;
8. wildcard, range ou padrão intermediário sobreposto;
9. `#[cfg]` ou atributos distintos;
10. macros iguais, macros distintas, `todo!()` e `unimplemented!()`;
11. corpo vazio, salvo decisão explícita posterior;
12. diferenças em `return`, `break`, labels, `continue`, `?` e `await`;
13. comentários e whitespace diferentes com AST igual;
14. literais Unicode e escapes textualmente próximos;
15. tabelas declarativas intencionais, para medir ruído sem suprimi-las por heurística.

### Fronteiras e metamorfismos

- permutar somente braços dentro de um grupo adjacente não muda a cardinalidade;
- inserir comentário ou executar `rustfmt` não muda o resultado;
- trocar qualquer operador nomeado ou anônimo elimina a equivalência;
- inserir braço intermediário muda `PROVEN_ADJACENT` para `NON_ADJACENT`;
- mudar uma ocorrência do binding no corpo elimina compatibilidade;
- substituir corpo por placeholder elimina o diagnóstico.

## 9. Mutantes obrigatórios

Cada mutante deve sobreviver no baseline e morrer pela suíte nova:

1. fingerprint apaga operadores anônimos;
2. igualdade usa somente hash;
3. guard é ignorado;
4. modo de binding é ignorado;
5. usos de binding são comparados como conjunto, sem posição/multiplicidade;
6. adjacência é ignorada;
7. `cfg` é ignorado;
8. macro textual é considerada corpo comum;
9. placeholder é aceito;
10. agrupador emite pares sobrepostos em vez de grupo maximal;
11. diagnóstico muda de ordem conforme `HashMap`;
12. sugestão perde um padrão preexistente em `A | B`.

Meta mínima: 12/12 mortos. Mutante equivalente precisa de justificativa e substituição;
não conta como morto.

## 10. Piloto no Typst Crystalline

Usar o mesmo corpus congelado de P0115 antes de atualizar números. Se qualquer SHA-256
divergir, produzir novo manifesto e marcar `CORPUS-DRIFT`.

Relatar:

```text
matches analisados
grupos brutos por corpo
PROVEN_ADJACENT
NON_ADJACENT
BINDING_UNPROVEN
GUARD_DIFFERENT
CFG_UNPROVEN
MACRO_UNPROVEN
PLACEHOLDER_EXCLUDED
EMPTY_BODY
```

Revisar manualmente 100% de `PROVEN_ADJACENT` e amostra determinística mínima de 30 dos
demais canais, incluindo todos os falsos fortes conhecidos de P0115.

Para cada candidato acionável, registrar:

```text
path | match_line | arm_lines | patterns | bindings | guard |
body_fingerprint | classification | reviewer_verdict | rationale
```

Aplicar sugestões somente em cópia temporária do corpus. Não editar o checkout consumidor.

## 11. Critérios de promoção

V27 pode permanecer registrada como `Info` se:

- parser e regra passam toda a matriz;
- 12/12 mutantes morrem;
- determinismo é comprovado em três execuções e após `rustfmt`;
- precisão manual de `PROVEN_ADJACENT` é 100%;
- nenhum achado depende de macro, `cfg`, binding não provado ou braço não adjacente;
- catálogo text/SARIF contém V0–V27 exatamente uma vez;
- suíte integral, `cargo fmt --check`, `git diff --check`, dry-run de hashes e auto-lint
  passam.

Autofix de produção exige passo posterior e autoridade própria. Mesmo com precisão de
diagnóstico de 100%, P0116 não o promove automaticamente.

Encerrar com exatamente um veredito:

- `IMPLEMENTED — V27 INFO`; ou
- `LAB CLOSED — V27 DETECTOR ONLY`; ou
- `SPEC-GAP — V27`; ou
- `REJECTED — V27 IMPLEMENTATION`.

## 12. Ordem de execução segregada

1. congelar baseline, corpus e autoridade manual;
2. escrever fixtures negativas/positivas e mutantes antes da implementação final;
3. ampliar IR L1 e extração L3;
4. implementar classificador puro V27;
5. integrar catálogo, apresentação e SARIF como `Info`;
6. executar testes unitários, integração e mutação;
7. executar piloto sem writes no Typst Crystalline;
8. testar patch somente em cópia temporária;
9. confrontar achados com autoridade humana;
10. escrever assessment, hashes, limitações e veredito.

## 13. Fora de escopo

- regra inversa de V20 ou ocupação de V28;
- score `V19 - V27` ou cancelamento entre regras;
- equivalência semântica geral de expressões;
- resolução completa de tipos ou expansão de macros;
- consolidação não adjacente em produção;
- autofix no checkout analisado;
- supressão automática de tabelas apenas por tamanho;
- promoção de `Info` para `Warning` antes do piloto.

## 14. Evidência de fechamento

Materializar `lab/assessment_v27_implementation.md` contendo:

1. commit-base e estado dos dois repositórios;
2. hashes de fixtures, autoridade, corpus e resultados;
3. delta explícito da IR e do registro V0–V27;
4. matriz fixture→resultado;
5. tabela mutante→teste matador;
6. relatório do piloto e revisão humana;
7. patches de laboratório antes/depois, sem aplicá-los ao consumidor;
8. regressões V19/V20 e suíte completa;
9. limitações residuais;
10. veredito único da seção 11.

Referências conceituais: Rust Clippy `match_same_arms`, Rust Reference sobre
`or-patterns` e erro E0409 sobre consistência de bindings. Essas referências orientam o
contrato, mas a evidência de promoção pertence aos testes e ao corpus deste passo.
