# Passo operacional 0117 — impedir falso V1 em raiz absoluta ou simbólica

> **Estado:** PROPOSTO — pronto para execução
> **Defeito:** falha de confinamento do prompt é colapsada em `exists = false`
> **Regra preservada:** V1 significa exclusivamente header ausente ou prompt realmente ausente
> **Política preservada:** raízes e caminhos simbólicos continuam bloqueados de forma explícita
> **Predecessor:** P0116 / commit `4457695`

## 1. Reprodução congelada

O mesmo checkout físico é acessível por duas representações:

```text
/repos/Antigravity/tekt-linter
/home/dikluwe/Documentos/Antigravity/tekt-linter
```

`readlink -f` resolve ambas para `/repos/Antigravity/tekt-linter`. O binário instalado no
commit `4457695` produz:

```text
crystalline-lint /repos/Antigravity/tekt-linter --checks v1
=> V1 = 0

crystalline-lint /home/dikluwe/Documentos/Antigravity/tekt-linter --checks v1
=> V1 falso para todos os arquivos com @prompt
```

O caminho relativo `.` e o absoluto canônico não reproduzem o defeito. A condição causal
é uma raiz textual cujo ancestral é simbólico/alias, não a propriedade “absoluto” sozinha.

Antes da implementação, congelar:

- commit, binário instalado e hashes das duas representações;
- `readlink -f`, `stat` e componentes relevantes da raiz;
- saída text e SARIF para relativo, absoluto canônico e absoluto alias;
- número de arquivos analisados e contagem V1 em cada execução;
- testes atuais que rejeitam raiz, ancestral ou prompt simbólico.

## 2. Causa observável

A cadeia atual é:

```text
validate_local_root(root)
    -> Err("root component ... symlink")
confined_regular_file(root, prompt)
    -> Err(reason)
FsPromptReader::exists(prompt)
    -> false
parser.prompt_file_exists
    -> false
V1
    -> "prompt inexistente"
```

O erro não está na decisão de segurança de `validate_local_root`. O defeito é apagar a
classe do resultado e transformar qualquer erro em ausência.

É proibido corrigir com:

- `unwrap_or(false)` ou equivalente;
- `Path::exists()` sem confinamento;
- canonicalização silenciosa que contorne a política contra raízes simbólicas;
- supressão global de V1 quando o projeto usa caminho absoluto;
- comparação textual especial para `/home`, `/repos` ou qualquer prefixo local;
- warning V1 com mensagem diferente para erro de infraestrutura.

## 3. Semântica normativa

Introduzir um vocabulário fechado em L1:

```rust
pub enum PromptPresence {
    Present,
    Missing,
    InfrastructureFailure { reason: String },
}
```

Se for necessário distinguir classes estáveis sem transportar texto livre, usar enum
tipado mais razão:

```text
PromptAccessFailureKind =
    InvalidRoot
  | SymbolicRoot
  | UnsafeRelativePath
  | MissingAncestor
  | PermissionDenied
  | EscapesRoot
  | SymbolicPromptPath
  | NotRegularFile
  | Io
```

Tabela normativa:

| Header | Resolução | Resultado |
|---|---|---|
| ausente | não consultada | V1 |
| presente | `Present` | sem V1 |
| presente | `Missing` | V1 |
| presente | `InfrastructureFailure` | Fatal de infraestrutura, sem V1 |

`Unknown`, erro de I/O ou confinamento nunca equivalem a `Missing`.

## 4. Contrato PromptReader

Substituir a API booleana, evitando manter dois oráculos divergentes:

```rust
pub trait PromptReader {
    fn presence(&self, prompt_path: &str) -> PromptPresence;
    fn read_hash(&self, prompt_path: &str) -> PromptHashResolution;
}
```

`PromptHashResolution` deve preservar pelo menos:

```text
Hash(value)
Missing
InfrastructureFailure(reason)
```

Não deixar `read_hash() -> Option<String>` como segundo ponto que continue apagando
falhas. V5 só pode afirmar drift quando existem hash declarado e hash atual. Falha de
acesso ao hash é infraestrutura, não ausência silenciosa nem V5.

O cache deve armazenar o enum integral, incluindo a classe e a razão do erro. Não fazer
cache apenas de `bool`/`Option`. Resultados para identidades textuais diferentes não podem
colidir; chamadas repetidas para a mesma identidade retornam exatamente o mesmo estado.

