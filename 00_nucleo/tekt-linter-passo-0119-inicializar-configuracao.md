# Passo operacional 0119 — inicializar `crystalline.toml` com segurança

> **Estado:** PLANEJADO — não executado
> **Data:** 2026-08-26
> **Baseline funcional:** `7013ac76378f0800c6c35565081ec74e251757aa`
> **Objetivo terminal:** um projeto novo sem configuração recebe um
> `crystalline.toml` mínimo, determinístico e válido por comando explícito, sem sobrescrever
> configuração existente nem inventar uma arquitetura ambígua

## 1. Problema

Hoje, quando `crystalline.toml` não existe, o lint usa `CrystallineConfig::default()` em
memória e não deixa no projeto um contrato revisável. Isso facilita a primeira execução,
mas cria duas dificuldades:

- o projeto parece configurado embora nenhuma política esteja versionada;
- cada projeto novo precisa montar manualmente o TOML antes de customizar camadas,
  exclusões, portas e gravidades.

A correção não deve fazer o comando normal de lint escrever silenciosamente no projeto.
Análise permanece somente leitura. A criação será uma operação explícita e separada.

## 2. Interface congelada

Adicionar o comando:

```text
crystalline-lint init [PATH]
```

Sem `PATH`, usar `.`. O destino canônico é:

```text
<raiz-do-projeto>/crystalline.toml
```

Saídas terminais:

| Estado observado | Resultado |
|---|---|
| arquivo ausente e topologia reconhecível | criar exatamente um TOML; exit 0 |
| arquivo regular já existente | não escrever; exit 1 |
| destino é symlink, diretório ou tipo especial | não seguir/não escrever; exit 2 |
| raiz inexistente, symlink ou inacessível | zero escrita; exit 2 |
| topologia ambígua ou sem evidência suficiente | zero escrita; exit 2 e listar a ambiguidade |
| falha durante publicação | preservar estado anterior; exit 2; nenhum arquivo parcial |

Não adicionar `--force` neste passo. Sobrescrita e migração de configuração existente são
problemas diferentes e exigem decisão própria.

`init` passa a ser palavra reservada no primeiro argumento. Para analisar um diretório
literalmente chamado `init`, usar `crystalline-lint ./init`. O subcomando aceita somente
seu `PATH`; flags próprias do lint (`--config`, `--fix-hashes`, `--update-snapshot`,
`--dry-run`, `--quiet`, `--format`, `--checks`) combinadas com `init` são erro de parsing
Clap, exit 2 e zero escrita.

### Apresentação normativa

Usar `human_path` para representar paths. Não imprimir aspas. Os resultados de domínio
têm bytes e canais exatos:

```text
sucesso  stdout: CREATED <path>/crystalline.toml\n
         stderr: vazio

Present  stdout: vazio
         stderr: crystalline-lint: config already exists: <path>/crystalline.toml\n

Unsafe   stdout: vazio
         stderr: crystalline-lint: unsafe config initialization: <reason>\n

Unknown  stdout: vazio
         stderr: crystalline-lint: cannot infer Tekt topology: <reason>\n
```

Razões fechadas deste passo:

- `project root does not exist`;
- `project root is not a physical directory`;
- `project root is not accessible`;
- `config destination is not a regular file` (quando o destino existe mas não é regular;
  symlink, inclusive dangling/loop, é `Unsafe`, não `Present`);
- `required directories missing: 00_nucleo, 01_core` (listar somente ausentes, nessa ordem);
- `case-ambiguous layer directories: <nomes>` (nomes ordenados por bytes e separados por
  `, `);
- `cannot inventory project root: <erro-do-SO>`;
- `cannot create temporary config: <erro-do-SO>`;
- `cannot write temporary config: <erro-do-SO>`;
- `generated config failed validation: <erro-do-loader>`;
- `config destination appeared concurrently`;
- `cannot publish config: <erro-do-SO>`;
- `cannot clean temporary config: <erro-do-SO>`.

Erros de parsing pertencem ao Clap e seguem sua apresentação vigente; os oráculos exigem
apenas exit 2 e zero escrita para eles.

