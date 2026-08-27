# Passo operacional 0118 — não aplicar V2 ao próprio arquivo de teste

> **Estado:** PLANEJADO — revisão de escopo, não executado
> **Data:** 2026-08-26
> **Baseline funcional:** `961030bb3520acfd74b53858cf55020bfebb8ffb`
> **Regra:** V2 `MissingTestCoverage`
> **Objetivo terminal:** arquivos de teste reconhecidos estruturalmente não recebem V2;
> módulos produtivos L1 sem teste continuam recebendo exatamente uma V2

## 1. Defeito

O parser Python usa chamadas como `pytest.raises()` para inferir cobertura. Quando analisa
o próprio `test_*.py`, um teste idiomático que contém apenas `assert` fica com
`has_test_coverage = false` e recebe V2. Um arquivo equivalente que chama
`pytest.raises()` escapa por acaso.

V2 está perguntando ao arquivo de teste se ele possui outro teste, quando deveria aplicar
a obrigação apenas ao módulo produtivo.

Não é correção válida inserir chamadas artificiais de framework, excluir arquivos ou
mudar sua camada para silenciar o detector.

## 2. Obrigação congelada

Separar duas propriedades:

```text
is_test_artifact     — papel estrutural do arquivo
has_test_coverage    — cobertura associada ao módulo produtivo
```

Tabela normativa:

| Arquivo L1 | É teste | Tem cobertura | Resultado V2 |
|---|---:|---:|---:|
| módulo produtivo | não | não | exatamente 1 Error |
| módulo produtivo | não | sim | 0 |
| arquivo de teste | sim | não aplicável | 0 |
| arquivo de teste | sim | sim ou contém framework | 0 |
| módulo somente declarativo | não | isento pela regra vigente | 0 |

Para camadas diferentes de L1, V2 continua não aplicável.

O conteúdo do arquivo de teste não decide seu papel. `assert`, `pytest.raises`, fixtures,
helpers e parametrização devem produzir o mesmo resultado quando o path é o mesmo.

## 3. Escopo desta revisão

Implementar somente a identidade estrutural de artefatos de teste e seu consumo por V2.
Não redesenhar ainda toda a relação de cobertura, não criar manifesto de associação e não
alterar ownership de prompts.

Adicionar à IR um dado fechado equivalente a:

```rust
enum SourceRole {
    Production,
    Test,
}
```

Se a implementação descobrir que algum path de fonte aceito não pode ser classificado
deterministicamente, deve preservar um estado `Unknown` e projetá-lo como falha V0, nunca
como cobertura. Não é necessário fabricar casos opacos fora do domínio atualmente aceito
pelo walker apenas para ampliar o lote.

O booleano `has_test_coverage` pode permanecer nesta revisão para módulos produtivos. Ele
não pode ser usado para reconstruir `SourceRole`.

## 4. Convenções preservadas

Centralizar a classificação por path no walker, reutilizando as convenções já vigentes:

| Linguagem | Papel `Test` |
|---|---|
| Python | `test_<stem>.py`, `<stem>_test.py` |
| Rust | `<stem>_test.rs` |
| TypeScript | `<stem>.test.ts[x]`, `<stem>.spec.ts[x]` |
| Zig | `<stem>_test.zig` |
| Go | `<stem>_test.go` |
| C | `test_<stem>.c`, `<stem>_test.c` |
| C++ | `test_<stem>.cpp|cc`, `<stem>_test.cpp|cc` |
| Java | convenções `Test`, `Tests` e `_test` já reconhecidas |
| Elixir | `<stem>_test.exs`, `<stem>_test.ex` |

Regras:

- comparação case-sensitive;
- casar nome completo, nunca mera substring;
- stem precisa ser não vazio;
- symlink, diretório ou arquivo excluído não vira evidência de teste;
- classificação independe da ordem do walker;
- blocos inline como `#[cfg(test)]` continuam sendo cobertura do módulo produtivo, não
  transformam esse módulo em artefato `Test`.

## 5. Python — correção obrigatória

Para `test_catalogo.py` e `catalogo_test.py` em L1:

- corpo somente com `assert` → zero V2;
- corpo com `pytest.raises()` → zero V2;
- corpo somente com helpers → zero V2;
- arquivo sintaticamente inválido continua produzindo erro de parse; o papel de teste não
  mascara V0.

Controles negativos:

- `catalogo.py` chamando `pytest.raises()` e sem teste associado → uma V2;
- `contest_catalogo.py`, `testament.py` e `catalogo_test_helper.py` → produção;
- `catalogo.py` com `test_catalogo.py` adjacente → zero V2 no produtivo e zero no teste;
- removendo o teste adjacente → exatamente uma V2 no produtivo.

Chamadas de `pytest`/`unittest` deixam de funcionar como prova suficiente de cobertura de
um arquivo produtivo. Cobertura inline só permanece onde a linguagem possui construção
de teste realmente pertencente ao módulo, conforme contrato vigente.

