# Assessment 0006 — I/O de prompts, snapshots e hashes

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvos:** `prompt_reader`, `prompt_walker`, `hash_writer` e `prompt_snapshot_reader`

## Hipótese

Esses adapters parecem pequenos e mecânicos. Esperamos que conteúdo diferente produza
identidade diferente, que paths permaneçam confinados e que escrita atômica preserve o
arquivo em qualquer falha. Um RED reclassifica toda proveniência V5/V6/V7.

## Alegações sob teste

1. Paths recebidos são relativos, confinados à raiz e não escapam por `..`, absoluto ou
   symlink; `exists` significa arquivo regular local, não diretório ou link externo.
2. O hash ignora somente a metainformação canônica no header. A mesma sequência de bytes
   fora dessa linha produz o mesmo hash; qualquer outra mudança de bytes, inclusive
   newline final, CRLF e texto contendo as palavras da meta, permanece observável.
3. Limite de tamanho, leitura e hash usam a mesma captura lógica; arquivo ausente,
   ilegível, grande ou trocado não vira hash válido de outra entrada.
4. Walker de prompts não segue symlink, retorna conjunto determinístico, aplica exceção
   por path exato e não confunde erro interno com ausência de prompt.
5. Snapshot aceita exatamente um marcador canônico e schema completo; marcador
   duplicado, texto-isca, JSON truncado/desconhecido ou path externo não vira snapshot.
6. Escrita de hash/meta valida digest, altera somente a linha autorizada, preserva bytes
   e permissões restantes, usa temporário único no mesmo diretório e não deixa resíduo
   nem trunca destino em falha ou concorrência.

## Gate curto

Até seis propriedades independentes, sem alterar produção. Testes de corrida/TOCTOU só
entram se determinísticos; caso contrário, registrar limitação. Comportamento hoje
absorvido por `Option` pode ser classificado `SPEC-GAP` quando a interface não distingue
ausência de erro.

## Continuidade

Nenhum merge antes da cobertura integral do linter. REDs desta fronteira devem ser
saneados antes de assessments que dependam de V5/V6/V7.

## Resultado da triagem

O gate independente terminou com um PASS e cinco REDs:

- readers e snapshot aceitam vazio, `.`, absoluto, `..`, diretório e symlink externo;
- hash oculta newline final, CRLF e linha-isca no body, embora preserve BOM/espaço;
- limite de 10 MiB passou nos dois lados da fronteira;
- walker passou ordem/exceções/symlinks, mas aceitou `prompts` como arquivo e devolveu
  scan vazio em vez de erro;
- snapshot aceitou marcador em parágrafo/fence, duplicata e campo JSON desconhecido;
- writers aceitaram digest inválido, normalizaram bytes e perderam permissões.

Frescura do cache, TOCTOU de metadata/read e concorrência de writers permanecem
`SPEC-GAP`: não existe seam/barreira pública para prova determinística. Os cinco REDs
ficam congelados e devem ser saneados antes de retomar a triagem dependente de V5/V6/V7.