### Precedência total

Classificar e interromper na primeira condição verdadeira, nesta ordem:

1. erro de parsing Clap;
2. raiz inexistente;
3. raiz symlink, arquivo ou outro tipo que não diretório físico;
4. raiz sem acesso suficiente para inventário;
5. destino existente symlink/diretório/especial (`Unsafe`);
6. destino existente arquivo regular (`Present`), mesmo se inválido ou se a topologia for
   incompleta;
7. falha de inventário;
8. uma ou mais variantes ambíguas por caixa (`Unknown`);
9. ausência de L0 e/ou L1 (`Unknown`);
10. falhas de temporário, escrita, validação, publicação ou limpeza, na ordem operacional.

Para ambiguidade, reunir todas as entradas conflitantes, ordenar pelos bytes do nome e
separar por `, `. Ela precede a lista de diretórios obrigatórios ausentes. Para ausência,
listar somente os canônicos ausentes, sempre `00_nucleo` antes de `01_core`.

## 3. Obrigação observável

O inicializador deve ser função de um inventário congelado da raiz, não da ordem de
enumeração do filesystem:

```text
inventário regular + política de detecção -> InitPlan
InitPlan + escritor atômico              -> InitReceipt
```

Separar:

- `Absent`: destino seguro e ausente;
- `Present`: configuração existente, nunca sobrescrita;
- `Unsafe`: symlink, diretório, arquivo especial ou raiz insegura;
- `Unknown`: topologia sem classificação determinística.

`Unknown` nunca vira configuração padrão por conveniência.

## 4. Detecção de topologia

Reconhecer somente diretórios de camada no primeiro nível da raiz. A tabela inicial é:

| Camada | nomes aceitos sem ambiguidade |
|---|---|
| L0 | `00_nucleo` |
| L1 | `01_core` |
| L2 | `02_shell` |
| L3 | `03_infra` |
| L4 | `04_wiring` |
| Lab | `lab` |

Regras:

1. somente diretório físico regular conta; symlink não é evidência;
2. ordenar por bytes/path antes de construir o plano;
3. pelo menos L0 e L1 devem existir para reconhecer uma topologia Tekt neste passo;
4. camadas opcionais ausentes não são emitidas em `[layers]`;
5. para cada nome canônico, qualquer entrada de primeiro nível que seja diferente em
   bytes, mas igual por `eq_ignore_ascii_case`, torna a topologia `Unknown`; comparar
   somente ASCII e somente contra os seis nomes canônicos;
6. diretórios arbitrários (`src`, `core`, `nucleus`, nomes Unicode e nomes não UTF-8) são
   irrelevantes, salvo se satisfizerem a regra ASCII anterior; não existe outro conceito
   de “alias não reconhecido” neste passo;
7. não inferir camada pelo conteúdo, linguagem, `Cargo.toml`, `package.json` ou imports;
8. não criar diretórios de camada nem `00_nucleo`; somente o TOML.

Topologias personalizadas continuam suportadas pelo carregador, mas exigem TOML escrito
deliberadamente pelo projeto. Inferi-las automaticamente fica fora deste passo.

## 5. Conteúdo gerado

Gerar um documento mínimo, estável e imediatamente carregável:

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

As demais camadas reconhecidas são inseridas em ordem `L2`, `L3`, `L4`, `lab`, tanto em
`[layers]` quanto em `strict_directories`.

Não gerar por padrão:

- allowlists externas;
- exceções V7/V9/V16;
- aliases TypeScript/Python;
- contratos semânticos;
- níveis de regras redundantes com os defaults do binário;
- `[module_layers]`, pois nomes de módulos não podem ser deduzidos apenas da árvore;
- comentários específicos do próprio `tekt-linter`.

O resultado deve terminar com uma única quebra de linha, usar LF e ser idêntico para o
mesmo inventário, independentemente da plataforma e da ordem de criação dos diretórios.

## 6. Fronteiras arquiteturais

- **L1:** entidades puras `ConfigDestinationState`, `DetectedTopology`, `InitPlan` e decisão
  total de criação/recusa;
