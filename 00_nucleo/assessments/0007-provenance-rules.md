# Assessment 0007 — regras puras de proveniência V5/V6/V7

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvos:** `prompt_drift.rs`, `prompt_stale.rs`, `orphan_prompt.rs`

## Hipótese

Depois do saneamento byte-exato de prompt I/O, as três regras L1 parecem mecânicas:
V5 compara digests já calculados, V6 compara duas interfaces já extraídas e V7 calcula
diferença entre dois inventários. Esperamos zero achados. Um RED é relevante porque
essas regras transformam a ligação prompt–código em diagnóstico consumido por agentes.

## Alegações sob teste

1. V5 emite exatamente um diagnóstico se, e somente se, os hashes declarado e atual
   existirem e forem diferentes; ausência de header, hash ou prompt permanece domínio
   das regras responsáveis por essas ausências.
2. V5 preserva no diagnóstico os dois valores e o path recebido, sem fazer I/O nem
   normalização adicional da identidade byte-exata entregue por L3.
3. V6 é invariante à mera permutação da mesma interface, mas detecta adição, remoção ou
   mudança em qualquer campo de função, tipo e reexport.
4. O delta de V6 é determinístico e completo: não perde multiplicidade semanticamente
   observável, não inventa diferenças e sua descrição independe da ordem de extração.
5. V7 emite uma violação por prompt realmente órfão, preserva o nível injetado e produz
   ordem e conteúdo determinísticos para inventários logicamente equivalentes.
6. Nenhuma das três regras aceita colisão ou diferença de representação como prova de
   igualdade; normalização autorizada pertence às fronteiras L3 já saneadas, não a L1.

## Gate curto

Até seis propriedades independentes, sem alterar produção. O verificador pode usar
dublês das traits públicas e permutações/exaustão finita. Casos cuja semântica não esteja
decidida são `SPEC-GAP`, não RED. A triagem termina em `PASS`, `RED` ou `SPEC-GAP` por
alegação e não autoriza correção automática.

## Segregação

- A congela este contrato e não interpreta a produção para justificar resultados.
- B escreve o gate a partir deste arquivo sem ler a implementação dos três alvos.
- C, somente após o primeiro gate, procura contraexemplos sem ler os testes de B.
- O orquestrador classifica divergências e decide se haverá saneamento separado.

## Parada

Se houver RED, congelar a evidência e parar antes de modificar L1. Se tudo passar,
registrar o laudo e avançar ao próximo lote. Não fazer merge, instalação ou release.
