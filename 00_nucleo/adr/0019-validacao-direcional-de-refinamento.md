# ADR-0019 — Validação direcional de refinamento sobre fatos observáveis

**Status:** ACEITO — aprovado pelo humano em 2026-08-23
**Data:** 2026-08-23  
**Origem operacional:** `tekt-linter-passo-validacao-de-refinamento.md`  
**Escopo desta revisão:** Etapas A e B1 autorizadas; Git, wrapper e SMT não autorizados

## Contexto

V23–V25 reconhecem perda de contexto, perda de campo e reentrada de autoridade em uma
única versão do código, sob contratos explícitos. ADR-0018 limita honestamente a
análise, mas o produto atual representa um caso não analisável pelo mesmo resultado
externo de um caso preservado: ausência de diagnóstico.

O Alive2 demonstra uma estratégia complementar: validar cada transformação concreta,
comparando fonte e alvo por uma relação direcional de refinamento, em vez de provar o
transformador inteiro. Sua utilidade depende de IR comum, semântica explícita,
contraexemplo e limites declarados. A semântica LLVM, execução simbólica e SMT não são
transferíveis diretamente ao Tekt Linter.

Um experimento descartável em `lab/refinement_probe.rs` comparou fatos normalizados sem
parser, filesystem ou solver. Nove casos cobriram preservação, perda, normalização
permitida/proibida, invenção de autoridade, direcionalidade e evidência desconhecida.
Todos passaram.

## Medição e oráculos

Investigação feita em `2026-08-23T22:50:44-03:00`.

- `tekt-linter`: HEAD `4c9583d8860236788ceaeb05d795d58453d0fc69`, working tree
  não commitado, 63 arquivos rastreados alterados, 2.813 inserções e 704 remoções,
  além de arquivos não rastreados.
- `typst-crystalline` atual: HEAD
  `ffd527c85dd7d547413d33cbc2d27a80e32a3f8c`, working tree rastreado limpo.
- fonte histórica defeituosa:
  `781b207b4a5de9c2bfbe5819918a193d1d9293e5`.

Os dois SHAs do oráculo fornecem transformações reais:

| Contrato | Antes | Depois | Relação esperada |
|---|---|---|---|
| raio contextual | `resolve_pt(0.0)`/`.abs.0` nos exportadores | resolução centralizada preserva `Length` até receber contexto | correção refina o estado defeituoso; apagar contexto não refina o contrato original |
| identidade de fonte | `FontVariations::default()` em `resolve_font_combo` | `style.variations.clone().unwrap_or_default()` | campo obrigatório preservado |
| autoridade `ssty`/math | classificador duplicado e proxies `contains("math")` | owner importado de `font_metrics` e uso de `style.math` | alvo deixa de inventar owners/proxies |

Esses pares devem ser congelados em fixtures mínimas; não se deve depender de o
repositório externo continuar retendo os SHAs.

## Decisão proposta

### 1. Adotar uma capacidade, não presumir nova regra numerada

Criar uma capacidade chamada provisoriamente `refinement`, cujo produto principal é um
`RefinementVerdict`. A decisão sobre exposição como check `V*`, subcomando ou ambos
fica para um ADR de interface depois das fixtures RED. Não alinhar o recurso a números
de passos.

### 2. Comparar conjuntos normalizados de observáveis

A unidade mínima é um `ArtifactFacts`: identificador de artefato, versão da extração e
mapa ordenado de observáveis com identidade estável. A primeira versão recebe snapshots
já extraídos. Comparação direta de ASTs ou commits não pertence ao núcleo.

Relações iniciais:

- `preserve(source, target)`: o valor observável deve sobreviver;
- `may_normalize(source, target, accepted)`: igualdade ou forma alvo declarada;
- `must_not_invent(target)`: o alvo não pode introduzir o fato;
- `must_exist(target)`: reservado para provar necessidade empírica antes de incluir.

Não criar um lattice genérico de estados no primeiro incremento. Estados como
`contextual`, `resolved`, `normalized` e `erased` são valores do domínio do contrato;
a ordem entre eles deve ser declarada quando necessária, não universalizada por
conveniência.

### 3. Resultado ternário é invariante do domínio

```text
Preserved
Violated(Witness)
Unknown(UnknownReason)
```

`Unknown` é distinto de `Preserved` e deve sobreviver até CLI e SARIF. Razões iniciais
devem ser fechadas e mecanicamente identificáveis: observável ausente/ambíguo, parser
sem suporte, construção opaca, contrato parcial e orçamento esgotado. Texto livre pode
complementar a razão, nunca substituí-la.

### 4. Testemunha é estruturada e reproduzível

Uma testemunha contém contrato, relação, artefatos fonte/alvo, chaves, valores, origem
das evidências e versões dos extratores. A testemunha não afirma uma entrada executável
do programa: na primeira versão, é um contraexemplo ao contrato sobre fatos observados.

