# Passo operacional 0104 — propriedade biunívoca entre prompts e código

> **Natureza:** envelope operacional temporário; não é regra arquitetural permanente
> **Estado:** planejado; não executado
> **Branch:** `codex/v15-bijective-prompt-ownership`
> **Baseline:** `84fa3006ad6557722cfbe4d10c78c7d0de6b4195`
> **Decisão humana:** cada prompt proprietário governa exatamente um código e cada código
> possui exatamente um prompt proprietário

## Objetivo

Completar a invariância de linhagem 1:1 que hoje existe apenas na direção local:

```text
V1  — código possui prompt válido
V15 — código não possui múltiplos prompts
V15 — NOVO: prompt proprietário não possui múltiplos códigos
V7  — prompt proprietário não fica órfão
```

Em conjunto, V1 + V7 + V15 local + V15 global formam a bijeção:

```text
código A ⇄ prompt A
código B ⇄ prompt B
```

P0104 também impede `--fix-hashes` de iniciar qualquer escrita quando o inventário não é
biunívoco. Não implementar metadata plural nem aceitar “último consumer vence”.

## Não objetivo

- não migrar o `typst-crystalline` neste passo;
- não executar a migração global P1179;
- não criar `@contract`, `@inherits` ou outra relação compartilhada;
- não duplicar automaticamente conteúdo normativo para fabricar prompts proprietários;
- não alterar V5/V7/V15 para acomodar o estado atual dos repositórios;
- não interpretar prompt compartilhado como exceção legítima.

Documentos arquiteturais compartilhados poderão receber uma relação distinta em passo
futuro. Enquanto essa relação não existir, `@prompt` significa propriedade exclusiva.

## Insumos hash-pinned

