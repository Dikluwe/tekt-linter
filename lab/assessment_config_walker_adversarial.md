# Triagem adversarial — configuração e walker

## Convenção e harness

- **RED:** observação contradiz uma alegação congelada do assessment.
- **SPEC-GAP:** prompt/schema não decide uma política necessária; não inventar
  expectativa para fazê-la falhar.

Todos os fixtures devem ser criados sob um diretório temporário novo, usar nomes fixos
e registrar a árvore antes da execução. Casos Unix de permissões restauram o mode no
teardown. Comparações de enumeração usam paths relativos em bytes nativos, não
`to_string_lossy()`.

São propostas seis propriedades black-box/API de alto sinal.

## P1 — `[layers]` é função, não disputa de `HashMap`

### Fixture determinístico

Criar dois TOMLs equivalentes salvo pela ordem textual:

```toml
[layers]
L1 = "shared"
L2 = "shared"
mystery = "shared"
```

e a permutação inversa. Em cada raiz, criar `shared/x.rs`. Carregar por
`CrystallineConfig::load` e, somente se aceitar, chamar `resolve_file_layer` muitas
vezes em processos novos.

Adicionar controles:

- `L1 = "one"`, `L1` duplicado no próprio TOML: parser deve rejeitar a chave TOML;
- `L1 = "one"`, `l1 = "one"`: a política de case deve ser explícita;
- `Lab = "lab-a"` e `lab = "lab-b"`: ambas são hoje reconhecidas na resolução;
- diretório vazio, absoluto, `.` ou contendo `/` em valor de layer.

### Mutação e oráculo

Mata “primeiro match vence”, aceitar chave desconhecida e validar apenas sintaxe TOML.

**RED exato:** `load` retorna `Ok` para duas chaves que mapeiam o mesmo diretório, ou
para chave desconhecida que disputa um diretório conhecido; ou permutar o TOML/processo
muda L1/L2/Unknown. O alvo valida apenas contratos semânticos, não `[layers]`, e itera
um `HashMap`, logo o primeiro caso já contradiz a alegação mesmo se um seed particular
parecer estável.

**SPEC-GAP:** prompts aceitam `lab | Lab` na leitura, mas não definem se ambas podem
coexistir nem se nomes de diretório podem ser nested paths. Rejeição conservadora é
compatível com o assessment, porém a forma precisa requer decisão de schema.

## P2 — erro de travessia e erro de leitura nunca viram silêncio

### Fixture determinístico

Separar dois subcasos:

1. **Leitura:** criar `good.rs` UTF-8 e `invalid.rs` com bytes UTF-8 inválidos. Consumir
   o iterador inteiro; isso independe de usuário root e deve produzir um `Ok` e um
   `Err(SourceError::Unreadable)`.
2. **WalkDir (Unix):** criar `blocked/inside.rs`, abrir o diretório para montar a árvore,
   depois remover todas as permissões. Executar como UID não privilegiado; restaurar
   permissões no teardown. Alternativa determinística: apagar/trocar o diretório entre
   sua descoberta e descida por sincronização do harness.

### Mutação e oráculo

Mata `filter_map(Result::ok)`, continuar sem materializar erro e confundir erro de
travessia com exclusão.

**RED exato:** `invalid.rs` não gera exatamente um `Err`; ou o erro WalkDir de
`blocked` não aparece como erro observável e o iterador termina como se o subtree fosse
excluído/vazio. O alvo usa `.filter_map(|entry| entry.ok())`, portanto erros do WalkDir
são descartados silenciosamente — candidato RED direto.

**SPEC-GAP:** `SourceError` público pode não possuir variante específica para traversal.
Isso exige extensão ou mapeamento documentado, mas não autoriza silêncio. O prompt diz
que o iterador continua após arquivo ilegível; “fail-fast” aqui significa falha
observável, não necessariamente abortar a enumeração inteira.

## P3 — conjunto e ordem canônica independem da criação

### Fixture determinístico

Criar a mesma árvore em duas raízes, em ordens opostas:

```text
01_core/z.rs
01_core/a.rs
02_shell/m.ts
nested/c.py
```

Usar conteúdo sentinela idêntico por path, configurações idênticas e nenhuma exclusão.
Coletar todos os resultados, convertendo cada `Ok` para `(path relativo nativo,
language, layer, content, has_adjacent_test)` e cada `Err` para posição + identidade do
path. Repetir depois de recriar diretórios e em filesystems suportados diferentes.

### Mutação e oráculo

Mata dependência de `readdir`, ordenar somente diagnósticos depois do walker e sort não
total em erros/oks.

**RED exato:** as duas raízes produzem conjuntos diferentes ou sequências de paths
diferentes. `WalkDir::new(...).into_iter()` não recebe ordenação no alvo; portanto a
ordem observada pode seguir criação/filesystem e viola a alegação 4 mesmo que a etapa
posterior ordene violações.

**SPEC-GAP:** o prompt histórico promete descoberta lazy, mas não ordem canônica do
`FileProvider`; o assessment atual a exige. Se preservar laziness impedir sort global,
é necessária uma estratégia determinística por diretório ou revisão explícita da
alegação, não um teste condicionado à ordem casual do filesystem.

## P4 — symlink e escape não são ausência nem cobertura

### Fixture determinístico (Unix)