## 5. Transporte pela IR

Substituir em `ParsedFile`:

```rust
prompt_file_exists: bool
```

por:

```rust
prompt_presence: PromptPresence
```

e atualizar `HasPromptFilesystem` para expor o enum completo.

Todos os nove parsers devem seguir a mesma ordem:

1. extrair o header;
2. não consultar reader quando o header está ausente;
3. consultar `presence()` exatamente uma vez quando presente;
4. consultar hash somente se `Present`;
5. preservar falha integralmente até a projeção de diagnóstico.

Rust, TypeScript, Python, C, C++, Zig, Go, Java e Elixir não podem implementar políticas
diferentes. Cobrir a composição `MultiParser` com spies de contagem.

## 6. Projeção de diagnóstico

### V1

V1 permanece regra L1 pura e reporta somente:

```text
MissingHeader
MissingPrompt(path)
```

Ela não formata, converte ou silencia `InfrastructureFailure`.

### Infraestrutura

Projetar falha de acesso como diagnóstico Fatal de infraestrutura, preferencialmente V0
com mensagem específica:

```text
Falha ao resolver prompt `<path>` sob raiz `<root>`: <kind>: <reason>
```

Se reutilizar V0 conflitar com sua semântica de source ilegível, criar identidade fechada
`PROMPT-IO`; não ocupar V28 e não reutilizar V1/V5/V7/V26.

O diagnóstico deve:

- preservar a classe de erro;
- apontar para o arquivo consumidor e o header quando a falha for específica do prompt;
- ser Fatal e produzir exit code não zero;
- nunca afirmar que o prompt não existe;
- ser deduplicado deterministicamente por `(root, prompt_path, failure_kind, reason)` para
  não gerar uma avalanche idêntica por arquivo consumidor.

Falha estrutural da própria raiz antes de qualquer prompt deve produzir exatamente um
Fatal global e impedir a pipeline, sem V1 derivados.

## 7. Política de raízes simbólicas

P0117 não altera silenciosamente a política vigente. O comportamento esperado para a raiz
alias da reprodução é:

```text
V1 = 0
Fatal de infraestrutura = 1
mensagem menciona raiz simbólica/alias ou componente não local
```

Uma decisão futura pode aceitar raiz simbólica canonicalizando-a uma única vez na
fronteira L4, mas isso exige passo próprio que substitua explicitamente os testes atuais
de rejeição. Aceitar alias não é necessário para corrigir a classificação falsa de V1.

O absoluto canônico deve continuar funcionando:

```text
V1 = 0
Fatal de infraestrutura = 0
```

## 8. Matriz obrigatória

### Raiz do projeto

| Caso | V1 | Infra | Observação |
|---|---:|---:|---|
| `.` relativo válido | 0 | 0 | controle |
| absoluto canônico válido | 0 | 0 | mesma saída semântica do relativo |
| absoluto com separadores redundantes normalizáveis | 0 | 0 | se aceito pela política |
| alias com ancestral simbólico | 0 | 1 Fatal | reprodução principal |
| raiz diretamente simbólica | 0 | 1 Fatal | política vigente |
| raiz inexistente | 0 | 1 Fatal | não vira projeto vazio |
| raiz sem permissão | 0 | 1 Fatal | plataforma Unix |
| raiz arquivo regular | 0 | 1 Fatal | não é diretório |

### Caminho do prompt

| Caso | Resultado |
|---|---|
| prompt regular presente | `Present` |
| arquivo realmente ausente sob raiz válida | `Missing` e exatamente um V1 |
| prompt é diretório | infraestrutura, não V1 |
| prompt é symlink interno | infraestrutura, não V1 |
| ancestral do prompt é symlink | infraestrutura, não V1 |
| `../`, absoluto ou componente inseguro no header | infraestrutura, não V1 |
| permissão negada | infraestrutura, não V1 |
| arquivo acima do budget | infraestrutura/hash failure, não Missing |
| Unicode e bytes de path válidos | identidade preservada |

### Header e hash

- header ausente não chama reader e gera somente V1;
- header presente + `Present` não gera V1;
- header presente + `Missing` gera somente V1;
- header presente + falha não gera V1 e produz Fatal;
- hash ausente porque prompt não existe mantém `Missing`;
- hash indisponível por I/O não vira ausência nem drift V5;
- cache preserva e repete cada variante sem nova leitura.

## 9. Teste black-box obrigatório

