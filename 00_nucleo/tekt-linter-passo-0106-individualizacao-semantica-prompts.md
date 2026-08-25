# Passo operacional 0106 — individualização semântica dos 13 prompts compartilhados

> **Natureza:** envelope operacional temporário; não cria nova regra arquitetural
> **Estado:** planejado; não executado
> **Branch:** `codex/tekt-nucleus-artifact`
> **Baseline:** `a7da834`
> **Dependências:** P0104 (bijeção) e P0105 (Núcleo Tekt)
> **Decisão humana:** todo código produtivo terá prompt proprietário próprio; somente
> claims genuinamente comuns podem migrar para Núcleo Tekt `.tekt`

## Objetivo

Eliminar os 13 compartilhamentos históricos do próprio `tekt-linter`, individualizando
semanticamente 44 relações `código → prompt` e criando os núcleos estritamente necessários:

```text
antes                              depois

prompt amplo ──> código A          núcleo opcional ─┬─> prompt A ⇄ código A
             └─> código B                          └─> prompt B ⇄ código B
```

P0106 fecha simultaneamente os bloqueios herdados:

- V15 global: 13 → 0;
- prompts proprietários: 45 → pelo menos 76 distintos no domínio observado;
- consumers sob ownership compartilhado: 44 → 0;
- `--fix-hashes`: desbloqueado somente depois do último grupo;
- novos headers P0104/P0105: resselados pelo fluxo oficial;
- auto-lint V1/V5/V7/V15/V26 integralmente verde.

O horizonte é finito: 13 documentos, 44 códigos e no mínimo 31 novos prompts. O passo
revisa também os 13 prompts preservados; não considera suficiente apenas mudar 31 headers.

## Não objetivo

- não alterar comportamento Rust, API pública, parser ou regras do linter;
- não modificar o formato `.tekt`, V15, V26 ou `fix-hashes`;
- não criar núcleo para justificar artificialmente todo compartilhamento;
- não copiar integralmente um prompt amplo para vários arquivos;
- não mover ADRs, passos, relatórios ou diagnóstico para `.tekt`;
- não modificar Typst Crystalline, Bateia ou `tekt-cargo-dsm`;
- não fazer merge, push, release ou instalar o binário.

Se a leitura semântica revelar mudança desejável em produção, registrar novo passo; P0106
edita somente L0, referências de lineage e metadata calculada.

## Baseline normativo

O Assessment 0034 deve fixar por SHA-256:

- P0104, Assessment 0032 e inventário 0032-A;
- ADR-0022, P0105, Assessment 0033 e relatório P0105;
- os 13 prompts compartilhados atuais;
- os 44 códigos consumidores, somente para provar identidade e responsabilidade observada;
- V15, V26, parser de núcleo, prompt reader e reparador atuais.

Qualquer divergência antes de A bloqueia. Não resselar para esconder drift de entrada.

## Invariantes de migração

1. cada código termina com exatamente um `@prompt` Markdown;
2. cada prompt proprietário termina com exatamente um código produtivo;
3. `.tekt` nunca aparece em `@prompt` de código;
4. núcleo não lista consumers e não contém `Hash do Código`;
5. prompt proprietário continua inteligível isoladamente quanto à responsabilidade do owner;
6. claim retirada do prompt só pode ir a núcleo se dois ou mais owners realmente dependem
   dela e os gates demonstram essa dependência;
7. contexto útil mas não normativo pode ser resumido ou citado, não copiado como obrigação;
8. histórico de passo, diagnóstico datado e medição não viram claim perene;
9. nomes de prompt derivam da responsabilidade, não apenas do filename;
10. nenhuma etapa escolhe owner pela ordem do filesystem ou pelo prompt antigo.

## Classificação obrigatória por trecho

Cada um dos 13 prompts deve ser decomposto em uma tabela congelada antes de edição:

| Classe | Destino permitido |
|---|---|
| `OWNER:<consumer>` | prompt proprietário daquele consumer |
| `SHARED-CLAIM` | candidato a `.tekt`, com id/modalidade e pelo menos dois prompts consumers |
| `CONTEXT` | resumo/citação opcional nos prompts; não entra no hash transitivo como núcleo |
| `HISTORY` | relatório/diagnóstico ou remoção do contrato ativo |
| `CONTRADICTION` | `SPEC-GAP`; bloquear o grupo até arbitragem |
| `UNSUPPORTED` | não materializar; registrar ausência de autoridade |

Uma seção inteira não pode ser marcada `SHARED-CLAIM` sem atomização em claims. Claims
`may` não servem para esconder indecisão entre dois comportamentos incompatíveis.

## Manifesto integral

