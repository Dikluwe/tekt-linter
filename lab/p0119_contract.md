# Contrato observável P0119 — inicialização segura

## Decisão

**SEAL — CONTRACT READY FOR INDEPENDENT ORACLES.**

Este selo cobre apenas a completude e o poder discriminatório do contrato P0119. Não
aprova implementação, não comprova execução de gates e não autoriza `READY TO MERGE`.
Todos os contratos anteriores de P0119, derivados de `e26bd5f`, `aa6fead` ou `7ab329e`,
estão invalidados e não podem servir de entrada à cadeia.

## Recibo causal e isolamento

- Regime: protocolo completo da ADR-0020.
- Papel: autor do contrato; sem autoridade para implementar, escrever oráculos ou julgar
  implementação.
- Spec congelada: `00_nucleo/tekt-linter-passo-0119-inicializar-configuracao.md` no commit
  `611fabd`.
- SHA-256 da spec: `e9440017a99a301f1e042cff95f87e07603397eacd59f173175e56c28b1aa71e`.
- Baseline funcional: `7013ac76378f0800c6c35565081ec74e251757aa`.
- Entradas auxiliares: ADR-0020 e convenções L1 existentes no commit congelado.
- Escrita autorizada: somente este arquivo.
- Implementação candidata: não lida; informada como inexistente durante a autoria.
- Limite: checkout e filesystem são compartilhados entre agentes. A restrição de leitura
  foi observada operacionalmente, mas não imposta por capability sandbox. Classificação:
  **executado sem atestação de isolamento**.

## Plataforma e limites de garantia

O contrato executável e o selo P0119 cobrem Unix/Linux no ambiente do linter instalado.
Windows deve compilar e não regredir no básico, mas testes de nomes não UTF-8, arquivos
especiais e primitivas condicionais de publicação não compõem o selo. Queda abrupta,
`SIGKILL`, kernel panic, perda de energia e durabilidade pós-publicação, inclusive `fsync`
do diretório, estão fora do escopo. Sucesso atesta criação visível dos mesmos inode/bytes
validados antes da publicação.

## Interface e observáveis

Comando: `crystalline-lint init [PATH]`; `PATH` ausente equivale a `.`. O único destino é
`<PATH>/crystalline.toml`; `--config` não participa. `init` é reservado no primeiro
argumento; para lintar diretório literal `init`, usar `crystalline-lint ./init`.

O subcomando aceita apenas `PATH`. Combinação com `--config`, `--fix-hashes`,
`--update-snapshot`, `--dry-run`, `--quiet`, `--format` ou `--checks` é erro Clap: exit 2 e
zero escrita. A apresentação desses erros é a vigente do Clap e não integra os bytes
congelados de domínio.

O oráculo observa código, stdout/stderr byte a byte, inventário antes/depois, identidade,
tipo e bytes do destino, temporário próprio identificado pelo adapter, bytes publicados e
validação pelo `CrystallineConfig::load` real. Paths usam `human_path`, sem aspas.

## Precedência total e estados

Interromper na primeira condição verdadeira:

1. erro Clap;
2. raiz inexistente;
3. raiz symlink, arquivo ou outro tipo não diretório físico;
4. raiz sem acesso suficiente para inventário;
5. destino existente symlink, diretório ou especial (`Unsafe`);
6. destino existente regular (`Present`), válido ou inválido, independentemente da
   topologia;
7. falha de inventário;
8. uma ou mais variantes ambíguas por caixa (`Unknown`);
9. ausência de L0 e/ou L1 (`Unknown`);
10. falhas de criar, escrever/sincronizar, validar, publicar ou limpar temporário, na ordem
    operacional.

| Estado | Predicado | Decisão | Exit |
|---|---|---|---:|
| `Absent` | raiz segura; destino ausente | detectar e talvez criar | — |
| `Present` | destino regular existente | recusar sem escrever | 1 |
| `Unsafe` | raiz insegura, destino não regular ou falha operacional | recusar/falhar | 2 |
| `Unknown` | colisão ASCII de caixa ou obrigatórias ausentes | recusar sem escrever | 2 |

