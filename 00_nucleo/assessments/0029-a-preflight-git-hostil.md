# Assessment 0029/A — preflight normativo e testabilidade Git hostil

**Papel:** A — adversário L0 e de testabilidade  
**Data:** 2026-08-25  
**Leitura autorizada:** Assessment 0029 e seus quatorze insumos hash-pinned  
**Produção lida:** não  
**Veredito:** `SPEC-GAP` bloqueante — B1/B2 proibidos até saneamento e resselamento

## Integridade dos insumos

Os quatorze SHA-256 foram recalculados e coincidem byte a byte com o Assessment 0029:

| Unidade | SHA-256 recalculado | Resultado |
|---|---|---|
| protocolo P0100 | `e85740fb3057b030c0a328e32dbd70e1ed3b36bc7bdde2c618dda4169337da06` | `PASS` |
| contrato de refinamento | `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97` | `PASS` |
| ADR B2 | `088e5806c948d60c2f5b1ea2c04c4b181672c037c31f53c0b125ddf594a497d6` | `PASS` |
| arquitetura Tekt | `9027da3f425bd3a70bcb776de52e5f2703989a04a47d5ff52264795aa7a6d0a0` | `PASS` |
| protocolo segregado | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` | `PASS` |
| ADR segregado | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` | `PASS` |
| Assessment F04 | `46b1bcec486c8e909fe1bc66a36e9e0a9b7c91d992ead6f8fd20f53fd73b4ba2` | `PASS` |
| decisão F04 | `1fa048e01935717806ef48ae6ea74cda62cc2f26b6807ee2d806f8176adf9f06` | `PASS` |
| fechamento P0099 | `15d0d757414358f94f073b7084c1b0d057c834986343148c0dad56fb8f854588` | `PASS` |
| Assessment Git histórico | `5a38f20563a865a12dc0c052a2b7a5dd0d46cb17452600c183c8781bce8a5d17` | `PASS` |
| fechamento P0072 | `d43d1dd6e9d356b0f3dcd652a57c02cf7af0d24ecf24fb8c6d419d7b2a393fb7` | `PASS` |
| gate Git real histórico | `9609ebdb84d21fb79cddd744392d9fb8692513c809bf651c52eefa1c8b75c434` | `PASS` |
| inventário estrutural | `fac7a67068e6f63a969f3725710026afed3f828275859e8f49cafb6a1ec914e2` | `PASS` |
| inventário normativo | `3f4fee9273c72ca0202e9f2ab95e551f7f53e55c7862493ba34660a148f94e3e` | `PASS` |

O gate histórico foi usado somente como evidência retroativa da matriz já confrontada;
seus exits e sua implementação cliente não foram usados como autoridade normativa.

## Classificação dos doze itens de preflight