### 5. Separação por camadas

- **L1:** fatos normalizados, contrato compilado, veredito, testemunha e comparador puro.
- **L2:** argumentos, apresentação e política explícita de exit code.
- **L3:** leitura/escrita de snapshots, extração por linguagem e, futuramente, acesso
  não destrutivo a revisões.
- **L4:** composição dos dois lados e ordenação determinística.
- **Lab:** protótipos; nunca importado pelas camadas principais.

### 6. Entrega em duas etapas

**Etapa A:** comparar dois snapshots explícitos, canônicos e versionados. Nenhum comando
externo e nenhuma manipulação de Git.

**Etapa B, decisão posterior:** extrair snapshots de duas revisões ou envolver um
comando. Só entra após política de efeitos, consentimento e recuperação ser decidida.

### Etapa B1 aprovada — extração explícita do working tree

Em 2026-08-24 o humano autorizou apenas a primeira metade da Etapa B: gerar snapshots
de um diretório explicitamente fornecido, sem acessar revisões Git nem executar
comandos. A extração inicial usa queries tree-sitter Rust declaradas no contrato. O
produto não infere observáveis por nomes e não embute contratos do projeto-oráculo.

Arquivos relativos são confinados à raiz; a saída não contém timestamp e deve ser
byte-a-byte determinística. O formato permanece v1 e registra uma versão estável do
extrator.

### 7. Relação com V6 e V23–V25

- V6 permanece drift de interface pública contra snapshot causal; não é substituída.
- V23–V25 permanecem lint local de um único estado.
- `refinement` compara estados e pode reutilizar os mesmos fatos normalizados.
- Um mesmo achado não deve ser emitido duas vezes no mesmo modo. A política de
  deduplicação deve escolher o diagnóstico com evidência mais forte, preservando a
  proveniência do check suprimido.

### 8. Sem SMT no primeiro incremento

Os três oráculos são decidíveis por comparação finita de fatos declarados. Solver,
execução simbólica, memória e análise interprocedural ficam fora do escopo até existir
um caso real que não possa ser expresso pelo modelo finito.

## Consequências

**Positivas:** distingue silêncio comprovado de incapacidade; oferece testemunha
estruturada; reutiliza fatos de V23–V25; permite validar reescritas de LLM sem provar o
LLM; mantém L1 puro e multi-linguagem.

**Negativas:** exige dois artefatos e identidades estáveis; aumenta o contrato público;
`Unknown` requer representação adequada em text/SARIF e política de CI própria; o modo
snapshot adiciona versionamento de formato.

**Riscos:** mapeamento nominal pode fingir identidade; snapshots podem divergir do
extrator; `may_normalize` amplo pode esconder perda. Mitigações: identidade declarada,
versão do extrator na chave, normalizações enumeradas e fixtures diferenciais.

## Alternativas rejeitadas

- Estender V24 para comparar commits: mistura lint local com verificação binária.
- Tratar ausência de observação como aprovação: falso senso de prova.
- Retornar apenas violation/não-violation: perde a insuficiência de evidência.
- Copiar a semântica de refinamento do LLVM: domínio errado.
- Introduzir SMT agora: custo e superfície sem oráculo que o exija.
- Executar `git checkout` para obter o estado anterior: mutação destrutiva desnecessária.
- Implementar wrapper na primeira entrega: amplia autoridade e efeitos antes de haver
  núcleo estável.

## Interface aprovada para a Etapa A

O recurso entra como subcomando, não como regra `V*`:

```bash
crystalline-lint refine \
  --before before.refinement.json \
  --after after.refinement.json \
  --contract refinement.toml \
  --format text
```

Exit codes: `0` para `Preserved`, `1` para qualquer `Violated`, `2` para `Unknown`
sem violação e para erro de entrada/configuração. Formatos iniciais: `text` e `sarif`.

## Gate

Gate aprovado pelo humano em 2026-08-23. A materialização fica limitada à Etapa A:

- snapshots explícitos, sem leitura de Git ou execução de comandos;
- nenhum código do laboratório é promovido: a solução é reescrita a partir do L0;
- nenhum número `V*` é reservado;
- wrapper, SMT e extração interprocedural continuam não autorizados.
- leitura de Git continua não autorizada; B1 recebe somente um diretório explícito.

## Adenda proposta B2 — fonte imutável de revisões Git

**Estado da adenda:** ACEITA — aprovada pelo humano em 2026-08-24 após o ensaio
ponta a ponta. A materialização está autorizada no branch dedicado, dentro dos limites
desta seção.

### Backend recomendado

Usar um único processo local `git cat-file --batch-command --buffer`, precedido por
resolução única de cada ref em commit/OID e enumeração de blobs com `git ls-tree -z`.
O processo recebe argumentos e protocolo por `Command`, nunca por shell. A exportação
externa de snapshots (B1 + `refine`) permanece alternativa suportada e de menor
autoridade.

