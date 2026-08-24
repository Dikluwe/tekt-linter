# Passo operacional 0072 — saneamento determinístico segregado

> **Natureza:** envelope operacional temporário para agentes; não é regra arquitetural
> **Estado:** escrito, não executado
> **Identidade:** P0072, continuação operacional do P0071
> **Branch:** `codex/segregated-materialization`
> **Base de evidência:** assessments 0001–0004 e respectivos gates RED congelados

## Objetivo

Fechar as pendências encontradas pela triagem retroativa antes de ampliar a busca. O
saneamento deve remover ordem incidental, ambiguidade silenciosa e proveniência externa
não declarada, sem transformar código legado em material historicamente `sealed`.

Ao final, todos os REDs legítimos deixam de ser ignorados, a suíte completa permanece
verde e um adversário independente confirma que as correções não apenas deslocaram a
ambiguidade para outra fronteira.

## Decisões congeladas

### D1 — `ProjectIndex::alien_files`

`alien_files` representa um conjunto canônico. Após `merge_local` e `merge`, deve estar
ordenado por path e sem duplicatas. Permutar ou reparticionar a redução produz o mesmo
índice observável.

Não confiar apenas numa ordenação posterior de diagnósticos: a entidade pública precisa
cumprir a comutatividade que documenta.

### D2 — ordem total de violações

`sort_violations` usa a chave total, nesta ordem:

1. severidade descendente;
2. path crescente;
3. linha crescente;
4. coluna crescente;
5. `rule_id` crescente;
6. mensagem crescente.

Duas violações integralmente iguais podem permanecer iguais. Qualquer outra diferença
tem desempate explícito, independente da ordem produzida pelo Rayon.

### D3 — registry sem vencedor incidental

Construção do `CrateRegistry` passa a ser falível e distingue erro estrutural de
workspace não Cargo:

- membro semanticamente idêntico repetido pode ser deduplicado;
- mesmo nome normalizado com definição diferente bloqueia;
- mesmo diretório canônico associado a membros diferentes bloqueia;
- colisões entre chaves como `foo-bar` e `foo_bar`, inclusive em renames, bloqueiam se
  suas definições não forem semanticamente idênticas;
- `member_layer` e `owner_of` nunca escolhem um vencedor pela ordem de entrada.

Introduzir erro tipado em L3 e propagá-lo por L4 como falha de infraestrutura. Não usar
panic, `unwrap_or_default` nem fallback para registry vazio em manifesto presente e
inválido. `Cargo.toml` realmente ausente continua significando projeto não Cargo.

### D4 — location canônica do inventário

Para cada módulo V22, a location é o menor path pela ordem nativa determinística de
`Path`, não o primeiro arquivo recebido. Contagem, filtros e percentual permanecem
inalterados.

### D5 — object database autocontido no refinamento selado

`seal-refinement` e a leitura B2 usada por ele não aceitam objetos fornecidos por
alternates ou por ambiente externo:

- limpar `GIT_OBJECT_DIRECTORY`, `GIT_ALTERNATE_OBJECT_DIRECTORIES`,
  `GIT_COMMON_DIR`, `GIT_DIR` e `GIT_WORK_TREE` dos subprocessos;
- detectar e bloquear `objects/info/alternates` não vazio antes de resolver refs;
- bloquear configuração equivalente que permita object store fora do repositório;
- não seguir alternates, não buscar e não executar helper;
- retornar exit `2`, sem selo parcial e sem alterar repositório ou object stores.

Este passo não promete suporte a worktree vinculada, bare repository ou object pool.
Esses formatos ficam fechados por padrão até contrato próprio. O erro deve dizer que o
modo exige object database autocontido; não alegar que o repositório é inválido para Git.

### D6 — paths não UTF-8

Não usar substituição lossy como identidade de path:

- texto humano preserva bytes inválidos por escape `\\xNN` em Unix;
- SARIF usa URI ASCII canônica com percent-encoding dos bytes que não podem aparecer
  literalmente; `%` é sempre codificado quando pertence ao path original;
