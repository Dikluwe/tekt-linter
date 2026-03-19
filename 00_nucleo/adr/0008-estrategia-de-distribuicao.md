# ⚖️ ADR-0008: Estratégia de Distribuição

**Status**: `PROPOSTO`
**Data**: 2026-03-16

---

## Contexto

O `crystalline-lint` existe para ser executado em CI de projectos
terceiros a cada PR. O ciclo crítico é:

```
agente escreve código → PR aberto → CI corre linter → violações
reportadas → agente corrige → próximo PR
```

Se o CI de um projecto terceiro precisar de compilar o linter a
partir do código-fonte, esse ciclo fica comprometido. As dependências
pesadas — tree-sitter (biblioteca C com bindings), rayon (runtime
de paralelismo) — acrescentam vários minutos ao job de CI em
runners frios. Num ciclo onde o agente pode abrir dezenas de PRs
por sessão, essa latência acumula e derrota o propósito da
ferramenta.

### Problema secundário — `cargo install` em CI

`cargo install` compila sempre do zero em ambientes de CI sem cache
de artefactos Cargo. Mesmo com cache, a invalidação de `Cargo.lock`
ou de dependências força recompilação. O resultado é indistinguível
do problema primário.

### Solução existente, não formalizada

O README já documenta a instalação via `curl` como caminho rápido.
O que falta é:

1. Uma esteira automatizada que produza os binários a cada release
2. Um conjunto fechado de targets que a esteira é obrigada a cobrir
3. Uma decisão explícita sobre o papel do `crates.io`

---

## Decisão

### 1. Binário estático como artefacto primário de distribuição

O modo de consumo canónico em CI externo é:

```bash
curl -sSL \
  https://github.com/Dikluwe/tekt-linter/releases/latest/download/crystalline-lint-linux-x86_64 \
  -o crystalline-lint
chmod +x crystalline-lint
./crystalline-lint .
```

O binário é descarregado, tornado executável e invocado — zero
compilação, zero dependências de runtime. Este modo é o único
recomendado em esteiras de CI de terceiros.

**`cargo install` é explicitamente desencorajado em CI externo.**
É permitido em terminais locais de humanos onde a latência de
compilação é aceitável, mas não deve aparecer em receitas de CI
documentadas como caminho primário.

A razão não é apenas latência: a esteira de binários garante que
todos os utilizadores de uma versão específica correm exactamente
o mesmo artefacto, compilado com o mesmo toolchain e flags, sem
variação por ambiente. Isso é propriedade de auditoria, não apenas
de performance.

### 2. Automação disparada exclusivamente por tags semânticas

A publicação de um release é disparada exclusivamente pela criação
de uma tag `vX.Y.Z` no repositório. Zero passos manuais.

**Gatilho:**
```yaml
on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'
```

**O que a esteira produz por tag:**

1. **Compilação em matriz** — um job por target (ver secção 3),
   cada um num runner adequado ao target
2. **Strip do binário** — `strip` ou `llvm-strip` para reduzir
   o tamanho do artefacto
3. **GitHub Release** — criado automaticamente com o nome da tag,
   changelog gerado a partir de commits desde a tag anterior
4. **Upload de artefactos** — todos os binários da matriz anexados
   ao release como assets descarregáveis
5. **Publicação no crates.io** — passo final, opcional (ver secção 4)

**Esqueleto do workflow:**
```yaml
jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            use_cross: true
            asset_name: crystalline-lint-linux-x86_64
          - target: aarch64-apple-darwin
            os: macos-latest
            use_cross: false
            asset_name: crystalline-lint-macos-aarch64
    steps:
      - uses: actions/checkout@v4
      - name: Install cross (se use_cross)
        if: matrix.use_cross
        run: cargo install cross --locked
      - name: Build
        run: |
          if [ "${{ matrix.use_cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            rustup target add ${{ matrix.target }}
            cargo build --release --target ${{ matrix.target }}
          fi
      - name: Strip
        run: strip target/${{ matrix.target }}/release/crystalline-lint
      - name: Upload asset
        uses: actions/upload-release-asset@v1
        with:
          asset_name: ${{ matrix.asset_name }}
          asset_path: target/${{ matrix.target }}/release/crystalline-lint
```

A esteira falha se qualquer target obrigatório falhar. A publicação
no `crates.io` não bloqueia o release (ver secção 4).

### 3. Matriz de targets

#### Obrigatórios

| Target | Runner | Motivo |
|--------|--------|--------|
| `x86_64-unknown-linux-musl` | `ubuntu-latest` + `cross` | CI canónico — musl garante zero dependências dinâmicas; funciona em Alpine, Debian, Ubuntu, containers Docker |
| `aarch64-apple-darwin` | `macos-latest` | Uso local humano em Apple Silicon (M1/M2/M3); o runner macOS da GitHub já corre em aarch64 |

**Porquê musl e não glibc para Linux?**

