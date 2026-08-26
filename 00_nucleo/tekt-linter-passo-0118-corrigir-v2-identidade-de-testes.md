# Passo operacional 0118 — corrigir V2 pela identidade do artefato de teste

> **Estado:** PLANEJADO — não executado
> **Data:** 2026-08-26
> **Baseline:** `961030bb3520acfd74b53858cf55020bfebb8ffb`
> **Regra:** V2 `MissingTestCoverage`
> **Objetivo terminal:** V2 é emitida somente para módulo produtivo L1 sem cobertura
> estrutural associada; arquivos de teste nunca precisam provar que possuem outro teste

## 1. Defeito confirmado

O detector Python atual mistura duas perguntas diferentes:

1. este arquivo é um artefato de teste?;
2. este módulo produtivo possui cobertura associada?

`PyParser` considera cobertura local apenas quando encontra chamadas reconhecidas de
`pytest`/`unittest`, um arquivo adjacente ou um módulo somente declarativo. Como o próprio
arquivo `test_*.py` também é classificado como L1, testes idiomáticos que usam somente
`assert` recebem V2. Arquivos equivalentes escapam acidentalmente quando chamam
`pytest.raises()`.

Isso é defeito do linter. Inserir chamadas sem necessidade para silenciar V2 é evasão da
ferramenta e fica proibido como correção.

## 2. Obrigação congelada

V2 observa uma relação estrutural dirigida:

```text
artefato produtivo L1 --é verificado por--> artefato de teste
```

Ela não observa uma propriedade recursiva do arquivo de teste. Portanto:

```text
V2(produção L1 sem teste) = 1
V2(produção L1 com teste associado) = 0
V2(arquivo identificado como teste) = 0
```

O conteúdo do teste não altera sua identidade. `assert`, `pytest.raises`, fixtures,
parametrização ou helpers podem contribuir para a qualidade do teste, mas não são o
critério de V2. V2 continua sendo uma regra estrutural de presença, não medição de
cobertura executada, qualidade ou suficiência das asserções.

## 3. Domínio e identidade

### 3.1 Papéis fechados

Adicionar à IR uma classificação fechada, com nomes finais decididos pelo autor do
contrato antes da implementação:

```text
SourceRole::Production
SourceRole::Test
SourceRole::DeclarationOnly
SourceRole::Unknown
```

- `Production`: pode receber V2 quando L1 e sem teste associado;
- `Test`: é evidência candidata para outro artefato e nunca recebe V2;
- `DeclarationOnly`: preserva a isenção vigente, sem fingir cobertura;
- `Unknown`: não vira `Test`, `Production` nem cobertura por default. É projetado como
  exatamente um V0 Fatal com razão fechada e path, nunca como V2 ou sucesso silencioso.

Não codificar `Test` como `has_test_coverage = true`: papel do arquivo e relação de
cobertura são observáveis diferentes.

### 3.2 Convenções mínimas por linguagem

O primeiro lote deve preservar as convenções já reconhecidas pelo walker:

| Linguagem | Artefato de teste |
|---|---|
| Python | `test_<stem>.py`, `<stem>_test.py` |
| Rust | `<stem>_test.rs` e bloco inline `#[cfg(test)]` no produtivo |
| TypeScript | `<stem>.test.ts[x]`, `<stem>.spec.ts[x]`; preservar a matriz cruzada atual, na qual `.ts` e `.tsx` procuram os quatro sufixos |
| Zig | `<stem>_test.zig` |
| Go | `<stem>_test.go` |
| C | `test_<stem>.c`, `<stem>_test.c` |
| C++ | para fontes `.cpp`, `.cc` e `.cxx`: `test_<stem>.cpp|cc`, `<stem>_test.cpp|cc`; headers permanecem fora desta associação |
| Java | `<stem>Test.java`, `<stem>Tests.java`, `Test<stem>.java`, `<stem>_test.java`, com `<stem>` não vazio e case-sensitive |
| Elixir | `<stem>_test.exs` e `<stem>_test.ex`, com `<stem>` não vazio e case-sensitive |

Convenção não enumerada, path não UTF-8, ambiguidade de stem, artefato gerado opaco ou
configuração externa não resolvida é `Unknown` e produz V0 Fatal, nunca associação
inventada. Um nome apenas parecido (`Testament.java`, `contest.py`,
`catalogo_test_helper.py`) não casa sem que a transformação fechada produza um stem alvo
não vazio e um candidato produtivo compatível no inventário.

### 3.3 Relação de cobertura

Substituir o booleano sobrecarregado por informação que preserve pelo menos:

- papel do arquivo analisado;
- identidade lógica do módulo produtivo;
- conjunto determinístico de testes associados;
- origem da associação: inline, adjacente convencional ou manifesto explícito;
- estado `Unknown` com razão quando a associação não puder ser determinada.

L3 extrai papel e relações físicas. L1 recebe somente valores puros e decide V2. L1 não
abre diretórios nem interpreta nomes de arquivo.

A relação substitui a antiga formulação informal 1:1 de cobertura por:

```text
cada evidência de teste aponta para no máximo um módulo produtivo
um módulo produtivo pode possuir um ou mais artefatos de teste
```