`Unknown` nunca cria, nunca usa default por conveniência e nunca é sucesso. Ambiguidade de
caixa precede ausência. Reunir todas as entradas conflitantes, ordenar pelos bytes do nome
e separar por `, `. Na ausência, listar somente `00_nucleo` e/ou `01_core`, nessa ordem.

## Inventário e topologia

Inventário é a sequência congelada das entradas imediatas da raiz, com bytes de path/nome
e tipo físico, sem seguir symlink, ordenada lexicograficamente por bytes antes do plano.
Somente diretório físico regular com nome canônico exato é evidência:

| Camada | Nome | Ordem |
|---|---|---:|
| L0 | `00_nucleo` | 0 |
| L1 | `01_core` | 1 |
| L2 | `02_shell` | 2 |
| L3 | `03_infra` | 3 |
| L4 | `04_wiring` | 4 |
| lab | `lab` | 5 |

L0 e L1 são obrigatórias; opcionais ausentes não são emitidas. Qualquer entrada diferente
em bytes mas igual a um canônico por `eq_ignore_ascii_case` torna a topologia `Unknown`.
A comparação é somente ASCII e somente contra os seis canônicos. `src`, `core`, `nucleus`,
Unicode e não UTF-8 são irrelevantes se não satisfizerem essa regra. Não há outro alias.
Conteúdo, linguagem, manifests, imports e ordem do filesystem são irrelevantes. Não criar
raiz nem diretórios de camada.

## API L1 proposta

L1 permanece pura, sem CLI, filesystem, TOML ou exit code:

```rust
pub enum ConfigDestinationState {
    Absent,
    Present,
    Unsafe(UnsafeReason),
}

pub struct InventoryEntry {
    pub path_bytes: Vec<u8>,
    pub kind: PhysicalEntryKind,
}

pub struct DetectedTopology {
    pub layers: Vec<DetectedLayer>, // L0,L1,L2,L3,L4,lab
}

pub enum TopologyDecision {
    Recognized(DetectedTopology),
    Unknown(TopologyAmbiguity),
}

pub enum InitPlan {
    Create(DetectedTopology),
    RefusePresent,
    RefuseUnsafe(UnsafeReason),
    RefuseUnknown(TopologyAmbiguity),
}

pub enum InitReceipt {
    Created,
    RefusedPresent,
    RefusedUnsafe(UnsafeReason),
    RefusedUnknown(TopologyAmbiguity),
    PublicationFailed(PublicationFailure),
}

pub fn detect_topology(entries: &[InventoryEntry]) -> TopologyDecision;
pub fn decide_init(
    destination: ConfigDestinationState,
    topology: Option<TopologyDecision>,
) -> InitPlan;
```

L3 inventaria, serializa e publica; L4 respeita a sequência e produz recibo; L2 projeta
`Created -> 0`, `RefusedPresent -> 1`, e todos os demais recibos `-> 2`.

## Bytes normativos

Raiz mínima, somente L0/L1, produz exatamente estes bytes UTF-8:

```toml
[project]
root = "."

[layers]
L0 = "00_nucleo"
L1 = "01_core"

[excluded]
build = "target"
deps = "node_modules"
vcs = ".git"
venv = ".venv"
pycache = "__pycache__"

[analysis.lineage]
strict_directories = ["00_nucleo", "01_core"]
```

Raiz L0–L4+lab produz exatamente:

```toml
[project]
root = "."

[layers]
L0 = "00_nucleo"
L1 = "01_core"
L2 = "02_shell"
L3 = "03_infra"
L4 = "04_wiring"
lab = "lab"

[excluded]
build = "target"
deps = "node_modules"
vcs = ".git"
venv = ".venv"
pycache = "__pycache__"

[analysis.lineage]
strict_directories = ["00_nucleo", "01_core", "02_shell", "03_infra", "04_wiring", "lab"]
```

Cada documento termina com exatamente um LF, sem BOM, CR, whitespace terminal ou linha
extra. Para qualquer subconjunto opcional, remover apenas as linhas e itens ausentes,
preservando ordem e todo o resto. Não emitir `[module_layers]`, allowlists, exceções,
aliases, contratos semânticos, níveis redundantes ou comentários específicos. Caminho
relativo/absoluto e permutação do inventário produzem bytes idênticos.

