# Passo operacional 0105 — artefato compartilhável Núcleo Tekt (`.tekt`)

> **Natureza:** envelope operacional temporário; a decisão permanente deve ser registrada
> em ADR antes da produção
> **Estado:** planejado; não executado
> **Branch prevista:** `codex/tekt-nucleus-artifact`
> **Baseline dependente:** `561d993` (P0104 fechado como `BLOCKED`)
> **Decisão humana:** Tekt precisa de um artefato L0 compartilhável, distinto de prompt,
> para expressar invariantes comuns sem quebrar a bijeção `prompt ⇄ código`

## Objetivo

Especificar e implementar no linter o **Núcleo Tekt**, arquivo declarativo `.tekt` que pode
ser consumido por vários prompts proprietários:

```text
núcleo compartilhado `.tekt`
        ├──> prompt A ⇄ código A
        ├──> prompt B ⇄ código B
        └──> prompt C ⇄ código C
```

O novo artefato resolve o `SPEC-GAP` de P0104 sem reabrir ownership plural. `@prompt`
continua exclusivo e biunívoco. Um núcleo não gera código, não substitui prompt e não pode
ser apontado diretamente por header de produção.

P0105 termina quando o formato, o grafo, os hashes, os diagnósticos e o wiring estiverem
implementados e verdes em fixtures próprias. Ele **não** converte ainda os 13 prompts
compartilhados do `tekt-linter` nem arquivos de outros projetos.

## Não objetivo

- não individualizar os 13 grupos encontrados por P0104;
- não alterar `typst-crystalline`, Bateia ou `tekt-cargo-dsm`;
- não criar uma linguagem executável, macros, condicionais ou avaliação dinâmica;
- não permitir que `.tekt` importe código, consulte filesystem ou determine wiring;
- não mover conteúdo automaticamente de Markdown para `.tekt`;
- não aceitar prompt compartilhado como compatibilidade temporária;
- não transformar ADR, passo, relatório ou diagnóstico em Núcleo Tekt;
- não fazer merge, push, release nem instalar o binário.

## Decisões normativas a congelar em ADR

Antes de alterar produção, criar ADR nova em `00_nucleo/adr/` estabelecendo:

1. **duas relações diferentes**:
   - ownership: `prompt ⇄ código`, cardinalidade 1:1;
   - dependência: `núcleo → prompts`, cardinalidade 0:N;
2. núcleo é L0 normativo compartilhável, mas não é prompt;
3. código referencia somente seu prompt proprietário;
4. prompt referencia zero ou mais núcleos por path lógico e SHA-256 completo;
5. a dependência forma DAG; ciclos são inválidos;
6. identidade usa path lógico integral, relativo à raiz, case-sensitive e sem
   canonicalização física em L1;
7. alteração de núcleo invalida seus pins nos prompts e o hash efetivo desses prompts;
8. núcleo órfão é diagnóstico próprio, nunca V7;
9. ausência, parse inválido ou hash divergente falham fechados;
10. `.tekt` não possui `Hash do Código`, pois não possui código proprietário.

O ADR deve decidir também o nome definitivo. Até essa decisão, o termo normativo do passo
é **Núcleo Tekt** e `kind = "nucleus"`.

## Localização e namespace

Local canônico inicial:

```text
00_nucleo/prompts/_nuclei/**/*.tekt
```

A co-localização facilita descoberta, mas `_nuclei` constitui namespace separado:

- prompt walker considera apenas `.md`;
- nucleus walker considera apenas `.tekt` sob `_nuclei`;
- `@prompt .../*.tekt` é inválido;
- referência de núcleo a `.md` é inválida;
- arquivos `.tekt` fora do namespace canônico são erro de inventário, não ignorados;
- symlink, escape da raiz e path não UTF-8 seguem política fail-closed do walker.

## Formato `.tekt` versão 1

`.tekt` usa UTF-8 e sintaxe TOML 1.0 estrita, embora mantenha extensão própria. Não criar
parser textual ad hoc; desserializar para uma estrutura versionada e rejeitar campos
desconhecidos (`deny_unknown_fields`).

Exemplo normativo mínimo:

```toml
tekt = 1
kind = "nucleus"
id = "path"
title = "Identidade lógica de paths"

[[claims]]
id = "logical-identity"
level = "must"
statement = "A identidade é o path lógico integral recebido da fronteira."

[[claims]]
id = "no-physical-canonicalization-in-core"
level = "must-not"
statement = "L1 não consulta nem canonicaliza o filesystem."
```

### Campos da raiz

| Campo | Cardinalidade | Semântica |
|---|---:|---|
| `tekt` | 1 | inteiro `1`; versões desconhecidas falham fechadas |
| `kind` | 1 | literal `"nucleus"` |
| `id` | 1 | `[a-z][a-z0-9-]{0,63}`; identidade humana local estável |
| `title` | 1 | UTF-8 não vazio, máximo 160 bytes |
| `depends` | 0..N | dependências em outros núcleos, path + SHA-256 completo |
| `claims` | 1..N | invariantes atômicas e identificadas |

### Dependências entre núcleos

```toml
[[depends]]
path = "00_nucleo/prompts/_nuclei/identity.tekt"
sha256 = "<64 hex minúsculos>"
```

- `path` deve ser lógico, relativo, canônico e terminar em `.tekt`;
- pares repetidos são erro, não deduplicação silenciosa;
- duas entradas com mesmo path e hashes diferentes são Fatal de entrada;
- dependências são ordenadas por bytes para cálculo e apresentação;
- self-loop e ciclo transitivo são Error bloqueante;
- profundidade e quantidade devem possuir limites configurados por constantes auditáveis,
  não por recursão irrestrita.

### Claims

```toml
[[claims]]
id = "case-sensitive"
level = "must"
statement = "Comparações preservam caixa."
```

- `id`: mesmo léxico do `id` raiz e único dentro do documento;
- `level`: `must`, `must-not` ou `may`;
- `statement`: UTF-8 não vazio, máximo 2 KiB, sem semântica executável;
- ordem das claims é normativa e preservada para apresentação;
- o linter valida estrutura, identidade, grafo e hash; ele não tenta provar a verdade da
  linguagem natural da claim.

O limite acima é deliberado: v1 estrutura contratos compartilhados, mas não finge ser um
provador formal nem uma DSL comportamental.

## Referência em prompt proprietário

Prompts Markdown passam a admitir um bloco estrutural único, antes da primeira seção:

```markdown
Núcleos Tekt:
- 00_nucleo/prompts/_nuclei/path.tekt sha256:<64 hex minúsculos>
```

Sem dependências, o bloco é omitido. Regras:

- no máximo um bloco;
- uma entrada por path;
- paths em ordem lexical de bytes;
- SHA-256 completo, nunca prefixo de oito caracteres;
- texto parecido fora do bloco não cria dependência;
- path ausente, duplicado, fora do namespace ou hash inválido é erro;
- um prompt pode consumir vários núcleos;
- um núcleo pode ser consumido por vários prompts;
- núcleo não lista consumers: a relação reversa é derivada pelo índice integral.

## Hash e causalidade

### Hash do núcleo

`nucleus_sha256 = SHA-256(bytes integrais do arquivo .tekt)`.

Não há linha de hash autorreferente dentro do núcleo.

### Hash efetivo do prompt

Para prompt sem núcleos, preservar exatamente o algoritmo V5 vigente, evitando migração
global sem causa.

Para prompt com núcleos:

```text
effective_prompt_hash = SHA-256(
    prompt_normative_bytes
    || 0x00 || "TEKT-NUCLEUS-V1" || 0x00
    || para cada dependência em ordem de path:
         path_utf8 || 0x00 || nucleus_sha256_bytes
)[0..8]
```

`prompt_normative_bytes` usa a mesma exclusão de metadata mutável já vigente em V5. O
hash usa o digest real do núcleo lido, não apenas o pin declarado. Assim, mudar um núcleo
torna simultaneamente vermelho:

- o pin completo no prompt;
- o hash efetivo do prompt no código.

Dependências transitivas entram pelo digest efetivo do núcleo, definido recursivamente com
o mesmo domínio e ordem total. A especificação deve fixar bytes exatos e vetores de teste
antes da implementação; qualquer ambiguidade é `SPEC-GAP`, não escolha do implementador.

## Regras e diagnósticos

Criar **V26 — NucleusIntegrity** sem sobrecarregar V1/V5/V7/V15.

V26 cobre:

- `.tekt` malformado, versão/kind/campo desconhecido;
- id/path/claim inválido ou duplicado;
- núcleo/dependência ausente ou ilegível;
- pin SHA-256 divergente;
- self-loop ou ciclo transitivo;
- `.tekt` fora do namespace canônico;
- prompt referenciando núcleo de modo malformado;
- núcleo órfão, com nível configurável e default Warning;
- conflito de identidade observado durante agregação.

Separação preservada:

- V1: código sem prompt proprietário válido;
- V5: hash efetivo prompt→código divergente;
- V7: prompt Markdown órfão;
- V15: bijeção prompt⇄código;
- V26: integridade e grafo de Núcleos Tekt.

Falha de I/O que impeça provar o grafo é Fatal de infraestrutura, não Warning V26.

## Fronteiras Tekt

- **L1 entities:** `NucleusDocument`, `NucleusClaim`, `NucleusDependency`,
  `PromptNucleusRef`, paths lógicos owned/borrowed conforme IR vigente.
- **L1 rule:** recebe documentos e arestas já extraídos; valida domínio, DAG,
  cardinalidades e produz V26 deterministicamente; zero I/O.
- **L1 contract:** porta de leitura de bytes/inventário quando necessária, sem tipo L3.
- **L2:** planejamento/apresentação de atualização de pins e hashes; não abre arquivos.
- **L3:** walker confinado, parser TOML estrito, SHA-256 e escrita atômica/rollback.
- **L4:** compõe walkers, preserva `(nucleus, prompt)` na IR, reduz o grafo integral e
  injeta-o na regra L1 e no cálculo V5.

Não transportar `std::fs::Metadata`, paths canonicalizados ou handles de arquivo para L1.

## Protocolo segregado

### A — ADR e vetores normativos

Antes de gates ou produção:

1. criar ADR permanente;
2. criar `00_nucleo/prompts/nucleus-artifact.md`, prompt proprietário da implementação do
   artefato no linter;
3. congelar gramática v1, limites, bytes do hash e pelo menos cinco vetores SHA-256;
4. criar Assessment 0033 com paths e hashes L0 completos;
5. inventariar colisões com nomenclaturas existentes (`nucleus`, `contract`, `spec`,
   `shared`) sem editar outros projetos.

Se a fórmula de hash transitivo ou o tratamento de ciclos não estiver byte a byte
decidido, parar como `SPEC-GAP` antes de B.

### B1 — gate cego do formato

Criar exclusivamente `tests/nucleus_format_assessment.rs`, sem ler produção, cobrindo:

- documento mínimo e completo;
- campos ausentes/desconhecidos;
- versão/kind inválido;
- ids nos limites 0/1/64/65 e Unicode hostil;
- claims vazias, duplicadas, níveis inválidos e limite de bytes;
- TOML duplicado, BOM, CRLF, NUL e UTF-8 inválido;
- round-trip somente se a serialização for parte do contrato.

### B2 — gate cego do grafo

Criar `tests/nucleus_graph_assessment.rs` inteiramente in-memory:

- zero núcleos e DAG simples/diamante;
- self-loop, ciclos de 2, 3 e muitos nós;
- dependência ausente;
- identidades case-sensitive e paths próximos;
- duplicatas e conflito do mesmo path;
- permutações/partições produzem bytes idênticos;
- órfãos e múltiplos prompts consumidores;
- nenhum efeito sobre a bijeção prompt⇄código.

### B3 — gate cego do hash transitivo

Criar `tests/nucleus_hash_assessment.rs` a partir dos vetores L0:

- compatibilidade bit a bit de prompts sem núcleo;
- um e vários núcleos em ordens diferentes;
- mudança de um byte no núcleo invalida pin e V5 efetiva;
- cadeia e diamante sem dupla contagem acidental;
- ciclos nunca produzem digest;
- path, separadores e domínio não admitem concatenação ambígua;
- SHA completo no pin e prefixo de oito somente em `@prompt-hash`.

### B4 — gate do consumidor real

Criar fixture exclusiva e executar o binário para provar:

- `.tekt` válido compartilhado por dois prompts 1:1 não gera V15;
- alterar núcleo gera V26 e V5 nos dois códigos dependentes;
- núcleo órfão gera somente V26;
- código apontando diretamente a `.tekt` falha;
- prompt walker não classifica `.tekt` como prompt Markdown;
- exclusões, symlinks e escapes falham conforme política vigente;
- saída text/SARIF é completa e determinística.