Múltiplos testes válidos do mesmo módulo continuam produzindo apenas o estado `Covered`.
Um teste que corresponda a dois módulos é `Unknown(AmbiguousTarget)` e gera V0 Fatal; não
cobre nenhum deles por ordem de descoberta.

Paths relacionais em L1 usam identidade lógica UTF-8 relativa à raiz, separador `/`, sem
`.`/`..`/prefixo absoluto, preservando case e bytes Unicode. Path físico não representável
é `Unknown(NonUtf8Path)`. Ordenação é lexical pelos bytes UTF-8 do path lógico e depois
pela ordem normativa da origem, nunca por `PathBuf` dependente da plataforma.

## 4. Prompts que originam programa e testes

### 4.1 Não ampliar `@prompt` para propriedade plural

P0104/P0106 fixaram `@prompt` como relação proprietária 1:1. Logo é proibido fazer:

```text
programa.py ─┐
             ├─ @prompt prompt-unico.md
test_programa.py ─┘
```

Isso reintroduziria ownership compartilhado e enfraqueceria V15.

### 4.2 Modelo compatível

Quando uma intenção gera programa e teste, materializar três unidades:

```text
                    núcleo/contrato compartilhado
                         ↑                 ↑
prompt proprietário do programa     prompt proprietário do teste
             ↕                                ↕
         programa.py  ───── é verificado por ───── test_programa.py
```

- cada arquivo mantém exatamente um prompt proprietário;
- cada prompt proprietário mantém exatamente um arquivo owner;
- a obrigação comum pode residir em núcleo `.tekt` ou contrato compartilhado com
  dependências nominais;
- o prompt do programa declara comportamento e a obrigação de verificação;
- o prompt do teste declara alvo produtivo, observáveis, ataques e limites;
- V2 usa a relação entre artefatos, não a igualdade de prompts;
- V15 continua fiscalizando a bijeção de ownership.

### 4.3 Manifesto explícito para layouts não adjacentes

O laboratório deve avaliar uma relação declarativa apenas para casos em que a convenção
de paths não basta, por exemplo:

```text
production = "01_core/traducao/catalogo.py"
tests = ["tests/traducao/test_catalogo.py"]
```

Essa relação precisa ter owner arquitetural definido, schema fechado, paths lógicos
case-sensitive, ordenação determinística e validação fail-closed. Ela não deve ser
embutida livremente em comentários nem inferida por imports, nomes de funções ou chamadas
`pytest`.

O manifesto explícito é candidato de laboratório. Se não houver decisão sobre seu owner,
formato ou interação com V15/V26, registrar `SPEC-GAP` e entregar primeiro somente a
correção segura dos arquivos de teste reconhecidos.

## 5. Insumos congelados

| Unidade | SHA-256/OID |
|---|---|
| baseline Git | `961030bb3520acfd74b53858cf55020bfebb8ffb` |
| prompt V2 | `a2cb1487abe13b016ae0e190c47e031f1212b60d8832552e5d8ccfa4f2cc33e6` |
| regra V2 | `6072016519b73d6df8831dc4cfe7cc8f97ee94fd3116e8eda963e338fca86d1f` |
| contrato SourceFile | `b6e8594f867d7e478aad957b51c8ef2c10fd94a0384a2c1a9da0d5a75942a187` |
| walker | `89e10027080a5c4e882d6377a92aeabd183e1e0abfc117ce3c0b333060c01ac5` |
| parser Python | `b482a59a181ba38179f095b389c68585b375ba9881f5dd5712e8f40bf0d0c51c` |
| P0104, bijeção | `30b4db584b6ca73ee518471f25b8ff25207c0212bd48dd21884d56dff22be0e4` |
| P0106, individualização | `5310ff9ea92af4bc699a50a14b771fbc78fec2374b1042c84c883cfc91ac6d85` |

O corpus da Tradução com os cinco falsos positivos deve ser congelado por paths, bytes e
saída integral antes da implementação. Sem esse corpus, o passo pode ser escrito e os
fixtures sintéticos podem ser criados, mas o veredito sobre os cinco achados permanece
pendente.

## 6. Protocolo completo

Este passo altera semântica e exige protocolo completo da ADR-0020.

### A — inventário somente leitura

Produzir assessment com:

1. os cinco falsos V2 da Tradução;
2. pares produção↔teste correspondentes;
3. por que `assert` puro falha e `pytest.raises` escapa hoje;
4. convenções reais de todas as linguagens suportadas;
5. arquivos de teste atualmente classificados como L1 produtivo;
6. prompts proprietários e núcleos dos pares inventariados;
7. layouts adjacentes e não adjacentes;
8. impacto esperado em V1/V5/V7/V15/V26.

### B — contrato e oráculos antes da solução

O autor do contrato recebe este passo, baseline e corpus, sem ler o patch candidato. O
autor de oráculos materializa testes RED independentes. O adversário cria mutações
negativas. O verificador sela somente com score 1.0 sobre mutantes válidos.

### C — implementação mínima