| # | Item | Classe | Fundamentação L0 |
|---:|---|---|---|
| 1 | comandos, ordem, argumentos e delimitadores | `SPEC-GAP` | O ADR nomeia `rev-parse --verify --end-of-options`, `ls-tree -z` e `cat-file --batch-command --buffer`, mas não congela argv completo, diretório de trabalho, ordem/número de processos, separador de path, comandos enviados ao batch nem ciclo de flush/encerramento. |
| 2 | ambiente removido/forçado e `PATH`/executável | `SPEC-GAP` | Quatro variáveis forçadas e config global/sistema neutralizada são requisitos, porém não há allowlist/denylist completa, valores de `HOME`/`XDG_CONFIG_HOME`, política de `PATH`, localização/injeção do executável ou conjunto exato de `-c`. |
| 3 | sintaxe de ref e separação option/pathspec | `SPEC-GAP` | Git deve validar a ref e `--end-of-options` é exigido, mas não se decide vazio/NUL/newline/bytes não UTF-8, sufixo `^{commit}` como argumento separado ou concatenado, nem gramática de paths lógicos e pathspec magic. |
| 4 | resolução única e OID opaco | `PASS` | Prompt, ADR e decisão F04 exigem uma resolução por ref para commit e somente OIDs imutáveis posteriores, opacos a SHA-1/SHA-256. |
| 5 | blob/ausências/symlink/gitlink | `PASS` | Somente blob regular; ausência real segue `on_missing`; objeto esperado ausente/ilegível, modo `120000`, modo `160000` e tipo inesperado são erro/inconclusão, nunca ausência conhecida. |
| 6 | framing `ls-tree`/batch | `SPEC-GAP` | NUL, tipo, modo, tamanho e framing devem ser validados e truncamento deve falhar, mas a gramática byte-level completa de requisição/resposta, newline, duplicatas, ordem, tamanho decimal, stderr e bytes não UTF-8 não foi publicada. |
| 7 | budgets e publicação | `SPEC-GAP` | Os três limites e a proibição de truncamento estão fixados; falta definir se 512 conta paths declarados, únicos ou por revisão, quando duplicatas contam, overhead/soma exata e precedência entre budget e outras falhas. |
| 8 | timeout, kill, reap e descendentes | `SPEC-GAP` | Há 10 s por operação, kill/reap do processo e proibição de resultado parcial, mas “operação” não está delimitada e não há contrato de grupo/job, descendentes, grace period, drenagem/fecho de pipes ou resultado quando kill/reap falha. |
| 9 | neutralização de efeitos/configuração | `RED` normativo | A autocontenção fechada pelo P0072 bloqueia alternates/object stores externos, enquanto a adenda B2 ainda afirma que alternates ficam a cargo da leitura local do Git. Além disso, hooks/protocolos/configs são proibidos sem argv/ambiente exatos que constituam a neutralização verificável. |
| 10 | API/porta de processo hostil | `SPEC-GAP` | O L0 publica apenas porta abstrata de conteúdo em L1. Não publica símbolo, assinatura ou mecanismo L3 para escolher um executável controlado e atravessar o adapter real sem depender de `PATH` herdado ou da produção como expectativa. |
| 11 | equivalência B2/B1 e OID | `PASS` | Mesmos bytes alimentam explicitamente o mesmo extrator B1; OIDs resolvidos participam de `artifact_id` e testemunhas; comparador/normalizador paralelo é proibido. |
| 12 | taxonomia entrada versus `Unknown` | `SPEC-GAP` | Ref inexistente é erro de entrada e budget admite `Unknown(BudgetExhausted)` ou erro; para objeto, tipo, framing e leitura o L0 permite alternativamente erro ou inconclusão, sem classes/prefixos ou mapeamento exato. |

Resultado: três `PASS`, oito `SPEC-GAP` e um `RED` normativo. O item 9 exige
reconciliação; os itens 1, 2, 3, 6, 7, 8, 10 e 12 não sustentam expectativas únicas.

## Classificação das treze alegações candidatas

| # | Alegação | Classe | Fundamentação L0 |
|---:|---|---|---|
| 1 | uma resolução por ref e uso posterior só do OID | `PASS` | Requisito inequívoco e observável pelo processo controlado após a publicação da seam. |
| 2 | OID opaco, sem comprimento SHA-1 | `PASS` | ADR exige SHA-1/SHA-256 opacos e validação pelo Git. |
| 3 | estado do repositório inalterado em todo resultado | `PASS` | Prompt, ADR e F04 proíbem mutação de working tree, índice, HEAD, branch, refs e stash, inclusive em erro. |
| 4 | origem não ampliada por `.git` indireto/alternates/ambiente | `RED` normativo | Autocontenção P0072 e F04 exigem bloqueio; a adenda vigente permite alternates sob leitura local. O gate não pode escolher uma das duas autoridades. |
| 5 | argv sem shell e entradas hostis não reinterpretadas | `SPEC-GAP` | A regra geral é clara, mas argv e gramáticas exatas ainda não permitem provar opção/pathspec/configuração para toda entrada prometida. |
| 6 | ambiente desabilita efeitos externos | `SPEC-GAP` | Intenção e quatro variáveis estão fixadas; conjunto exato de ambiente/configuração não está. |
| 7 | nenhum fetch/checkout/worktree/stash/build/filter/textconv/LFS/submódulo | `PASS` | A proibição é nominal e consistente no envelope confirmado. |
| 8 | timeout encerra/reap sem bloqueio nem publicação parcial | `SPEC-GAP` | Falta a semântica de descendentes, grupos/jobs, pipes e falha de encerramento. |
| 9 | somente blob regular; tipos proibidos não são ausência | `PASS` | Tipo/mode e projeção de ausência são decididos nominalmente. |
| 10 | framing/tamanho/truncamento/tipo/bytes hostis falham fechados | `SPEC-GAP` | A propriedade fail-closed é clara, mas a gramática byte-level e classe de resultado não são únicas. |
| 11 | budgets sem truncamento/publicação parcial | `SPEC-GAP` | Limites são claros; contabilidade, duplicata e escolha `Unknown` versus erro permanecem abertas. |
| 12 | ausência no tree usa `on_missing`; objeto ausente não | `PASS` | Distinção explícita no prompt, ADR e F04. |
| 13 | equivalência com B1 sem normalizador paralelo | `PASS` | Mesmo extrator e comparador são requisitos explícitos; Git real fica apenas como regressão. |