- **L2:** comando `init`, mensagens, códigos de saída e apresentação do plano;
- **L3:** inventário confinado, serialização TOML determinística e escrita atômica;
- **L4:** sequência validar raiz → classificar destino → inventariar → planejar → criar,
  escrever e sincronizar temporário → validar temporário → publicar sem substituição.

`CrystallineConfig::load` continua sendo a autoridade de validação dos bytes candidatos.
Ele lê o temporário completo antes da publicação. Não há releitura obrigatória depois da
publicação, porque o objeto publicado deve ser o mesmo inode/bytes já validados; o
inicializador não mantém um segundo validador permissivo.

O caminho passado por `--config` não participa de `init` neste passo. O comando sempre
materializa `<PATH>/crystalline.toml`, evitando semântica dupla entre raiz e destino.

## 7. Segurança de escrita

Antes de escrever:

1. validar a raiz pelo mesmo confinamento físico usado para prompts;
2. inspecionar o destino sem seguir symlink;
3. criar temporário exclusivo no mesmo diretório da raiz;
4. escrever bytes completos, sincronizar e validar o temporário com
   `CrystallineConfig::load`;
5. publicar por criação atômica que falha se o destino passou a existir;
6. remover somente o temporário pertencente à operação em caso de falha.

Uma corrida em que outro processo cria `crystalline.toml` entre plano e publicação deve
preservar o arquivo concorrente e retornar conflito. Nunca usar truncamento, rename que
substitui destino existente ou escrita direta no arquivo final.

Em toda falha retornada, a operação tenta remover somente o temporário que criou. Se essa
remoção também falhar, retorna a razão normativa `cannot clean temporary config` e pode
restar exclusivamente esse temporário pertencente à operação; esse é o único terminal
retornado excepcionado da garantia de ausência de resíduos. Queda abrupta, `SIGKILL`,
kernel panic e perda de energia também podem deixar temporário órfão e ficam fora do
contrato. O nome temporário deve ser exclusivo e não previsível. Durabilidade após
publicação (incluindo `fsync` do diretório) fica fora deste passo; o sucesso atesta criação
visível e conteúdo validado.

O contrato executável obrigatório é Unix/Linux, ambiente do linter instalado. Em Windows,
compilação e comportamento básico não devem regredir, mas nomes não UTF-8, arquivo especial
e primitivas de publicação recebem testes condicionais e não compõem o selo P0119.

Nos motivos com `<erro-do-SO>`, interpolar exatamente `std::io::Error::to_string()`, sem
`Debug`, cadeia adicional, código bruto ou path acrescentado. Em `<erro-do-loader>`, usar
exatamente a `String` retornada por `CrystallineConfig::load`. O prefixo normativo fornece
o contexto.

Se a operação principal falhar e a remoção do temporário também falhar, o erro de limpeza
tem precedência e a saída contém somente `cannot clean temporary config: <erro-do-SO>`.
Se a limpeza tiver sucesso, preservar a razão da falha principal. Depois de publicação
bem-sucedida não existe limpeza do destino final.

## 8. Compatibilidade com o lint atual

- `crystalline-lint PATH` continua somente leitura;
- ausência do TOML durante lint continua usando `CrystallineConfig::default()` neste
  passo, para não introduzir quebra indireta;
- `init` seguido de lint deve ter comportamento determinístico e configuração carregável;
- TOML existente válido ou inválido nunca é corrigido por `init`;
- caminhos relativo e absoluto para a mesma raiz produzem os mesmos bytes;
- V1, V2, V5, V7, V15 e V26 não mudam de semântica.

Uma futura política que exija TOML versionado ou gere automaticamente no primeiro lint
fica adiada. Ela demandaria opt-in e migração próprios.

## 9. Fixtures e oráculos independentes

Criar projetos sintéticos temporários, sem copiar configurações de consumidores:

1. raiz canônica com L0–L4 e lab;
2. raiz mínima somente com L0/L1;
3. ordem inversa de criação dos mesmos diretórios;
4. TOML regular preexistente com bytes sentinela;
5. TOML inválido preexistente;
6. destino symlink para dentro e para fora da raiz;
7. destino diretório e arquivo especial;
8. raiz symlink, inexistente e inacessível;
9. somente L1 ou raiz vazia, classificadas como `Unknown`;
10. corrida controlada criando o destino entre plano e commit;
11. paths com Unicode válido e nomes não UTF-8;
12. execução por caminho relativo e absoluto.

