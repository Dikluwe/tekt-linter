# Triagem adversarial — I/O de prompts, snapshots e hashes

## Convenção e harness

- **RED:** observação contradiz uma alegação congelada do assessment.
- **SPEC-GAP:** a API ou os prompts não distinguem estados necessários para decidir.

Fixtures ficam em raízes temporárias novas. Comparações de escrita capturam antes/depois
os bytes, mode, owner quando observável e listagem do diretório. Corridas só contam como
RED se sincronizadas por barreira ou por filesystem controlado.

São propostas seis propriedades independentes.

## P1 — confinement único para reader, snapshot e `exists`

### Fixture determinístico

Criar `root/prompts/in.md`, `outside/out.md`, diretório `root/prompts/dir.md` e, em Unix,
links `root/prompts/file-link.md -> outside/out.md` e
`root/prompts/dir-link -> outside/`. Instanciar `FsPromptReader` e
`FsPromptSnapshotReader` com `nucleo_root = root`; chamar ambas as APIs com:

```text
prompts/in.md
../outside/out.md
/path/absoluto/out.md
prompts/file-link.md
prompts/dir-link/out.md
prompts/dir.md
"", ".", "prompts/../prompts/in.md"
```

Para paths absolutos, obter o nome do fixture em runtime; `PathBuf::join` ignora a raiz
quando o segundo operando é absoluto.

### Mutação e oráculo

Mata join sem validação, checagem lexical apenas, `exists()` genérico e follow de
symlink intermediário/final.

**RED exato:** `read_hash` ou `read_snapshot` lê bytes externos; `exists` retorna true
para diretório, symlink externo ou path fora da raiz; ou formas não relativas/confinadas
chegam a uma entrada válida. Os dois readers atuais fazem somente `root.join(path)` e
`exists` usa `Path::exists`, portanto `..`, absoluto e links são candidatos RED diretos.

**SPEC-GAP:** os traits retornam `Option`/`bool` e não distinguem ausente, inválido,
escape, link e I/O. Rejeição pode ser representada como `None/false`, mas não permite
diagnóstico acionável. A alegação de confinement é testável; a classe precisa do erro
exige evolução do contrato.

## P2 — hash sensível a bytes, removendo só a meta canônica

### Fixture determinístico

Para prompt e source, calcular hashes de pares que diferem em exatamente um aspecto:

1. sem newline final versus com `\n`;
2. LF versus CRLF;
3. espaço final, tab, BOM e linha vazia extra;
4. texto de corpo `A Hash do Código: B` versus `A Hash do Codigo: B`;
5. bloco/string de código contendo `@prompt-hash` versus texto vizinho;
6. meta canônica no header com valores diferentes — **único par que deve colidir**;
7. segunda meta fora do header — deve permanecer observável ou tornar entrada inválida.

Usar SHA-256 de referência sobre os bytes após remover por offsets somente a linha
canônica autorizada, preservando todos os separadores restantes.

### Mutação e oráculo

Mata `str::lines().join("\n")`, filtro por `contains` e leitura somente UTF-8 quando a
identidade declarada é de bytes.

**RED exato:** newline final, CRLF ou qualquer alteração fora da meta produz o mesmo
hash; ou uma frase-isca no corpo é apagada do domínio do hash. O alvo normaliza todas
as linhas e remove qualquer linha contendo `Hash do Código:`/`@prompt-hash`, logo ambos
os falsos negativos são diretamente reproduzíveis.

**SPEC-GAP:** prompts históricos falam em “linha meta”, mas não congelam sintaxe exata,
posição, quantidade nem se BOM/arquivo não UTF-8 é erro. O assessment congela escopo no
header e identidade byte-sensitive suficiente para classificar normalização e isca
como RED; a gramática canônica ainda precisa ser explicitada.

## P3 — size/read/hash são uma captura e o cache não fossiliza erro

### Fixture determinístico

Controles sem corrida:

- prompt exatamente 10 MiB e 10 MiB + 1 byte;
- arquivo esparso acima do limite;
- diretório, FIFO e arquivo ilegível no path;
- primeira chamada inexistente seguida de criação, usando reader simples e cached;
- primeira chamada válida seguida de troca de conteúdo, usando cached.

Para TOCTOU, somente usar seam/barreira: pausar após metadata, trocar por arquivo maior
ou symlink externo, liberar a leitura e verificar que nenhum hash é publicado. Sem seam,
registrar como não testado, não usar loop probabilístico.

### Mutação e oráculo

Mata limite apenas em metadata, reopen por nome e cache infinito de `None`/hash sem
identidade da captura.

**RED exato:** arquivo acima de 10 MiB gera hash; bytes hasheados não pertencem ao
objeto cujo tamanho foi aprovado; cached reader devolve `None`/hash antigo após mudança
quando o contrato espera visão atual; FIFO bloqueia indefinidamente.

No alvo, metadata e `read_to_string` são operações separadas e o cache guarda
`Option<String>` somente por string do path. A janela TOCTOU é real por inspeção, mas
requer seam para RED determinístico.

**SPEC-GAP:** o prompt não define duração/frescura do cache, nem timeout para arquivo
especial. Cache por execução pode legitimamente congelar a primeira captura, desde que
isso seja declarado e a captura seja segura. O limite é exclusivo do prompt reader;
`compute_source_hash` não possui orçamento documentado.

## P4 — prompt walker: raiz válida, erro observável, set exato e symlink

### Fixture determinístico

Criar duas árvores equivalentes em ordens opostas com `.md`, não-`.md`, nested prompt,
exceção exata, nome prefixo/sufixo da exceção e symlinks interno/externo. Comparar o
conjunto de bytes dos paths, não a ordem de `HashSet`. Adicionar:

- `00_nucleo/prompts` ausente;
- `00_nucleo/prompts` como arquivo regular;
- raiz legível com subdiretório inacessível, executado como UID não privilegiado;
- `.md` com basename não UTF-8 e extensão ASCII;
- exceção `00_nucleo/prompts/a.md` versus `.../a.md.bak` e nested homônimo.

### Mutação e oráculo

Mata `.exists()` como validação de diretório, descarte `Err(_) => continue`, follow de
link e exceção por substring/basename.

**RED exato:** raiz que é arquivo retorna `Ok` vazio; symlink externo entra no set;
exceção remove path não idêntico; permutar criação muda o conjunto; path não UTF-8
colide/some sem `InvalidUtf8`; ou erro interno é indistinguível de scan completo. O
caso raiz-arquivo é candidato direto: `exists()` passa e WalkDir pode terminar sem
prompt, publicando `Ok(AllPrompts vazio)`.

**SPEC-GAP:** o prompt `prompt-walker` manda saltar erro individual silenciosamente,
enquanto o contrato `PromptProvider` diz que erros de diretório são propagados e o
assessment proíbe confundir erro com ausência. Essa contradição deve ser resolvida. A
API possui somente erro global, mas pode transportar o path interno como
`NucleoUnreadable`; silêncio não prova completude de V7.

Quanto à ordem: `AllPrompts.entries` é `HashSet`, portanto não promete bytes de
iteração canônicos. A propriedade congelada mais forte observável hoje é igualdade de
conjunto; exigir serialização ordenada seria evolução de contrato.

## P5 — snapshot exige seção, marcador único e schema fechado

### Fixture determinístico

Começar pelo snapshot canônico serializado e variar um fator:

- marcador em parágrafo, fenced code ou comentário sem seção;
- duas seções/marcadores, com JSONs iguais e depois conflitantes;
- marcador antes da seção canônica (isca first-match);
- comentário com prefixo/sufixo extra, duas estruturas JSON na linha ou braces em
  string;
- JSON truncado, duplicate keys, campo obrigatório ausente, tipo errado;
- campo top-level e campos nested desconhecidos;
- marker válido fora da última seção;
- path `..`, absoluto e symlink externo, reaproveitando P1.

### Mutação e oráculo

Mata `contains + find primeiro + primeiro { / último }`, serde permissivo a unknown
fields e ausência de contagem/posição de marcador.

**RED exato:** qualquer isca ou duplicata retorna `Some`; JSON com campo desconhecido
é aceito; path externo produz snapshot; ou marker canônico completo retorna `None`. O
extrator atual procura a primeira linha que apenas **contém** a frase, ignora seção e
unicidade; structs serde não usam `deny_unknown_fields`. São candidatos RED diretos.

**SPEC-GAP:** o contrato antigo diz `None` para “sem seção”, mas a implementação nem
verifica a seção. Ele também não define tratamento de duplicate keys, versão de schema
ou evolução compatível. O assessment congela schema completo/sem desconhecidos; uma
versão explícita seria preferível a fechamento implícito eterno.

## P6 — writes atômicos preservam alvo sob falha, concorrência e metadata

### Fixture determinístico

Para `write_hash` e `write_prompt_meta`:

1. arquivo com mode `0o640`, CRLF, sem newline final e bytes sentinela fora da linha;
2. digest válido de 8 hex; depois vazio, curto, longo, não-hex, whitespace e newline;
3. meta/header canônico e frases-isca em string/corpo; duplicatas canônicas;
4. diretório sem permissão de escrita e destino read-only, restaurando modes;
5. precriar o nome temporário previsto com conteúdo-sentinela;
6. duas threads/processos sincronizados escrevendo hashes diferentes no mesmo destino;
7. symlink/hardlink como destino e troca por barreira antes do rename.

Após cada operação, validar: ou sucesso integral autorizado, ou erro com destino
byte-idêntico; mode/owner preservados conforme contrato; nenhum temporário/resíduo;
somente a linha canônica mudou.

### Mutação e oráculo

Mata temporário fixo por PID, `fs::write` truncando temporário compartilhado,
`rename` sem preservar mode, digest não validado, substituição por `contains` e
reescrita via `lines/join`.

**RED exato:** digest inválido é gravado; frase-isca é substituída; CRLF ou bytes fora
da meta mudam; mode `0o640` vira mode do arquivo novo; concorrentes interferem pelo
mesmo temp; falha altera/trunca destino ou deixa `.crystalline-*-tmp-*`. Os writers
atuais usam nome temporário derivado somente do PID, não validam digest, normalizam
linhas e renomeiam arquivo recém-criado sobre o original, perdendo permissões — REDs
diretos.

**SPEC-GAP:** “preservar permissões restantes” está congelado, mas owner, ACL, xattrs,
timestamps e durabilidade (`fsync` de arquivo/diretório) não estão detalhados. Atomic
rename não implica crash durability. Concorrência precisa decidir entre serialização,
compare-and-swap da captura ou last-writer-wins; o que não pode ocorrer é corrupção ou
temporário compartilhado.

## Matriz priorizada

| Prioridade | Propriedade | Candidato principal |
|---|---|---|
| P0 | P1 confinement | `..`/absoluto/symlink lido; diretório conta em `exists` |
| P0 | P6 atomic write | temp por PID, mode perdido, digest livre, normalização de bytes |
| P0 | P2 hash | newline/CRLF invisíveis e linha-isca removida |
| P1 | P5 snapshot | marker-isca/duplicado e unknown fields aceitos |
| P1 | P4 walker | raiz `prompts` como arquivo retorna scan vazio; erros internos somem |
| P1 | P3 capture/cache | metadata/read TOCTOU; frescura do cache é SPEC-GAP |