Resultado: sete `PASS`, cinco `SPEC-GAP` e um `RED` normativo. `PASS` aqui significa
suficiência normativa da alegação, não conformidade da produção, que A não leu.

## Decisão de testabilidade

**Não existe seam/API L0 suficiente para B1/B2.** A expressão “L1 recebe uma porta
abstrata de conteúdo por path lógico” decide a arquitetura da extração, mas não permite
ao gate selecionar um executável hostil e observar o adapter/process lifecycle real.
Usar `PATH`, variável privada presumida, helper copiado ou comportamento descoberto no
código faria a expectativa depender da implementação; substituir o processo por mock
também não confrontaria `Command`, pipes, timeout, kill e reap.

Por isso B1 e B2 permanecem proibidos. A matriz do P0100 não deve ser materializada por
asserts inventados para preencher as lacunas.

## Saneamento L0 mínimo proposto

Alterar somente o contrato de refinamento e/ou a adenda ADR-0019, mantendo a decisão Git
em L3 e a semântica de fatos em L1:

1. **Resselar autocontenção:** declarar que repository root deve possuir `.git`
   diretório real interno, object database interno e nenhum alternate, object pool,
   linked worktree, bare repo, symlink ou override herdado (`GIT_DIR`,
   `GIT_WORK_TREE`, `GIT_COMMON_DIR`, `GIT_OBJECT_DIRECTORY`,
   `GIT_ALTERNATE_OBJECT_DIRECTORIES`). Remover a frase que delega alternates ao Git.
2. **Publicar uma seam L3 estreita:** expor uma entrada de auditoria que receba caminho
   absoluto do executável Git e invoque o mesmo adapter concreto usado em produção.
   Ela não recebe respostas simuladas, não contém semântica Git em L1 e não permite
   trocar budgets/política. O default produtivo continua selecionando `git`; B1/B2
   fornecem somente seu executável temporário controlado. Congelar nome, módulo,
   assinatura, visibilidade e tipos de retorno antes dos gates.
3. **Congelar transcript:** listar argv completo e em ordem para preflight, resolução,
   `ls-tree` e `cat-file`; `current_dir`; argumentos `--`/`--end-of-options`; mensagens
   exatas do batch; flush; stdin close; framing byte-level de stdout; política de
   stderr/status; duplicatas e bytes não UTF-8. Paths devem ser relativos, não vazios,
   sem NUL, componentes `.`/`..`, absoluto ou pathspec magic, preservando os demais
   bytes do sistema operacional quando representáveis pelo protocolo.
4. **Congelar ambiente:** especificar ambiente limpo ou lista exata removida, valores
   forçados, tratamento de `PATH` e `HOME`/XDG, `GIT_CONFIG_NOSYSTEM`, config global
   nula e cada `-c` usado para protocolos/hooks/lazy fetch/replace/locks.
5. **Congelar contabilidade:** 512 entradas observáveis distintas por contrato, 4 MiB
   inclusive por blob e 32 MiB inclusive por revisão; definir duplicatas, soma antes de
   publicar e precedência de falhas. Fixar um único resultado para cada excesso.
6. **Congelar lifecycle:** definir “operação Git”, relógio monotônico, deadline,
   processo/grupo ou job, encerramento de descendentes, fechamento dos pipes, reap,
   watchdog e classe de falha caso contenção não seja comprovável. Nenhum byte parcial
   chega à porta de conteúdo.
7. **Fechar taxonomia L3:** publicar enum/prefixos estáveis para `InvalidInput`,
   `MissingRef`, `MissingPath`, `MissingObject`, `ForbiddenObjectKind`,
   `InvalidFraming`, `BudgetExhausted`, `Timeout`, `ProcessFailure` e
   `ContainmentFailure`, com matriz única entre erro de entrada e razão `Unknown`.
   Códigos de exit permanecem fora, em F09.

O saneamento não autoriza lógica em L1, mudança de backend, dependência, rede, escrita,
temporário, exit ou composição ampla. Depois dele, recalcular todos os pins causais,
atualizar o Assessment 0029, congelar esta decisão resselada e somente então liberar B1
e B2 independentes.

## Veredito A

`SPEC-GAP` bloqueante. A autoridade para `refine-revisions` existe, mas o contrato ainda
não determina um oráculo hostil único nem uma seam capaz de exercê-lo. Há também `RED`
normativo explícito sobre alternates/object stores externos. Prosseguir agora produziria
`GATE-DEFECT`, não evidência funcional independente.