Criar fora da raiz `outside/secret.rs`. Dentro da raiz criar:

- `link.rs -> outside/secret.rs`;
- `linked_dir -> outside/`;
- `01_core/foo.rs` e `01_core/foo_test.rs -> outside/secret.rs`;
- symlink quebrado `01_core/broken_test.rs`;
- ciclo de diretório `loop -> .`.

Capturar hashes/atime quando aplicável do alvo externo antes/depois e consumir o walker
com timeout curto.

### Mutação e oráculo

Mata `follow_links`, canonicalizar e perder confinamento, usar `exists()` como prova de
arquivo regular e converter symlink em silêncio indistinguível.

**RED exato:** bytes externos são lidos; path retornado escapa da raiz; loop bloqueia;
ou `foo.rs.has_adjacent_test == true` por causa do symlink. O walker atualmente não
segue entradas symlink via `file_type().is_file()`, mas `check_adjacent_test` usa
`.exists()`: um symlink adjacente válido conta como cobertura, contrariando a alegação
8.

**SPEC-GAP:** o prompt não decide se symlink de fonte dentro da raiz deve ser erro ou
ignorado. O assessment decide apenas “não escapar”; porém sua alegação de que extensão
não suportada é a única filtragem silenciosa pressiona para erro observável. Fixar essa
política antes de classificar o simples descarte de `link.rs` como RED separado.

## P5 — exclusões são exatas e preservam identidade não UTF-8

### Fixture determinístico

Configurar:

```toml
[excluded]
build = "target"
[excluded_files]
one = "dir/lib.rs"
```

Criar `target/a.rs`, `not-target/a.rs`, `targeted/a.rs`, `dir/lib.rs`,
`dir/lib.rs.bak`, `other/dir/lib.rs` e `dir2/lib.rs`. Em Unix, acrescentar dois paths
distintos `dir/\xFF.rs` e `dir/\xFE.rs`, ambos com extensão ASCII `rs`, usando
`OsString` nativo.

Repetir `excluded_files` com `./dir/lib.rs`, `dir//lib.rs`, `dir\\lib.rs`, prefixo e
sufixo. Comparar paths em bytes.

### Mutação e oráculo

Mata comparação por substring/prefixo/sufixo, aplicação global por basename,
normalização lossy e colisão U+FFFD.

**RED exato:** qualquer arquivo além de `target/a.rs` e `dir/lib.rs` é excluído; um dos
dois paths não UTF-8 desaparece, colide com o outro ou recebe identidade do outro; ou a
ordem TOML muda o conjunto. No alvo, exclusão de diretório usa o último componente e
`excluded_files` usa relativo exato quando UTF-8; os controles devem passar.

**SPEC-GAP:** config TOML só representa Unicode, logo não pode nomear exatamente um
arquivo não UTF-8 em `excluded_files`. O comportamento seguro pode ser “não excluível
por essa interface, mas ainda observável”; inventar uma sintaxe `\xNN` seria mudança de
schema. Separadores repetidos e `.` também não têm canonicalização congelada: exigir
que não casem é o teste literal atual, não uma recomendação de UX.

## P6 — teste adjacente exige arquivo regular correto

### Fixture determinístico

Para cada linguagem suportada, criar o source e cada nome adjacente reconhecido. Para
cada candidato, repetir quatro tipos:

1. arquivo regular;
2. diretório com o mesmo nome;
3. symlink para arquivo dentro da raiz;
4. symlink para arquivo fora da raiz ou quebrado.

Cobrir ao menos Rust `foo_test.rs`, TS/TSX `foo.test.*`/`foo.spec.*`, Python
`foo_test.py`/`test_foo.py`, Zig, C/C++, Go, Java e Elixir. Executar também cada arquivo
que já tem nome de teste como o próprio `SourceFile`.

### Mutação e oráculo

Mata `.exists()`, retornar true para o próprio teste e convenções inconsistentes entre
linguagens.

**RED exato:** diretório, symlink externo ou broken link produz `true`; source regular
sem candidato produz `true`; candidato regular exato produz `false`; ou arquivo que já
é teste recebe `true` quando a política pública diz que ele é o teste, não que possui
teste adjacente. Diretórios com nome de teste são candidato RED direto porque
`Path::exists()` aceita qualquer tipo. O alvo também retorna `true` para arquivos Go,
Java e Elixir que já têm nome de teste, divergindo do princípio geral enunciado no
prompt.

**SPEC-GAP:** a tabela histórica congela convenções apenas para Rust, TypeScript e
Python, com adições parciais no código para outras linguagens. Antes de chamar Go/Java/
Elixir “já é teste” de RED normativo, congelar suas convenções; o defeito independente
de linguagem é aceitar diretório/symlink como arquivo adjacente regular.

## Matriz priorizada

| Prioridade | Propriedade | RED mais provável |
|---|---|---|
| P0 | P2 traversal/read | erro WalkDir some em `entry.ok()` |
| P0 | P1 layers | diretório duplicado/chave desconhecida aceitos |
| P1 | P6 adjacent type | diretório ou symlink conta como teste |
| P1 | P3 enumeration | ordem segue criação/readdir |
| P1 | P4 symlink escape | symlink adjacente externo prova cobertura |
| P2 | P5 exclusions/non-UTF8 | controles devem passar; schema non-UTF8 é SPEC-GAP |