Um binário compilado contra glibc está vinculado à versão de glibc
do sistema em tempo de compilação. Runners de CI têm versões de
glibc variáveis. Um binário musl não tem nenhuma dependência
dinâmica — é executável em qualquer distribuição Linux,
independentemente da versão de glibc ou da presença de libc.
Este é o único formato que pode ser documentado como "funciona
em qualquer CI Linux" sem asteriscos.

#### Adicionais (não obrigatórios nesta fase)

| Target | Motivo para incluir | Motivo para adiar |
|--------|---------------------|-------------------|
| `x86_64-apple-darwin` | Macs Intel ainda em uso | Crescimento de Apple Silicon torna este pool decrescente; pode ser adicionado quando demanda justificar |
| `x86_64-pc-windows-msvc` | Cobertura Windows | Requer runner `windows-latest` e tratamento de `.exe`; relevância baixa para CI Linux-first |

Targets adicionais podem ser incorporados à matriz sem alteração
desta decisão — são extensões, não revisões.

### 4. crates.io como bónus secundário

A publicação no `crates.io` (`cargo publish`) é executada como
último passo da esteira de release, após todos os binários terem
sido produzidos e anexados ao GitHub Release.

**Propriedades deste passo:**

- **Útil mas não crítico**: permite `cargo install crystalline-lint`
  em terminais locais de humanos; não afecta CI de terceiros
- **Não bloqueia o release**: se `cargo publish` falhar (conflito
  de versão, credenciais expiradas, limite de rate), o GitHub Release
  com os binários já foi criado e está funcional
- **Deve ser `continue-on-error: true`** no workflow para garantir
  a propriedade anterior

```yaml
  publish-crates:
    needs: [build, create-release]
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
      - name: Publish to crates.io
        run: cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}
```

A distinção semântica entre binário e crate é intencional: o
binário é um produto de operações; a crate é uma conveniência
para developers. Tratá-los com a mesma criticidade seria
sobre-engenharia.

---

## Consequências

### ✅ Positivas

- **CI de terceiros adota sem fricção**: `curl` + `chmod` + invocação
  é uma linha em qualquer Makefile, justfile ou CI YAML
- **Feedback imediato**: o ciclo agente→PR→CI→violações cai de
  minutos (compilação) para segundos (descarga de binário)
- **Paridade de artefactos**: todos os utilizadores de uma versão
  específica correm exactamente o mesmo binário — auditoria trivial
- **Release sem intervenção humana**: a tag é o único gesto necessário;
  a esteira trata do resto
- **musl garante portabilidade Linux real**: sem "funciona no meu
  Ubuntu mas não no runner Alpine do cliente"

### ❌ Negativas

- **Infra de CI necessária**: o repositório precisa de um workflow
  de GitHub Actions configurado e mantido; segredos (`CRATES_IO_TOKEN`,
  `GITHUB_TOKEN`) precisam de rotação
- **`cross` como dependência de build**: a compilação musl em
  `ubuntu-latest` depende de `cross` (ou configuração manual de
  musl toolchain); adiciona um passo de bootstrapping ao workflow
- **Binários não são verificáveis sem provenance**: utilizadores
  que exigem SLSA ou Sigstore terão de aguardar uma revisão futura
  deste ADR

### ⚙️ Neutras

- A versão semântica (`vX.Y.Z`) é a única fonte de verdade para
  releases — `Cargo.toml` deve estar em sync com a tag
- O README documenta `curl` como caminho primário; este ADR
  documenta o porquê — são documentos complementares
- Adicionar targets futuros à matriz não constitui revisão deste
  ADR — é operação de extensão

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| `cargo install` como modo canónico de CI | Zero infra de release | Latência de compilação compromete ciclo de feedback; variação por ambiente |
| Publicar apenas no `crates.io`, sem binários | Familiar para devs Rust | `cargo install` nas esteiras — mesma latência; não resolve nada para não-Rustaceans |
| Binário glibc em vez de musl para Linux | Compilação mais simples (sem `cross`) | Dependência implícita de versão de glibc; falha silenciosa em distribuições mais antigas ou containers Alpine |
| Release manual (humano cria GitHub Release) | Zero automação a manter | Gargalo humano; inconsistência entre releases; derrota o modelo de agente autónomo |
| Docker image como artefacto primário | Isolamento total | Adiciona Docker como dependência de runtime em CI; aumenta overhead por PR |

---

## Referências

- `README.md` — instalação via `curl` (modo canónico documentado)
- `linter-core.md` — arquitectura do linter e critérios de verificação
- `cargo.md` — nucleação do `Cargo.toml`
- ADR-0004: Reformulação do Motor de Análise (rayon, tree-sitter)
- [cargo/cross](https://github.com/cross-rs/cross) — compilação cruzada para musl
- [GitHub Actions: upload-release-asset](https://github.com/actions/upload-release-asset)