- paths UTF-8 comuns mantêm saída legível e semanticamente equivalente;
- Windows recebe teste próprio sobre sua representação nativa; não inferir garantia
  entre plataformas apenas a partir do fixture Unix.

A mesma função de codificação deve alimentar todas as saídas de máquina que afirmem
identidade de arquivo. Não converter um path inválido em outro path válido aparente.

## Protocolo segregado obrigatório

O L0 deste passo e os assessments são congelados antes de produção mudar.

### Agente A — implementação

Recebe este passo, prompts vigentes e código de produção. Não pode ler:

- `tests/*_assessment.rs`;
- relatórios adversariais em `lab/assessment_*_adversarial.md`;
- output detalhado dos gates, além de recibos mínimos fornecidos pelo orquestrador.

Implementa D1–D6, testes unitários próprios e migrações de API necessárias.

### Agente B — verificação

Recebe este passo, assessments e testes congelados. Não pode ler produção modificada.
Remove `#[ignore]` dos casos legítimos, adapta somente mudanças públicas exigidas pelo
contrato e acrescenta fixtures para rejeições estruturais. Não relaxa expectativas.

### Agente C — adversário

Recebe L0 e a implementação somente após o primeiro gate. Não lê os testes de B.
Ataca desempates secundários, duplicatas semanticamente diferentes, normalização em
cadeia, paths hostis e formas alternativas de configurar object databases.

### Orquestrador

É o único papel que vê todos os recibos. Classifica cada RED como implementação, teste
ou contrato antes de devolver informação mínima ao produtor responsável. Nenhum agente
declara sozinho que o saneamento passou.

## Gate mecânico

### G1 — REDs existentes

Ativar e tornar verdes, sem alterar sua intenção:

- comutatividade integral de `ProjectIndex`;
- ordem total de violações sob permutações;
- lookup de membro duplicado e empate de owner;
- location V22 sob permutação;
- bloqueio de Git alternates;
- round-trip de path Unix não UTF-8.

### G2 — rejeições e atomicidade

Adicionar casos para:

- colisão normalizada em nome, dependency e rename;
- manifestos de workspace presentes porém inválidos;
- variáveis Git externas herdadas do processo pai;
- arquivo alternates vazio versus não vazio;
- falha mantendo destino de selo anterior byte-idêntico;
- mensagem de erro sem alegação falsa de isolamento comprovado.

### G3 — não regressão

Executar:

```bash
cargo test --workspace
cargo test --test low_risk_entities_assessment
cargo test --test shell_presentation_assessment
cargo test --test inventory_registry_assessment
cargo test --test git_refinement_assessment
cargo test --test segregated_materialization_cli
cargo run --quiet -- . --format text
git diff --check
```

O linter não pode introduzir V1, V5, V7 ou warning novo nas linhas alteradas. Atualizar
hashes de linhagem somente depois de o L0 final estar congelado.

### G4 — determinismo metamórfico

Para cada coleção tocada, gerar várias permutações e dois particionamentos de redução.
Comparar estruturas ou bytes finais, não apenas contagem ou exit code. Para cada erro de
registry/Git, repetir com ordem e ambiente diferentes e exigir a mesma classe de erro.

## Relatório obrigatório

Criar `00_nucleo/relatorio-p0072-saneamento-deterministico-segregado.md` contendo:

- commits L0, RED e implementação;
- recibos de isolamento dos três agentes;
- classificação de qualquer divergência intermediária;
- tabela D1–D6 com teste, resultado e limitação residual;
- resultados da suíte e auto-lint;
- lista explícita do que continua apenas `assessed`;
- recomendação sobre retomar ou não a triagem de baixo risco.

## Parada

Não fazer merge em `master`, instalar binário, publicar release ou marcar o legado como
`sealed`. Parar no branch após relatório, worktree limpo e gates verdes. Integração é
uma decisão posterior.