A produz `00_nucleo/assessments/0034-manifest-individualizacao.tsv`, ordenado por bytes,
com uma linha por consumer:

```text
old_prompt<TAB>consumer<TAB>owner_prompt<TAB>nuclei_csv<TAB>classification_sha256<TAB>state
```

Regras:

- exatamente 44 linhas de dados;
- 44 `consumer` distintos e 44 `owner_prompt` distintos;
- cada consumer existe no inventário 0032-A;
- `nuclei_csv` pode ser vazio;
- todo núcleo listado possui pelo menos dois prompts consumers no manifesto, salvo núcleo
  que dependa de outro núcleo e tenha justificativa explícita;
- `classification_sha256` fixa a tabela semântica do grupo;
- estados permitidos antes de C: `CLASSIFIED`, `SPEC-GAP`;
- C só começa quando as 44 linhas estiverem `CLASSIFIED`.

## Lotes semânticos

### Lote 1 — pares de fronteira e suporte localizado

Sete grupos, 14 consumers:

1. `contracts/citation-freshness.md`;
2. `contracts/prompt-reader.md`;
3. `contracts/prompt-snapshot-reader.md`;
4. `rules/external-type-in-contract.md`;
5. `sarif-formatter.md`;
6. `segregated-materialization.md`;
7. `unsourced-constant.md`.

Priorizar separação port/adaptor, entity/rule e formatter/path. Não presumir que “mesmo
fluxo vertical” implica núcleo: interfaces e adapters têm responsabilidades diferentes.

### Lote 2 — famílias internas

Quatro grupos, 15 consumers:

1. `violation-types.md` — 4;
2. `file-walker.md` — 2;
3. `fix-hashes.md` — 4;
4. `rules/wildcard-saturation.md` — 5.

Aqui núcleos são plausíveis para invariantes de diagnóstico, confinamento, transação e
semântica de padrões. Ainda assim, algoritmo de uma regra específica permanece no prompt
da regra, não no núcleo familiar.

### Lote 3 — subsistemas complexos

Dois grupos, 15 consumers:

1. `refinement-validator.md` — 5;
2. `linter-core.md` — 10.

Executar por último. `linter-core.md` não pode virar um “núcleo universal”. Claims só
entram em `.tekt` quando possuem consumers nominais e gate de dependência. Facades `mod.rs`,
config, parsers, summary e wiring recebem contratos proprietários distintos.

## Protocolo segregado

### A1 — inventário e pinagem, somente leitura

Criar Assessment 0034 e manifesto integral. Recalcular:

- 13 prompts compartilhados;
- 44 consumers afetados;
- 32 pares previamente únicos;
- mínimo 31 prompts novos;
- zero V26 no baseline P0105;
- mesmos 13 V15, sem regressão de contagem.

Nenhuma edição de prompts/headers nesta fase.

### A2 — classificadores semânticos por grupo

Criar um arquivo em `00_nucleo/assessments/0034-groups/` para cada prompt antigo. Cada
classificador recebe:

- prompt antigo hash-pinned;
- lista nominal de consumers;
- símbolos/responsabilidades observáveis de cada consumer;
- ADRs diretamente aplicáveis, também hash-pinned;
- proibição de escrever L0 ou produção.

Saída: tabela trecho→classe, prompts proprietários propostos, claims candidatas e gaps.

O classificador não pode usar o texto de um prompt novo como justificativa retroativa.

### B1 — gate estrutural do manifesto

Criar `tests/prompt_individualization_manifest_assessment.rs` e provar:

- cardinalidades 13/44/44;
- nenhuma duplicata de consumer ou owner prompt;
- nenhum consumer fora do domínio;
- paths lógicos, case-sensitive e ordenados;
- classificação existente e hash correto para toda linha;
- núcleos referenciados respeitam multiplicidade e namespace;
- permutação do input produz o mesmo inventário normalizado.

### B2 — gate semântico por lote

Antes de materializar cada lote, um verificador recebe classificadores e L0 propostos, mas
não os prompts antigos sem a tabela autorizada. Ele confronta:

- toda responsabilidade observada tem owner explícito;
- nenhuma claim muda modalidade ou alcance;
- núcleo possui dois ou mais consumers justificados;
- prompts proprietários não contêm obrigações exclusivas de outro consumer;
- `HISTORY/CONTEXT` não reaparecem como contrato normativo;
- não há cópia substancial idêntica entre prompts, exceto bloco estrutural/template.

Resultado por arquivo: `PASS`, `RED-GATE`, `RED-L0` ou `SPEC-GAP`. Qualquer `SPEC-GAP`
bloqueia somente o grupo; não autoriza pular para o resselo final.

### B3 — gate do grafo projetado