## Apresentação normativa

`<path>` e `<reason>` são interpolados sem aspas; `\n` é um LF.

| Resultado | stdout exato | stderr exato | Exit |
|---|---|---|---:|
| sucesso | `CREATED <path>/crystalline.toml\n` | vazio | 0 |
| `Present` | vazio | `crystalline-lint: config already exists: <path>/crystalline.toml\n` | 1 |
| `Unsafe`/falha | vazio | `crystalline-lint: unsafe config initialization: <reason>\n` | 2 |
| `Unknown` | vazio | `crystalline-lint: cannot infer Tekt topology: <reason>\n` | 2 |

Razões fechadas:

- `project root does not exist`;
- `project root is not a physical directory`;
- `project root is not accessible`;
- `config destination is not a regular file`;
- `required directories missing: <ausentes>`;
- `case-ambiguous layer directories: <nomes>`;
- `cannot inventory project root: <erro-do-SO>`;
- `cannot create temporary config: <erro-do-SO>`;
- `cannot write temporary config: <erro-do-SO>`;
- `generated config failed validation: <erro-do-loader>`;
- `config destination appeared concurrently`;
- `cannot publish config: <erro-do-SO>`;
- `cannot clean temporary config: <erro-do-SO>`.

Somente as duas razões de topologia usam o envelope `Unknown`. `<erro-do-SO>` é exatamente
`std::io::Error::to_string()`: sem `Debug`, cadeia, código bruto ou path adicional.
`<erro-do-loader>` é exatamente a `String` retornada por `CrystallineConfig::load`.

## Publicação e cleanup

Sequência obrigatória: validar raiz; classificar destino sem seguir symlink; inventariar;
planejar; criar temporário exclusivo e não previsível no mesmo diretório; escrever bytes
completos e sincronizar; carregar/validar o temporário completo com
`CrystallineConfig::load`; publicar o mesmo inode/bytes por criação atômica que falha se o
destino apareceu. Não reler depois da publicação. Proibidos escrita direta, truncamento e
rename substitutivo.

Concorrente que cria o destino antes do commit vence e seus bytes ficam intactos. Toda
falha retornada tenta remover somente o temporário próprio. Se cleanup funciona, preservar
a razão principal e não deixar resíduo. Se cleanup também falha, sua razão tem precedência
e a saída contém somente `cannot clean temporary config: <erro-do-SO>`; pode restar
**exclusivamente o temporário próprio identificado pelo adapter**. Nenhum final parcial ou
arquivo alheio pode ser alterado. Após publicação bem-sucedida não se limpa o destino.

## Matriz de casos

| Caso | Resultado | Exit | Oráculo essencial |
|---|---|---:|---|
| L0+L1 ausente de config | sucesso | 0 | golden mínimo, apresentação e loader real |
| L0–L4+lab, ordens normal/inversa | sucesso | 0 | golden completo idêntico |
| todo subconjunto opcional | sucesso | 0 | somente presentes, ordem fixa |
| config regular válida/inválida | `Present` | 1 | sentinela intacta; precede topologia |
| destino symlink interno/externo/dangling/loop | `Unsafe` | 2 | não seguir; razão fechada |
| destino diretório/FIFO/socket | `Unsafe` | 2 | zero escrita |
| raiz inexistente | `Unsafe` | 2 | razão exata, zero escrita |
| raiz symlink/arquivo/especial | `Unsafe` | 2 | `not a physical directory`, fora intacto |
| raiz inacessível | `Unsafe` | 2 | razão exata, zero escrita |
| falha de inventário | `Unsafe` | 2 | Display exato do erro SO |
| raiz vazia/só L0/só L1 | `Unknown` | 2 | faltantes exatos e ordenados |
| uma/múltiplas variantes de caixa | `Unknown` | 2 | todas ordenadas; precede faltantes |
| Unicode/não UTF-8 não conflitante | conforme L0/L1 | 0/2 | irrelevante à detecção |
| relativo versus absoluto | mesmo domínio | mesmo | mesmos bytes de arquivo |
| corrida cria destino | falha | 2 | concorrente intacto; razão exata |
| falha create/write/sync/load/publish + cleanup OK | falha | 2 | razão principal; sem resíduo |
| falha principal + cleanup falha | falha | 2 | só razão cleanup; apenas temp próprio pode restar |
| `init` com flag de lint | Clap | 2 | zero escrita |
| `init` versus `./init` | subcomando versus lint | variável | reserva do primeiro argumento |
| lint normal sem config | compatibilidade | vigente | zero escrita; default em memória |