1. introduzir papel fechado do artefato na IR;
2. fazer o walker classificar arquivos de teste sem consultar seu conteúdo;
3. preservar associação produção↔teste separadamente;
4. remover de Python a dependência de chamadas `pytest`/`unittest` como identidade de
   arquivo de teste;
5. manter blocos inline como cobertura do módulo produtivo, não como papel do arquivo;
6. atualizar V2 para ignorar `Test` e avaliar somente `Production` L1;
7. manter `DeclarationOnly` como isenção explícita;
8. transportar `Unknown` sem convertê-lo em cobertura;
9. não alterar severidade, ID ou catálogo de V2;
10. não implementar manifesto plural enquanto o laboratório 4.3 estiver em `SPEC-GAP`.

## 7. Matriz normativa mínima

| Caso | Papel | Cobertura do produtivo | V2 |
|---|---|---:|---:|
| `catalogo.py` sem teste | Production | ausente | 1 |
| `catalogo.py` + `test_catalogo.py` | Production | presente | 0 |
| `test_catalogo.py` só com `assert` | Test | não aplicável | 0 |
| `test_catalogo.py` com `pytest.raises` | Test | não aplicável | 0 |
| `<stem>_test.py` só com helpers | Test | não aplicável | 0 |
| arquivo produtivo chamando `pytest.raises` | Production | ausente | 1 |
| arquivo declarativo | DeclarationOnly | isento | 0 |
| path ambíguo/opaco | Unknown | unknown | V0 Fatal, zero V2 |
| teste excluído/symlink/diretório | não aceito como evidência | ausente | 1 no produtivo |
| teste associado a dois módulos sem declaração | Unknown | unknown | bloqueia associação |

## 8. Ataques obrigatórios

1. trocar `Test` por `Production` nos cinco fixtures;
2. voltar a exigir chamada `pytest` para reconhecer teste;
3. considerar qualquer import de `pytest` como cobertura;
4. fazer módulo produtivo escapar por chamar `pytest.raises`;
5. tratar sufixo contido no meio do nome como teste;
6. aceitar diretório, symlink ou arquivo excluído como teste adjacente;
7. associar `test_a.py` a `a.py` por ordem de descoberta;
8. apagar `Unknown` convertendo-o em `true`;
9. misturar `SourceRole` e `has_test_coverage` num booleano;
10. compartilhar um prompt proprietário entre programa e teste;
11. permitir prompt de teste órfão ou owner plural;
12. alterar V15 para acomodar geração multi-artefato;
13. produzir resultados diferentes sob permutação do walker;
14. corrigir somente Python e regredir convenções já verdes de outra linguagem;
15. esconder os cinco achados por config/exclusão.

## 9. Gates

- testes puros de V2 para todos os papéis e `Unknown`;
- testes do walker para cada convenção suportada;
- cinco regressões Python com `assert` puro;
- controles `pytest.raises` em teste e em produção;
- black-box sobre fixture adjacente e layout separado;
- testes de ownership V1/V5/V7/V15/V26;
- `fix-hashes --dry-run` antes e depois;
- mutation score 1.0 no fragmento novo;
- `cargo fmt --all --check`;
- `cargo test --all-targets`;
- ratchet de complexidade do próprio linter;
- auto-lint completo e `git diff --check`;
- smoke somente leitura na Tradução, comparando exatamente os cinco V2 congelados.

## 10. Critério terminal

O passo pode terminar em:

- `READY TO MERGE — V2 ARTIFACT ROLE`, se todos os mutantes válidos forem mortos, os cinco
  falsos positivos desaparecerem e módulos produtivos sem teste continuarem vermelhos;
- `READY TO MERGE — CORE FIX / MANIFEST DEFERRED`, se a correção por papel estiver selada
  e a relação explícita de layouts não adjacentes permanecer separada;
- `BLOCKED — TEST RELATION SPEC-GAP`, se não houver identidade determinística entre
  produção e teste;
- `REJECTED — V2 IMPLEMENTATION`, se a solução depender do conteúdo das asserções,
  enfraquecer V15 ou converter `Unknown` em sucesso.

## 11. Fora de escopo

- medir cobertura dinâmica, mutation score do projeto consumidor ou qualidade de asserts;
- exigir `pytest`, `unittest` ou framework específico;
- criar chamada fictícia para satisfazer o detector;
- tornar todo arquivo cujo nome contém `test` automaticamente confiável;
- permitir um prompt proprietário para múltiplos arquivos;
- ocupar V28 ou criar novo ID de regra;
- alterar V2 fora de L1;
- escrever na Tradução durante o laboratório;
- merge, instalação global ou push sem autorização posterior.

## 12. Ordem de execução proposta

1. congelar corpus e saída dos cinco V2;
2. inventariar convenções e relações prompt↔owner↔teste;
3. selar contrato `SourceRole`/relação de cobertura;
4. escrever oráculos RED e mutações adversariais;
5. implementar IR, walker, parsers e V2;
6. executar mutação e gates integrais;
7. rodar smoke somente leitura na Tradução;
8. decidir separadamente o manifesto para layouts não adjacentes;
9. produzir assessment, recibos e veredito;
10. solicitar autorização de merge e atualização do linter.
