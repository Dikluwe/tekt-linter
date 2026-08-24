# P0073 — revisão adversarial final do Agente C

## Veredito

**NÃO REABRIR.** Os probes próprios não reproduziram os quatro REDs congelados:

- aliases/chaves/diretórios inválidos em `[layers]` foram rejeitados;
- resultados legíveis e ilegíveis foram emitidos juntos em ordem canônica;
- diretório, symlink interno/externo e FIFO não contaram como teste adjacente;
- symlinks de arquivo e diretório não foram seguidos nem leram conteúdo externo.

O subcaso socket ficou **SKIP ambiental**: o sandbox negou
`UnixListener::bind` com `EPERM`. A inspeção mostra que o mesmo predicado
`symlink_metadata(...).file_type().is_file()` usado nos demais tipos também rejeita
socket, mas isso não substitui evidência dinâmica. O gate pode permanecer fechado com
essa limitação explicitamente registrada; não alegar prova black-box do socket neste
ambiente.

Não foram lidos `tests/config_walker_assessment.rs` nem outputs detalhados do Agente B.
Produção e testes não foram alterados.

## Artefato e execução

Probe preservado fora do conjunto `.rs` escaneável:
`lab/p0073_adversarial_probe.rs.txt`.

Comandos reproduzíveis:

```bash
cargo build --quiet --lib
cp lab/p0073_adversarial_probe.rs.txt /tmp/p0073_adversarial_probe.rs
probe_rlib=$(find target/debug/deps -maxdepth 1 \
  -name 'libcrystalline_lint-*.rlib' -printf '%T@ %p\n' \
  | sort -n | tail -1 | cut -d' ' -f2-)
rustc --edition=2021 /tmp/p0073_adversarial_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint="$probe_rlib" \
  -o /tmp/p0073_adversarial_probe
/tmp/p0073_adversarial_probe
```

Resultado final:

```text
PASS layers aliases/unknown/duplicates/separators rejected; Unicode components accepted
PASS canonical order includes mixed readable/unreadable results
SKIP socket fixture: sandbox denied UnixListener::bind (Operation not permitted (os error 1))
PASS directory/symlink/FIFO rejected as adjacent; symlink escape not read (socket may be SKIP above)
```

## Evidência por superfície

### 1. Layers: aliases, Unicode e separadores — PASS

Fixtures rejeitadas por `CrystallineConfig::load`:

- coexistência `lab`/`Lab`;
- chave desconhecida `mystery`;
- `L1`/`L2` apontando ao mesmo diretório;
- diretório vazio, `.`, `..`, absoluto, com `/` ou `\`;
- chave Unicode lookalike `K1`.

Controle positivo com diretórios Unicode de componente único (`ação`, `laboratório`)
foi aceito. Isso preserva nomes nativos válidos sem confundi-los com aliases de chave.
Nenhuma permutação pode criar vencedor incidental porque os estados ambíguos falham no
carregamento.

### 2. Ordenação com erros mistos — PASS

A árvore continha dois `.rs` UTF-8 e um `.rs` com bytes inválidos. O walker retornou
dois `Ok(SourceFile)` e um `Err(SourceError::Unreadable)`. A sequência conjunta de
paths, incluindo o erro, já estava em ordem crescente. O erro de leitura não sumiu e
não deslocou a ordenação para duas coleções separadas.

Limite: o probe produz erro determinístico de `read_to_string`, não um erro interno do
WalkDir. Por inspeção, ambos entram no mesmo `Vec<Result<...>>` e no mesmo
`sort_results`; a plataforma atual não forneceu seam público para injetar erro WalkDir
sem usar o gate de B.

### 3. Adjacent tests com tipos hostis — PASS com socket SKIP

Para `foo.rs` e o candidato `foo_test.rs`:

| Tipo do candidato | Resultado |
|---|---|
| diretório | `has_adjacent_test = false` |
| symlink para arquivo externo | `false` |
| symlink para arquivo interno | `false` |
| FIFO | `false`, sem bloqueio |
| socket Unix | SKIP: criação negada pelo sandbox |
| arquivo regular local | `true` |

O resultado discrimina tipo real, não apenas existência ou nome. O FIFO também prova
que o walker não tentou abri-lo durante a busca de cobertura.

### 4. Escape da raiz — PASS

Foram criados na raiz um symlink `.rs` para arquivo externo e um symlink de diretório
para árvore externa. Nenhum `SourceFile`/erro saiu da raiz, nenhuma entrada de escape
foi emitida e o conteúdo sentinela externo não apareceu em qualquer arquivo lido.
`follow_links(false)` e o filtro por `file_type().is_file()` mantiveram o confinamento.

## Inspeção de regressões imediatas

- A ordenação usa `Path` nativo, não conversão lossy.
- Erros WalkDir são convertidos para `SourceError::Unreadable` com path antes da
  ordenação; não há mais `entry.ok()`.
- O teste adjacente usa `symlink_metadata`, que não segue link, e exige `is_file()`.
- A enumeração é materializada antes de ordenar. Isso abandona a laziness histórica,
  mas implementa diretamente a decisão P0073 de ordem global canônica. É mudança de
  contrato conhecida, não RED desta revisão.

## Recomendação

Não reabrir P0073 com base nesta revisão. Registrar o socket como cobertura condicional
pendente em ambiente Unix que permita criar AF_UNIX; não converter o SKIP em alegação
de sucesso dinâmico. Os casos executáveis solicitados passaram e não surgiu regressão
equivalente imediata.