A opção A supera C apenas em ergonomia e reprodutibilidade: congela os dois OIDs e
elimina a exportação manual sem tocar no working tree. Em contrapartida, concede ao
produto autoridade para iniciar o executável `git` instalado pelo utilizador e ler os
objetos locais do repositório indicado. Não concede autoridade de rede, escrita,
checkout, build ou execução de conteúdo do repositório.

`gix` não é escolhido agora porque acrescenta uma superfície Rust Git ampla, features
e supply chain ao binário para uma capacidade opcional. `git2` acrescenta bindings a
libgit2 e potencial compilação/linkagem de C. Ambos evitam o protocolo de subprocesso,
mas duplicam no produto compatibilidade que o Git local já fornece. `gix` deve ser
reavaliado se distribuição sem executável Git se tornar requisito. A escolha não muda
as dependências atuais.

### Modelo de ameaça e política de efeitos

Refs, paths, objetos, configuração local e repositórios são não confiáveis. O adapter
deve congelar refs com `rev-parse --verify --end-of-options <ref>^{commit}`, usar apenas
OIDs daí em diante, separar opções com `--`, consumir saídas delimitadas por NUL e
validar tipo, modo, tamanho e framing de cada resposta. Não se aceitam aliases, shell,
nomes de comando derivados do repositório nem mensagens do Git como identificadores
estáveis.

Cada processo deve receber ambiente mínimo com prompts, lazy fetch e locks opcionais
desabilitados (`GIT_TERMINAL_PROMPT=0`, `GIT_NO_LAZY_FETCH=1`,
`GIT_OPTIONAL_LOCKS=0`, `GIT_NO_REPLACE_OBJECTS=1`), configuração global/sistema
neutralizada e configuração de comando que proíba protocolos e hooks. O fluxo usa
blobs crus: não solicita
`--filters`, `--textconv` ou `--follow-symlinks`; portanto não executa filtros,
Git LFS ou drivers externos. Hooks não pertencem aos comandos de leitura usados.
Submódulos (modo `160000`) e symlinks (modo `120000`) não são atravessados; geram
resultado inconclusivo tipado quando forem a fonte declarada de um observável.

Objetos ausentes, inclusive em clone parcial, nunca iniciam fetch. Ref inexistente é
erro de entrada; entrada esperada no tree cujo objeto não pode ser lido é evidência
desconhecida, não ausência. Arquivo realmente ausente no tree continua obedecendo ao
`on_missing` do contrato. Ponteiro LFS é apenas conteúdo de blob e não aciona LFS.

### Orçamentos e atomicidade

Valores iniciais propostos: no máximo 512 paths observáveis, 4 MiB por blob, 32 MiB
somados por revisão e 10 segundos por operação Git. Exceder qualquer limite produz
`Unknown(BudgetExhausted)` ou erro de entrada antes de publicar resultado; conteúdo
nunca é truncado silenciosamente. O processo deve ser encerrado no timeout e toda
saída deve ser validada antes de virar snapshot.

Não há checkout nem temporário por padrão: blobs alimentam diretamente o mesmo
extrator B1 em memória e os dois snapshots só são publicados após sucesso completo.
Uma opção futura de diagnóstico poderá gravá-los atomicamente fora do repositório. O
working tree, índice, HEAD, branch e stash são ignorados e devem permanecer inalterados
mesmo em erro.

### Camadas e portabilidade

L1 recebe uma porta abstrata de conteúdo por path lógico e identidade imutável, sem
tipos Git. L3 implementa filesystem e Git sobre essa porta; L4 resolve os OIDs e chama
o extrator/comparador únicos. A equivalência entre `refine-revisions` e
`snapshot + refine` é requisito de fixture.

Linux, macOS e Windows são suportáveis via `std::process::Command`, stdin/stdout em
bytes e ausência de shell; a materialização deve testar os três. O requisito externo
proposto é Git 2.43 ou compatibilidade demonstrada com `--batch-command` e
`--end-of-options`. Repositórios SHA-1 e SHA-256 devem tratar OIDs como strings opacas;
alternates e shallow clones ficam a cargo da leitura local do Git, sempre sem fetch.

### Condição para aprovação

Se aprovada, a B2 será materializada em commit separado com fixtures RED para
imutabilidade, refs hostis, objetos ausentes, budgets, symlinks, submódulos e
equivalência B1/B2. Até essa aprovação, o gate anterior permanece vigente.

## Referências

- AliveToolkit, [`alive2`](https://github.com/AliveToolkit/alive2).
- Nuno P. Lopes et al., [*Alive2: Bounded Translation Validation for
  LLVM*](https://web.ist.utl.pt/nuno.lopes/pubs.php?id=alive2-pldi21), PLDI 2021.
- ADR-0001, ADR-0002 e ADR-0018 deste repositório.
