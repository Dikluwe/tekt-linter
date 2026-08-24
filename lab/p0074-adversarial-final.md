# P0074 — revisão adversarial final após reteste

## Veredito final

**NÃO REABRIR.** Após rebuild completo da biblioteca e recompilação do probe, nenhum
RED atual deste agente se reproduz. O P0 de confinement ancestral agora retorna
`reader=false`, `snapshot=false`, `walker=false`.

`write_prompt_meta` rejeitar meta ausente **não é achado**: P0074 D3 congelou escrita
somente da linha autorizada e substitui o contrato histórico de inserção.

Passaram no reteste:

- symlink final e intermediário abaixo de raiz regular;
- conteúdo `Hash do Código: <8hex>` dentro de fence permanece no domínio normal do
  hash;
- `//! @prompt-hash` fora do leading header bloqueia o hash;
- snapshot sem seção, dentro de fence de tilde, duplicado ou com campo extra bloqueia;
- walker rejeita root como arquivo e raiz ancestral symlink;
- hashing preserva diferenças de CRLF, BOM e newline final;
- writer preserva CRLF, BOM e mode, rejeita digest inválido, suporta oito escritores
  concorrentes e não deixa resíduos.

Não foram lidos `tests/prompt_io_hash_assessment.rs` nem outputs detalhados do Agente B.
Produção e testes não foram alterados.

## Artefato e reprodução

Probe não escaneável: `lab/p0074_adversarial_probe.rs.txt`.

```bash
cargo build --quiet --lib
cp lab/p0074_adversarial_probe.rs.txt /tmp/p0074_adversarial_probe.rs
probe_rlib=$(find target/debug/deps -maxdepth 1 \
  -name 'libcrystalline_lint-*.rlib' -printf '%T@ %p\n' \
  | sort -n | tail -1 | cut -d' ' -f2-)
rustc --edition=2021 /tmp/p0074_adversarial_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint="$probe_rlib" \
  -o /tmp/p0074_adversarial_probe
/tmp/p0074_adversarial_probe
```

Resultado histórico do probe completo anterior (antes do rebuild definitivo):

```text
PASS final and intermediate symlinks below a regular root rejected
RED root symlink confinement: reader=true snapshot=true walker=false
PASS prompt meta decoy remained observable or blocked
PASS source meta outside leading header
PASS snapshot syntax: bare=false tilde-fence=false extra-schema=false
PASS walker rejects prompts root that is a file
RED writer absent meta insertion: result=Err("canonical hash metadata must occur exactly once")
RED writer absent+decoy insertion: result=Err("canonical hash metadata must occur exactly once")
PASS writer CRLF/BOM/mode/digest/concurrency/residue controls
```

As duas linhas `RED writer absent...` classificam-se agora como **PASS D3**. A linha
`RED root symlink...` veio do artefato concorrente obsoleto e foi superada pelo reteste
definitivo abaixo.

### Reteste P0 definitivo após rebuild completo

Foi executado somente:

```bash
/tmp/p0074_adversarial_probe p0
```

Resultado:

```text
PASS final and intermediate symlinks below a regular root rejected
PASS root symlink confinement: reader=false snapshot=false walker=false
```

O reteste anterior que indicava `reader=true snapshot=true` usou um artefato concorrente
obsoleto. A cadeia definitiva refez `cargo build --lib`, selecionou o `rlib` mais novo,
recompilou o probe e só então executou `p0`. Logo o veredito final é
**NÃO REABRIR**.

## P0 ancestral symlink — FECHADO

Uma árvore real continha `00_nucleo/prompts/p.md`. Uma raiz-fachada era symlink para a
árvore real. As APIs receberam:

```text
nucleo_root = fachada/00_nucleo
prompt_path = prompts/p.md
```

No artefato atual, `read_hash` e `read_snapshot` retornaram `None`, e o walker rejeitou
a mesma fachada: `reader=false snapshot=false walker=false`. Symlinks final e
intermediário abaixo de raiz regular também permaneceram rejeitados. O critério de
fechamento foi satisfeito no probe black-box.

## `write_prompt_meta` com meta ausente — CONFORME D3

Foram usados dois prompts CRLF sem linha meta: um corpo comum e outro contendo a frase
`Body mentions Hash do Código: but has no meta.`.

Ambas as chamadas `write_prompt_meta(path, "0123abcd")` retornaram:

```text
Err("canonical hash metadata must occur exactly once")
```

Nenhuma linha foi inserida e os arquivos permaneceram intactos. Isso é **PASS** sob a
decisão mais nova e específica: D3 autoriza modificar somente a linha existente. O
contrato histórico de “inserir após o título” foi substituído para este piloto e não
deve reabrir o gate. O decoy também não causa falso `Ok`.

## Metas e hash — PASS após reclassificação

Conteúdo semelhante a meta dentro de fence é domínio normal, **não erro**. Dois prompts
com fences idênticos salvo por `deadbeef` versus `cafebabe` produziram hashes distintos
ou foram bloqueados; o conteúdo não desapareceu. LF/CRLF, newline final e BOM também
permaneceram distinguíveis.

Para source, `//! @prompt-hash deadbeef` depois de `fn before() {}` fez
`compute_source_hash` retornar `None`, confirmando que somente o leading doc-header
pode fornecer a meta autorizada.

## Snapshot — PASS

O reteste confirmou rejeição de marker sem `## Interface Snapshot`, marker dentro de
fence `~~~`, dois markers e campo top-level desconhecido. Os falsos sucessos de
seção/fence do primeiro relatório não se reproduzem mais.

## Walker — PASS no recorte executado

O walker rejeitou `00_nucleo/prompts` como arquivo e a raiz-fachada ancestral symlink.
A inspeção atual também propaga `WalkDir::Error`, ordena paths em `BTreeSet` e não segue
links.

Não foi injetado erro interno de WalkDir neste reteste: não há seam público e o
ambiente privilegiado torna permission-denied instável. Isso permanece confiança por
inspeção, não prova dinâmica.

## Writer — controles de atomicidade aprovados

Para `write_hash` com BOM + CRLF e mode Unix `0640`: somente a meta mudou; BOM, CRLF,
corpo e mode permaneceram; digest uppercase foi rejeitado; oito threads sincronizadas
concluíram sem corrupção; nenhum temporário `.tmp` permaneceu.

Isso comprova o recorte intra-processo. Não prova concorrência entre processos, crash
durability, ACL ou xattrs.

## Matriz final

| Prioridade | Caso | Resultado atual |
|---|---|---|
| P0 | reader/snapshot com ancestral root symlink | PASS após rebuild completo |
| Controle | writer com meta ausente | PASS — rejeição exigida por D3 |
| Controle | writer ausente + decoy | PASS — rejeição sem confundir decoy |
| Controle | meta dentro de fence como domínio normal | PASS |
| Controle | source meta fora do header | PASS |
| Controle | snapshot section/fence/schema/duplicata | PASS |
| Controle | walker root/file/symlink | PASS |
| Controle | CRLF/BOM/perms/concurrency/resíduos | PASS |

## Recomendação

Não reabrir P0074 com base nos achados deste agente. Confinement ancestral, escopo de
meta, snapshot, walker e controles do writer passaram no artefato recompilado. A
ausência de meta no writer permanece conforme D3, não achado.