| Unidade | Caminho | SHA-256 |
|---|---|---|
| contrato V15 | `00_nucleo/prompts/rules/multi-prompt-header.md` | `81ba0f080eac8c2db78f27f04f206ff746eecdd358fdb55b146523192704f053` |
| produção V15 | `01_core/rules/multi_prompt_header.rs` | `123f2ab3da2c130ae47d624b731327ec857d8b752399c1b744a8d48d7d86a400` |
| contrato fix-hashes | `00_nucleo/prompts/fix-hashes.md` | `d6cc361ed70301c002717b6e80a6c166a0ba1f149084c0f3000c373ba5d1daf9` |
| produção fix-hashes | `02_shell/fix_hashes.rs` | `26252ef696b1026168568e992a10b41a25b9848bd94e1ab0fa01403288ea3278` |
| arquitetura Tekt | `00_nucleo/prompts/linter-core.md` | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` |
| índice atual | `01_core/entities/project_index.rs` | `9bf8d5e772761347c52f628d9a0cde57d1a4dbd931dcb5e66968e6558e62aa91` |
| wiring atual | `04_wiring/main.rs` | `c64134adb944798050d2088921334368dde1c49be6e9f119871342a12217f2b5` |
| diagnóstico P1179 | `/repos/Antigravity/typst-crystalline/00_nucleo/diagnosticos/typst-p1179-auditoria-migracao-hashes-linter.md` | `3fee15dbe9c3610a2104f4523cac79039c368737a8f7cb8aafd9c5adc5d95e60` |
| manifesto P1179 | `/repos/Antigravity/typst-crystalline/00_nucleo/diagnosticos/typst-p1179-manifest-hashes-linter.tsv` | `67f3ec296c9e8bf54891b1c1a32cd323d8509c6957767a3f87e0135622315a6a` |

O Assessment 0032 recalcula os hashes antes de qualquer gate. O repositório externo é
somente evidência de diagnóstico; P0104 não escreve nele.

## Fronteiras Tekt

- L3 extrai referências de prompt e percorre arquivos; não decide cardinalidade.
- L4 agrega resultados e injeta uma visão integral e determinística.
- L1 recebe dados já extraídos e decide puramente a bijeção.
- L2 planeja/apresenta `--fix-hashes`, sem filesystem.
- L3 implementa escrita/rollback através de porta L2; L4 compõe.
- nenhuma regra L1 abre prompts, consulta paths reais ou mantém estado global.

A reciprocidade não pode ser implementada chamando `check(file)` isoladamente para cada
arquivo. Ela exige coleção integral antes da redução.

## Semântica normativa

### Domínio

Participam arquivos de produção classificados como L1, L2, L3 ou L4 e sua referência
canônica `@prompt` extraída do doc-header válido. Testes/fixtures seguem a política já
vigente de origem; Lab, L0 e Unknown não criam ownership produtivo.

V1 continua dona de ausência/header inválido. V15 não converte ausência em colisão.

### Identidade

- identidade de código é o path lógico integral preservado pelo walker;
- identidade de prompt é o valor integral e case-sensitive de `@prompt`;
- não canonicalizar textual ou fisicamente em L1;
- entradas duplicadas do mesmo arquivo não aumentam cardinalidade, mas conflito de
  conteúdo para a mesma identidade é erro de infraestrutura a montante;
- ordem de filesystem, parser e Rayon não altera resultado.

### Diagnóstico global V15

Para cada prompt com dois ou mais códigos distintos, emitir exatamente uma V15 Error:

- localização: primeiro path por ordem lexical de bytes, linha 1, coluna 0;
- mensagem: prompt integral, cardinalidade e lista integral de consumers em ordem
  lexical;
- sem truncar, escolher owner ou sugerir que o último consumer é canônico;
- ordem global: prompt por bytes, seguida da ordem total normal de diagnósticos.

A V15 local existente continua emitindo uma violação por arquivo com 2+ headers.

## Protocolo segregado

### A — inventário integral, somente leitura

A produz `00_nucleo/assessments/0032-a-inventario-ownership-prompts.md`:

- mapa `prompt → consumers` do `tekt-linter`, obtido pela mesma IR produtiva;
- contagem de prompts únicos, compartilhados, órfãos e consumers ambíguos;
- matriz de parsers/linguagens que realmente popula a referência canônica;
- pontos de Map/Reduce atuais e seam mínima de agregação;
- classificação dos compartilhamentos existentes, sem corrigi-los;
- análise da quantidade de prompts que precisariam ser individualizados antes de o
  próprio linter ficar verde.

A não lê o corpo das regras V15/fix-hashes depois de o assessment causal congelar os
símbolos permitidos, não edita produção e não inventa exceções.

### B1 — gate cego da bijeção

B1 cria exclusivamente `tests/prompt_ownership_bijection_assessment.rs`. Usa entidades
in-memory, sem filesystem, e confronta:

1. bijeção vazia e pares 1:1;
2. um código com dois prompts — preserva V15 local;
3. dois, três e muitos códigos para um prompt — uma V15 global;
4. duplicata idêntica do mesmo par — cardinalidade não aumenta;
5. dois prompts textualmente próximos permanecem distintos;
6. permutações de arquivos/prompts produzem bytes idênticos;
7. paths Unicode, case-sensitive, vazios hostis já classificados a montante e extremos
   não causam escolha implícita de owner;
8. L0/Lab/Unknown não entram no ownership produtivo.

O gate não importa produção V15 para fabricar o oráculo. Expectativas vêm do L0
hash-pinned.

### B2 — gate do consumidor real

B2 cria `tests/prompt_ownership_wiring_assessment.rs` e fixture exclusiva. Executa o
binário sobre projetos controlados e prova:

- dois arquivos individualmente válidos apontando ao mesmo prompt geram V15;
- mensagem lista ambos e resultado independe da ordem de criação;
- V1/V7 continuam distintos de V15;
- exclusões e origem de teste não criam ownership fantasma;
- todos os parsers que publicam prompt canônico participam com a mesma semântica;
- nenhuma leitura extra ocorre dentro da regra L1.

### B3 — gate transacional de fix-hashes

B3 cria `tests/fix_hashes_bijection_assessment.rs` com spies independentes. Deve provar:

- qualquer prompt compartilhado bloqueia o lote inteiro antes da primeira escrita;
- múltiplas colisões são apresentadas integralmente e deterministicamente;
- metadata ausente é inserida somente para par 1:1 ou bloqueada no preflight por decisão
  explícita; nunca ocorre `PartialWrite` silencioso;
- falha de preflight em um par impede escrita em todos os pares;
- falha durante aplicação restaura o estado anterior de código e prompt ou retorna erro
  fatal observável sem declarar o lote concluído;
- segunda passagem valida `@prompt-hash` e `Hash do Código:` nos dois sentidos;
- dry-run e execução usam o mesmo plano integral.

B3 deve reproduzir minimamente o falso fechamento P1179 antes de C.

### B4 — inventário externo congelado

B4 não escreve no Typst Crystalline. Revalida o manifesto P1179 e confirma:

- 421 consumers e 336 prompts;
- 22 prompts compartilhados por 107 consumers;
- 23 prompts sem metadata afetando 78 consumers;
- os seis tracked paths preexistentes permanecem byte a byte iguais;
- dry-run não altera worktree.

Divergência de contagem é nova evidência, não autorização para atualizar o manifesto.

## C — correção do linter

Somente após A e B1–B4 congelados:

1. atualizar o contrato V15 de “um arquivo, um prompt” para propriedade biunívoca;
2. criar entidade/visão L1 mínima para ownership integral, reutilizando IR existente sem
   transportar filesystem;
3. adicionar verificação global V15 determinística;
4. ligar a redução em L4 sem duplicar V1/V7 ou parsing;
5. tornar o planejamento `fix-hashes` integral e agrupado por prompt;
6. rejeitar ownership não biunívoco e preflight incompleto antes de qualquer escrita;
7. implementar inserção segura de metadata ausente ou congelar `SPEC-GAP` se a posição
   canônica não estiver decidida;
8. impedir falso `Nothing to fix` validando as duas direções;
9. resselar hashes somente pelo fluxo oficial.

Não alterar expectativas dos gates depois de abrir produção.

## D — saneamento do próprio tekt-linter

A nova V15 pode tornar o próprio repositório vermelho. Cada prompt compartilhado deve
ser tratado em commit posterior à regra:

- criar prompt proprietário específico para cada código;
- mover/copiar somente conteúdo realmente proprietário após confronto semântico;
- não introduzir exceção, whitelist ou metadata plural;
- manter documentos comuns fora de `@prompt` até existir relação compartilhada própria;
- resselar cada novo par e provar V1/V5/V7/V15 verdes.

Se a individualização exigir decidir herança/contrato compartilhado, classificar
`SPEC-GAP` e terminar `BLOCKED`; não duplicar L0 mecanicamente.

## E — adversário final

E confronta:

- colisão perdida por partição/paralelismo;
- owner escolhido pelo primeiro arquivo observado;
- normalização case/path escondendo ou fundindo identidades;
- V15 global omitida em algum parser;
- `fix-hashes` escrevendo antes do preflight integral;
- rollback parcial apresentado como sucesso;
- segunda passagem que verifica somente V5 direta;
- prompt compartilhado do próprio linter escondido por exclusão;
- alterações no Typst Crystalline durante P0104.

## Regressões obrigatórias

- B1–B4 novos;
- gates V1/V5/V7/V15 existentes;
- planning/execution/presentation de fix-hashes;
- prompt walker, prompt reader/io e project index;
- fixtures multi-parser;
- suíte completa do workspace;
- auto-lint V1/V5/V7/V15 e reparador dry-run;
- `rustfmt --check` somente nos Rust tocados e `git diff --check`.

## Saídas esperadas

- Assessment 0032 e inventário A;
- três gates segregados e revalidação B4;
- RED inicial reproduzível;
- V15 biunívoca e fix-hashes fail-before-write;
- saneamento do `tekt-linter` ou `SPEC-GAP` explícito;
- `00_nucleo/relatorio-p0104-propriedade-biunivoca-prompts.md`;
- fechamento `READY WITH RESIDUAL AUDIT` ou `BLOCKED`.

P0104 não autoriza modificar o Typst Crystalline, aplicar P1179, criar metadata plural,
introduzir relação compartilhada, fazer merge/push/release ou instalar o binário. Depois
de P0104 integrado e reinstalado, o Typst Crystalline recebe passo próprio para dividir
os 22 prompts compartilhados e repetir P1179 desde o dry-run.
