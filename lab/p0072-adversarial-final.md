# P0072 — relatório adversarial final do Agente C

## Veredito final após reteste

**PASS — NÃO REABRIR O GATE pelos achados D3/D5 deste agente.** Em reteste contra a
produção atual:

- **P1 / D3 fechado:** `CrateRegistry::from_members` rejeitou as colisões normalizadas
  internas em `deps` e `renames`.
- **P0 / D5 fechado:** o preflight rejeitou a fachada cujo `.git` era symlink, antes de
  qualquer resolução de ref ou leitura do object database externo.

Os REDs descritos nas seções históricas abaixo foram observações válidas da primeira
execução, mas não se reproduzem na produção atual. Este reteste executou somente os
dois casos de conformidade já documentados; não explorou vetores novos. Não foram lidos
gates, `tests/*_assessment.rs`, testes de integração do Agente B nem seus outputs.

## Probe executado

Fonte preservada como artefato de laboratório não compilável pelo workspace:
`lab/p0072_adversarial_probe.rs.txt`.

Comandos reproduzíveis a partir da raiz:

```bash
cargo build --quiet --lib
probe_rlib=$(find target/debug/deps -maxdepth 1 \
  -name 'libcrystalline_lint-*.rlib' -printf '%T@ %p\n' \
  | sort -n | tail -1 | cut -d' ' -f2-)
cp lab/p0072_adversarial_probe.rs.txt /tmp/p0072_adversarial_probe.rs
rustc --edition=2021 /tmp/p0072_adversarial_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint="$probe_rlib" \
  -o /tmp/p0072_adversarial_probe
/tmp/p0072_adversarial_probe
```

Resultado observado no reteste final:

```text
PASS D3 from_members rejected normalized dep/rename collisions
PASS D5 .git symlink rejected: refinement mode requires a self-contained object database; linked worktrees, bare repositories and gitdir indirection are not supported
```

O probe D5 ainda constrói o repositório externo e a fachada, mas somente chama
`resolve_commit` se o preflight retornar `Ok`. No reteste, esse ramo de falso sucesso
não foi alcançado.

## Histórico P0 — D5 aceitava `.git` symlink para banco externo

### Construção

1. Criar repositório externo normal, com um commit e object database próprio.
2. Criar diretório-fachada vazio.
3. Criar `fachada/.git` como symlink para `externo/.git`.
4. Chamar `require_self_contained_object_database(fachada)`.
5. Chamar `resolve_commit(fachada, "HEAD")`.

### Observação RED

Ambas as chamadas tiveram sucesso. `Path::is_dir()` segue o symlink; assim a mensagem
que afirma fechar “gitdir indirection” não corresponde ao que foi rejeitado. A segunda
chamada demonstrou que os objetos e refs externos não são apenas metadados tolerados:
eles alimentam efetivamente a leitura B2.

### Impacto

Viola diretamente D5: o modo selado aceita objetos fornecidos fora da raiz nominal. A
limpeza de `GIT_DIR`, `GIT_OBJECT_DIRECTORY` e alternates no ambiente não protege
contra indirection no próprio filesystem. Há também variantes não cobertas pelo gate:

- `.git/objects` como symlink para diretório externo;
- `.git/objects/pack` ou subdiretórios fanout como symlinks;
- troca TOCTOU de symlink após o preflight;
- `objects/info/alternates` ou `.git/config` como symlink trocado entre inspeção e uso.

### Critério para fechar

Rejeitar symlink/indirection em `.git` e em todo caminho capaz de fornecer objetos ou
configuração, usando inspeção que não siga links; vincular o diretório validado ao
diretório efetivamente usado pelos subprocessos. Um probe de `.git` symlink deve
retornar erro contendo “self-contained object database”, e nenhum `rev-parse`/
`cat-file` deve ler a origem externa.

## Histórico P1 — D3 deixava colisões entrarem por `from_members`

### Construção

Um único `MemberCrate` foi fornecido à API pública com:

```text
deps    = {"foo-bar", "foo_bar"}
renames = {"dep-x" -> "real-a", "dep_x" -> "real-b"}
```

`CrateRegistry::from_members` retornou `Ok`. `owner_of` devolveu o membro preservando
as duas deps e as duas renames conflitantes.

### Observação RED

A construção falível não bloqueou nem canonicalizou as colisões. A validação em
`parse_manifest` protege a rota TOML comum, mas D3 define a invariável do registro, e
`from_members` permanece uma rota pública de construção. O registro resultante viola
sua própria documentação de que `name` **e deps** estão normalizados; `renames` também
é consumido por chave de import normalizada.

### Impacto

O mesmo registro pode classificar de forma distinta conforme o chamador tenha usado
`parse_manifest` antes. Isso desloca a ambiguidade para a fronteira pública, em vez de
eliminá-la. Embora produção corrente use `from_root`, futuras composições ou callers
internos podem construir um estado que D3 declara impossível.

### Critério para fechar

`from_members` deve normalizar e validar deps e ambas as pontas de renames, rejeitando
definições conflitantes e deduplicando somente definições semanticamente idênticas.
Alternativamente, tornar impossível construir `MemberCrate` não validado fora do
módulo. O probe acima deve retornar erro tipado, não `Ok`.

## P1 residual — D6 permite controles no texto humano

`machine_path_uri` passou para bytes inválidos e `%`: `0xFF -> %FF` e `% -> %25`.
`human_path` passou para byte UTF-8 inválido: `0xFF -> \xFF`. Porém newline e ESC são
UTF-8/ASCII válidos e permanecem literais no texto humano.

Isso permite que um filename injete nova linha ou sequência de controle no diagnóstico.
D6 congela escape para **bytes inválidos**, mas não explicita controles válidos; logo é
**SPEC-GAP**, não RED inequívoco de D6. Recomenda-se congelar uma política de framing
humano e escapar pelo menos C0/DEL, preservando identidade sem executar controle de
terminal.

Há outro limite a decidir: `machine_path_uri` mantém `:` literal para suportar drive
Windows, mas num path relativo Unix como `evil:name.rs` isso pode ser interpretado como
scheme URI. A política deve distinguir drive prefix de colon comum ou documentar o URI
base/resolução esperada.

## Inspeção D1–D6

| Decisão | Resultado | Evidência/limite |
|---|---|---|
| D1 | PASS | inserção binária, sort+dedup no merge; duplicata e dois particionamentos coincidiram |
| D2 | PASS | desempates column/rule/message observados na ordem congelada |
| D3 | **PASS no reteste** | `from_members` rejeitou deps/renames com colisão normalizada |
| D4 | PASS | menor `Path` contribuinte permaneceu igual sob reversão dos files |
| D5 | **PASS no reteste** | `.git` symlink foi rejeitado pelo preflight; `resolve_commit` não foi chamado |
| D6 | PASS parcial | invalid byte e `%` corretos; controles humanos e colon relativo são SPEC-GAP |

## Ataques de seguimento históricos (não executados no reteste)

1. **P0:** repetir D5 com `.git` real e `.git/objects` symlink externo; procurar
   preflight `Ok` seguido de leitura externa.
2. **P0:** trocar symlink/alternates entre preflight e `rev-parse`; procurar selo ou
   snapshot construído de um store diferente do inspecionado.
3. **P1:** normalização em cadeia pela API pública: nome, deps, rename-key e
   rename-target com colisões idênticas e conflitantes sob permutações.
4. **P1:** filenames com newline, ESC, colon no primeiro segmento e `%HH` literal;
   procurar framing humano injetado ou URI que resolve para identidade diferente.

## Recomendação final ao orquestrador

Os dois blockers próprios deste agente estão fechados: **PASS D3** e **PASS D5**. Não
reabrir P0072 com base nesses achados históricos. A aprovação global continua sendo do
orquestrador e depende dos demais gates; este reteste não os leu nem os substitui.