Criar fixture/in-memory com os 44 pares e núcleos propostos. Antes de editar headers, provar:

- V15 projetada = 0;
- V7 projetada = 0;
- V26 projetada = 0;
- todos os 44 códigos têm um owner;
- todos os owner prompts têm um código;
- DAG sem ciclo/missing/orphan;
- mudança em cada núcleo invalida exatamente seus prompts transitivos declarados.

### C — materialização documental por lote

Somente após A/B congelados:

1. criar `.tekt` do lote, se houver claims aprovadas;
2. criar ou reescrever prompts proprietários;
3. atualizar somente `@prompt` nos códigos do lote;
4. usar hash placeholder explicitamente inválido até o resselo final;
5. atualizar manifesto para `MATERIALIZED`;
6. rodar gates semânticos e V15 dirigida;
7. congelar o lote em commit próprio.

Não executar `--fix-hashes` entre lotes: P0104 deve bloquear enquanto existir qualquer
ownership compartilhado. Esse bloqueio é evidência positiva de atomicidade global.

Critério após cada lote:

| Marco | V15 esperada |
|---|---:|
| baseline | 13 |
| lote 1 | 6 |
| lote 2 | 2 |
| lote 3 | 0 |

Contagem diferente exige inventário causal; não ajustar expectativa automaticamente.

### D — resselo atômico final

Somente com V15=0 e V26 estrutural=0:

1. registrar hash do worktree e plano esperado;
2. executar `--fix-hashes --dry-run` e exigir exatamente todos os pares stale, sem
   `Unavailable`;
3. executar `--fix-hashes` uma única vez;
4. repetir dry-run e exigir `Nothing to fix`;
5. validar pins de núcleos, hashes efetivos, `Hash do Código` e headers nos dois sentidos;
6. provar que nenhum arquivo fora do manifesto/metadata causal foi alterado;
7. congelar o resselo em commit separado.

Falha em qualquer write exige rollback integral. Não corrigir hashes manualmente.

### E — adversário final

Confrontar:

- prompt antigo preservado como owner arbitrário sem reescrita;
- núcleo criado por cópia integral do prompt antigo;
- claim exclusiva promovida a compartilhada;
- claim comum duplicada em vez de referenciada;
- consumer omitido ou owner prompt órfão;
- path/header atualizado para prompt errado mas hash consistente;
- V15 escondida por exclusão/config/checks;
- `.tekt` usado diretamente por código;
- núcleo órfão, ciclo ou pin transitivo stale;
- reparador executado antes do último lote;
- segunda passagem usando cache antigo;
- delta funcional Rust disfarçado de lineage;
- alteração em projeto externo.

## Política para arquivos antigos

Cada um dos 13 `.md` antigos recebe uma decisão explícita:

- **REWRITE-AS-OWNER:** permanece no mesmo path, mas é reescrito para seu único owner;
- **MOVE-TO-OWNER:** renomeado para path proprietário e referências atualizadas;
- **RETIRE:** nenhum owner legítimo; conteúdo classificado foi distribuído/nuclearizado e
  o arquivo é removido, provando V7=0.

Não manter prompt amplo órfão em whitelist. `orphan_exceptions` não pode crescer em P0106.

## Regressões obrigatórias

- gates P0104 B1/B2/B3;
- gates P0105 B1–B5;
- gates P0106 B1–B3 e verificadores de cada lote;
- suíte completa do workspace;
- prompt/nucleus walker, reader, hashing e confinamento;
- fix-hashes planning/execution/rollback/segunda passagem;
- CLI text/SARIF V0–V26;
- auto-lint V1/V5/V7/V15/V26;
- `--fix-hashes --dry-run` final idempotente;
- `rustfmt --check` apenas em Rust tocado — idealmente somente headers;
- `git diff --check`;
- hashes/status dos projetos externos inalterados.

## Saídas esperadas

- Assessment 0034;
- 13 classificadores semânticos;
- manifesto integral de 44 linhas;
- gates B1–B3 e evidência por lote;
- prompts proprietários 1:1 para os 44 consumers;
- zero ou mais Núcleos Tekt semanticamente justificados;
- commits separados para lotes 1, 2, 3 e resselo;
- V1/V5/V7/V15/V26 verdes;
- `00_nucleo/relatorio-p0106-individualizacao-semantica-prompts.md`;
- fechamento `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

P0106 não autoriza integração ou instalação. Depois de fechamento adversarial verde, um
passo curto próprio deve fazer merge do branch, reinstalar o linter do sistema e repetir
smoke tests somente leitura em Bateia e Typst Crystalline. A migração dos 22 prompts
compartilhados do Typst continua sendo passo separado, posterior à atualização do binário.