### B5 — gate transacional

Criar spies independentes para o plano de reparo:

- preflight lê e valida todo o DAG antes de writes;
- atualização ocorre topologicamente: pins dos prompts, hashes efetivos dos códigos e
  metadata reversa dos prompts;
- dry-run e execução compartilham o mesmo plano;
- colisão V15, ciclo, missing dependency ou parse inválido bloqueia o lote inteiro;
- falha em qualquer write restaura todos os bytes anteriores;
- segunda passagem valida pins, V5 direta/transitiva e metadata reversa;
- nunca declarar `Nothing to fix` somente porque o hash direto está verde.

### C — implementação

Somente após A e B1–B5 congelados:

1. materializar entidades e regra pura em L1;
2. implementar parser TOML estrito e walker confinado em L3;
3. estender a extração de prompts para referências de núcleo;
4. construir grafo integral determinístico em L4;
5. implementar V26 e registrar CLI/SARIF/config;
6. integrar hash efetivo a V5 sem alterar prompts sem dependências;
7. integrar plano transacional ao reparador existente;
8. atualizar template de prompts com o bloco opcional;
9. resselar somente pelo fluxo oficial;
10. manter P0104 V15 estrita, sem whitelist ou fallback plural.

### D — piloto local

Criar um único Núcleo Tekt real para um domínio pequeno e já compreendido do próprio
linter, preferencialmente identidade lógica de paths. Individualizar apenas os prompts dos
consumers desse piloto; não usar `linter-core` como primeiro caso.

O piloto deve provar:

- núcleo contém apenas claims realmente compartilhadas;
- cada prompt continua completo quanto à responsabilidade exclusiva do seu código;
- remover a referência ao núcleo torna a omissão observável por gate;
- V1/V5/V7/V15/V26 ficam verdes no subgrafo;
- nenhuma claim é copiada mecanicamente apenas para satisfazer cardinalidade.

Se não houver candidato pequeno semanticamente seguro, fechar P0105 com implementação e
fixtures verdes, mas `READY WITH RESIDUAL AUDIT`, adiando o piloto — nunca improvisar
conteúdo normativo.

### E — adversário final

Confrontar explicitamente:

- `.tekt` tratado acidentalmente como prompt ou código;
- código referenciando núcleo diretamente;
- hash calculado somente sobre o pin declarado;
- mudança transitiva não propagada;
- ciclo que termina por cache e parece válido;
- ordem de filesystem/TOML/HashMap alterando digest ou diagnóstico;
- path normalizado em L1;
- órfão escondido por extensão ou namespace;
- reparador escrevendo antes do DAG integral;
- rollback parcial apresentado como sucesso;
- prompt compartilhado reaparecendo como atalho;
- qualquer escrita nos projetos externos.

## Regressões obrigatórias

- B1–B5;
- P0104 B1/B2/B3 e V1/V5/V7/V15;
- prompt walker/reader/io, config, project index e path encoding;
- fix-hashes planning/execution/presentation/rollback;
- todos os parsers de linguagem;
- suíte completa do workspace;
- auto-lint V1/V5/V7/V15/V26;
- `--fix-hashes --dry-run` sem writes;
- `rustfmt --check` somente nos Rust tocados;
- `git diff --check`;
- status/hash dos repositórios externos idêntico antes/depois.

## Saídas esperadas

- ADR permanente do Núcleo Tekt;
- prompt proprietário `nucleus-artifact.md`;
- Assessment 0033 e inventário somente leitura;
- gramática v1 e vetores de hash normativos;
- gates B1–B5 com RED causal congelado;
- parser, grafo, V26, hash transitivo e reparo transacional;
- fixture/piloto local ou residual justificado;
- `00_nucleo/relatorio-p0105-artefato-nucleo-tekt.md`;
- fechamento `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

P0105 não autoriza converter os 13 compartilhamentos de P0104. Após P0105 integrado e o
binário reinstalado em passo próprio, escrever P0106 para classificar cada compartilhamento
como conteúdo proprietário ou claim de núcleo, individualizar os prompts e então retomar
Typst Crystalline e os demais consumidores.