Todos os casos de domínio comparam bytes/canais exatos, inventário e sentinelas sem usar
constantes exportadas pelo alvo.

## Mutações discriminatórias obrigatórias

| Mutante | Discriminador |
|---|---|
| lint comum cria config | lint sem TOML + inventário invariável |
| parsing/reserva de `init` diverge | `init`, `./init` e cada flag proibida |
| `Present` válido/inválido é sobrescrito | sentinelas + precedência + exit 1 |
| symlink de destino é seguido ou vira `Present` | quatro variantes de symlink |
| raiz symlink permite escape | sentinela externa |
| destino especial vira ausente | matriz Unix de tipos |
| cria raiz/camadas ausentes | inventário antes/depois |
| `Unknown` cai em template | vazia/só obrigatória + exit 2 |
| faltantes precedem colisão | fixture contendo ambos |
| colisão aceita ou lista incompleta/desordenada | múltiplas variantes byte-ordenadas |
| Unicode/não UTF-8 arbitrário vira conflito | raiz reconhecida com entrada irrelevante |
| alias/conteúdo/linguagem/import infere camada | fixtures sem canônicos |
| ordem do walker altera plano | permutações do inventário |
| opcionais emitidas ausentes/fora de ordem | todos os subconjuntos |
| path absoluto altera TOML | comparação literal relativo/absoluto |
| BOM/CRLF/LF/whitespace ou seção extra | goldens literais |
| erro usa Debug/path/código/cadeia extra | erro SO sintético com Display conhecido |
| loader error é reformatado | String sentinela do loader |
| código/canal/prefixo diverge | apresentação literal de cada razão |
| temporário previsível/não exclusivo | colisões controladas sem alteração alheia |
| publica antes de sync/loader | falhas injetadas; destino permanece ausente |
| usa validador permissivo paralelo | candidato rejeitado pelo loader real |
| escreve direto/trunca/substitui | falha/race com sentinelas |
| relê e falha depois de publicar | instrumentação exige nenhuma releitura |
| corrida substitui vencedor | criação entre plano e commit |
| falha + cleanup OK deixa resíduo | inventário pós-falha |
| cleanup falho preserva razão principal | dupla falha exige somente razão cleanup |
| cleanup falho deixa final/alheio/outro lixo | allowlist de único temp próprio do adapter |
| sucesso tenta limpar destino | identidade/arquivo final após publicação |
| erro/`Unknown` retorna sucesso | código, canais e inventário |
| oráculo importa template do alvo | goldens independentes deste contrato |

O verificador exige ao menos um positivo, negativo e opaco; positivos `PRESERVED`,
negativos `VIOLATED`, opacos `UNKNOWN`; `UNKNOWN` não mata negativo; e
`mutation_score = 1.0`. Sobrevivente relevante bloqueia o selo de implementação.

## Gates e invalidação

Executar posteriormente: testes puros de estados/plano; goldens mínimo/completo e
subconjuntos; CLI black-box; confinamento, symlink, especial e corrida; round-trip pelo
loader real antes da publicação; repetição/permutação; lint read-only; equivalência de
path; V1/V2/V5/V7/V15/V26; mutation score 1.0; `cargo fmt --all --check`;
`cargo test --all-targets`; ratchet de complexidade; `git diff --check`;
`--fix-hashes --dry-run`; smokes somente leitura em consumidores com/sem TOML. Smoke
externo não integra o selo nem autoriza escrita.

Mudança de spec/hash, baseline, template, apresentação, precedência, detecção, sequência de
publicação, cleanup, plataforma ou política de `Unknown` invalida este contrato desde a
primeira fase afetada.