## 6. Prompts de programa e teste

Programa e teste continuam sendo materializações distintas:

```text
prompt proprietário A ↔ programa
prompt proprietário B ↔ teste
```

Se ambos derivam de uma intenção comum, essa intenção pode ser um núcleo/contrato
compartilhado referenciado pelos dois prompts proprietários. Não compartilhar o mesmo
`@prompt`, pois V15 mantém ownership 1:1.

V2 não compara prompts para decidir cobertura. Ela usa papel do arquivo e associação
estrutural já reconhecida pelo walker.

Um manifesto explícito para testes não adjacentes fica adiado para passo próprio. A geração
automática de `crystalline.toml` também fica fora do P0118.

## 7. Fixtures independentes

Criar fixtures sintéticas mínimas dentro do `tekt-linter`; não copiar arquivos, configuração
ou código de projetos consumidores.

O corpus mínimo contém:

1. módulo Python produtivo sem teste;
2. o mesmo módulo com `test_<stem>.py` de `assert` puro;
3. variante `<stem>_test.py`;
4. teste com `pytest.raises()`;
5. produção com chamada `pytest.raises()` e sem teste;
6. nomes parecidos que não satisfazem a convenção;
7. pelo menos um controle por linguagem suportada para não regredir classificações atuais.

A Tradução participa apenas como smoke test externo, somente leitura:

- antes: cinco V2 nos arquivos `test_*.py` já registrados;
- depois: zero V2 nesses cinco paths;
- nenhum byte da Tradução é versionado no linter;
- esse smoke não compõe o selo reproduzível do contrato geral.

## 8. Protocolo de execução

Aplicar protocolo completo da ADR-0020 porque a mudança altera semântica:

1. congelar este passo e o baseline;
2. autor do contrato define `SourceRole` e a tabela V2 sem ler o patch;
3. autor de oráculos cria fixtures sintéticas RED sem ler a implementação;
4. adversário enumera mutações negativas;
5. verificador demonstra que os oráculos rejeitam todas as mutações válidas;
6. implementar IR, walker, adapters e regra;
7. repetir ataques, gates e smoke externo;
8. emitir assessment com limites honestos de isolamento.

O selo preliminar produzido para a versão anterior do P0118 é inválido, pois a obrigação
protegida mudou e o corpus externo deixou de integrar o contrato.

## 9. Ataques obrigatórios

1. `Test` volta a ser tratado como `Production`;
2. papel depende da presença de `pytest.raises()`;
3. chamada/import de `pytest` cobre módulo produtivo;
4. qualquer substring `test` classifica o arquivo como teste;
5. apenas prefixo ou apenas sufixo Python é reconhecido;
6. `#[cfg(test)]` transforma arquivo Rust produtivo em papel `Test`;
7. parser específico sobrescreve o papel recebido do walker;
8. diretório, symlink ou arquivo excluído conta como adjacente;
9. classificação muda com a ordem de descoberta;
10. papel `Test` isenta V1, V5, V7, V15 ou V26;
11. V15 é relaxada para permitir prompt compartilhado;
12. os cinco achados externos desaparecem por exclusão ou mudança de camada.

Todo mutante válido do fragmento novo deve ser morto. Sobrevivente relevante bloqueia o
veredito.

## 10. Gates

- testes puros da tabela V2;
- testes do classificador de paths;
- testes do parser Python com `assert` e `pytest.raises()`;
- black-box text e SARIF com fixtures sintéticas;
- regressão das convenções das demais linguagens;
- V1/V5/V7/V15/V26 verdes;
- mutation score 1.0 no fragmento alterado;
- `cargo fmt --all --check`;
- `cargo test --all-targets`;
- ratchet de complexidade;
- `git diff --check`;
- `--fix-hashes --dry-run`;
- auto-lint do próprio repositório;
- smoke V2 somente leitura na Tradução.

## 11. Critério terminal

`READY TO MERGE — V2 TEST ARTIFACT IDENTITY` exige:

- fixtures sintéticas discriminatórias verdes;
- módulo produtivo sem teste ainda vermelho;
- teste com `assert` puro verde;
- produção com `pytest.raises()` sem teste ainda vermelha;
- zero regressão nas outras linguagens;
- cinco falsos V2 externos removidos sem editar a Tradução;
- nenhum relaxamento de ownership ou de outras regras;
- todos os mutantes válidos do escopo mortos.

Caso contrário, terminar `REJECTED` ou `BLOCKED — SPEC-GAP`, sem merge.

## 12. Fora de escopo

- copiar corpus de projetos consumidores;
- medir cobertura dinâmica ou qualidade dos asserts;
- associar testes não adjacentes por manifesto;
- gerar ou atualizar `crystalline.toml`;
- alterar V2 para outras camadas;
- mudar severidade, ID ou catálogo da regra;
- ocupar V28;
- merge, instalação global ou push.