Criar fixture real mínima com:

```text
project/
  crystalline.toml
  00_nucleo/prompts/sample.md
  01_core/sample.rs
```

Executar o binário compilado, não função auxiliar:

```text
cwd externo + caminho relativo válido
cwd externo + absoluto canônico
cwd externo + alias simbólico
```

Comparar text e SARIF. Para relativo e absoluto canônico, normalizar somente o prefixo de
localização e exigir mesma cardinalidade, rule IDs, níveis e mensagens. Para alias:

```text
results[V1] = 0
results[infra Fatal] = 1
exit != 0
```

Adicionar regressão usando, quando disponível, as duas representações reais congeladas da
seção 1. A suíte portátil não pode depender dessas paths locais; usar `tempfile` e symlink
controlado como autoridade CI.

## 10. Mutantes obrigatórios

1. `InfrastructureFailure -> Missing`;
2. `Err(_) -> false` em `presence()`;
3. `read_hash` converte falha em `None`;
4. V1 aceita qualquer estado diferente de `Present`;
5. parser consulta hash apesar de presença falha;
6. parser sem header consulta reader;
7. cache apaga a razão do erro;
8. cache mistura `Missing` e falha;
9. L4 emite um Fatal por consumidor sem deduplicar;
10. raiz simbólica é canonicalizada silenciosamente;
11. caminho de prompt simbólico é aceito;
12. raiz inexistente produz zero achados e exit 0;
13. absoluto canônico é tratado como simbólico;
14. somente Rust recebe o novo contrato;
15. SARIF rotula a falha como V1.

Meta: 15/15 mutantes válidos mortos. `Unknown` não mata mutante de classificação.

## 11. Ordem de implementação

1. congelar reprodução e baseline integral;
2. escrever contrato fechado `PromptPresence`/`PromptHashResolution` em L1;
3. escrever fixtures e testes black-box RED;
4. alterar `PromptReader` e seu cache;
5. alterar `ParsedFile` e `HasPromptFilesystem`;
6. migrar os nove parsers;
7. restringir V1 a `Missing`;
8. projetar e deduplicar Fatal de infraestrutura em L4;
9. verificar V5 e fix/update para que nenhum apague falhas;
10. matar mutantes;
11. executar reprodução real pelas duas raízes;
12. rodar suíte, fmt, hashes, auto-lint e instalar somente após veredito.

## 12. Gates de aceitação

- relativo válido e absoluto canônico produzem V1=0;
- alias simbólico produz V1=0 e exatamente um Fatal de infraestrutura;
- prompt realmente ausente continua produzindo exatamente um V1;
- nenhum erro de confinamento, permissão ou I/O contém “prompt inexistente”;
- nove parsers preservam a mesma tabela de estados;
- `read_hash`, V5, `--fix-hashes` e `--update-snapshot` não apagam falhas;
- cache é determinístico e distingue todas as variantes;
- text e SARIF concordam em rule ID, nível e cardinalidade;
- 15/15 mutantes mortos;
- suíte integral, `cargo fmt --check`, `git diff --check`, auto-lint e dry-run de hashes
  passam;
- binário instalado só é atualizado depois de todos os gates.

Encerrar com exatamente um veredito:

- `IMPLEMENTED — V1 TRI-STATE FIX`; ou
- `SPEC-GAP — PROMPT ACCESS CLASSIFICATION`; ou
- `REJECTED — V1 PATH FIX`.

## 13. Evidência de fechamento

Materializar `lab/assessment_v1_absolute_path_fix.md` com:

1. baseline e comandos exatos da reprodução;
2. hashes/identidades das raízes textual e canônica;
3. contrato antes/depois;
4. tabela de migração dos nove parsers;
5. matriz fixture→estado→diagnóstico→exit code;
6. recibos text/SARIF do binário real;
7. tabela mutante→teste matador;
8. prova de ausência de falsos V1;
9. regressões V5/V7/V26 e operações de hash;
10. limitações de plataforma e atestação;
11. veredito único.

## 14. Fora de escopo

- aceitar automaticamente raízes simbólicas;
- enfraquecer confinamento de prompts;
- ocupar V28;
- mudar severidade de V1 verdadeiro;
- transformar falha de prompt em warning recuperável;
- corrigir paths de import dos parsers;
- modificar projetos consumidores durante a reprodução;
- instalar ou publicar binário antes do fechamento dos gates.