O oráculo deve comparar bytes exatos, exit code, stdout/stderr, inventário antes/depois e
ausência de temporários residuais, salvo no caso explícito `cannot clean temporary config`,
em que aceita somente o temporário próprio identificado pelo adapter.

## 10. Ataques obrigatórios

1. lint comum cria TOML implicitamente;
2. `init` sobrescreve arquivo existente válido ou inválido;
3. symlink de destino é seguido;
4. symlink de raiz permite escrita fora do projeto;
5. corrida substitui configuração concorrente;
6. temporário é publicado antes da validação;
7. falha deixa TOML parcial ou lixo temporário;
8. diretório ausente é criado silenciosamente;
9. raiz vazia recebe topologia padrão inventada;
10. detecção depende da ordem do walker;
11. camada é inferida por conteúdo ou linguagem;
12. `[module_layers]` ou allowlists específicas são adivinhadas;
13. caminho absoluto produz bytes diferentes do relativo;
14. configuração gerada não pode ser relida pelo loader real;
15. implementação usa a própria constante de template como único oráculo;
16. erro/`Unknown` é apresentado como sucesso.

Todo mutante válido do fragmento novo deve ser morto. Sobrevivente relevante bloqueia o
veredito.

## 11. Protocolo de execução

Aplicar o protocolo completo da ADR-0020:

1. congelar este passo, baseline e política de `Unknown`;
2. autor do contrato materializa tabela de estados e bytes esperados sem ler o patch;
3. autor de oráculos cria fixtures positivas, negativas e opacas;
4. adversário enumera mutações de sobrescrita, corrida, inferência e ordem;
5. verificador demonstra poder discriminatório e sela contrato com score 1.0;
6. implementador recebe somente passo e contrato selado;
7. integrar implementação e testes independentes;
8. repetir ataques e gates e emitir assessment limitado ao inicializador.

Se contrato, template, política de detecção ou baseline mudar após o selo, invalidar a
cadeia a partir da primeira fase afetada.

## 12. Gates

- testes puros da tabela de estados e do plano;
- golden test dos bytes mínimos e completos;
- testes black-box do comando e códigos de saída;
- testes de confinamento, symlink, arquivo especial e corrida;
- round-trip obrigatório pelo `CrystallineConfig::load` real;
- repetição e permutação do inventário;
- regressão do lint sem TOML, provando zero escrita;
- equivalência de raiz relativa/absoluta;
- auto-lint V1/V2/V5/V7/V15/V26;
- mutation score 1.0 no fragmento novo;
- `cargo fmt --all --check`;
- `cargo test --all-targets`;
- ratchet de complexidade;
- `git diff --check`;
- `--fix-hashes --dry-run`;
- smoke somente leitura em pelo menos um consumidor sem TOML e um com TOML.

O smoke externo não integra o selo reproduzível e não autoriza escrita no consumidor.

## 13. Critério terminal

`READY TO MERGE — SAFE CONFIG INIT` exige:

- criação explícita e única em projeto reconhecível sem TOML;
- bytes determinísticos e round-trip válido;
- zero sobrescrita em todos os estados preexistentes;
- zero escrita fora da raiz e zero seguimento de symlink;
- corrida concorrente preserva o vencedor;
- `Unknown` permanece bloqueante;
- lint comum permanece somente leitura;
- nenhuma inferência de política específica do projeto;
- todos os mutantes válidos mortos;
- veredito independente `APPROVE` com limites de isolamento registrados.

## 14. Fora de escopo

- `--force`, merge ou migração de TOML existente;
- manifesto de associação entre programa e teste;
- criação automática no primeiro lint;
- criação da topologia de diretórios;
- inferência por framework ou linguagem predominante;
- presets específicos de Rust, Python, TypeScript ou consumidores;
- instalação, merge, push, tag ou release.
