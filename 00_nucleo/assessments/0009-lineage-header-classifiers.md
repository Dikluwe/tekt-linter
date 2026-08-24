# Assessment 0009 — classificadores de cabeçalho V1/V15

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvos:** `prompt_header.rs`, `multi_prompt_header.rs`

## Hipótese

V1 e V15 recebem cabeçalhos já extraídos e executam predicados pequenos. Depois do
saneamento de prompt I/O, esperamos zero achados. Um RED é importante porque ausência
ou ambiguidade de linhagem muda diretamente quais prompts agentes consideram causais.

## Alegações sob teste

1. V1 aplica-se somente a L1–L4: header ausente ou referência inexistente produz uma
   violação; L0, Lab e Unknown não são obrigados por esta regra.
2. Em L1–L4, V1 passa somente quando header existe e `prompt_file_exists` é verdadeiro;
   a tabela das duas condições é completa e produz no máximo uma violação.
3. A severidade V1 é `Fatal` somente quando o path pertence por componentes a um
   diretório estrito; prefixos textuais próximos não pertencem. Fora deles é `Error`.
4. V15 emite uma única `Error` se, e somente se, há duas ou mais refs em L1–L4; zero ou
   uma ref e todas as quantidades em L0/Lab/Unknown são isentas.
5. V15 preserva quantidade, ordem, duplicatas e representação textual das refs na
   evidência, além do path e posição pública; não escolhe nem normaliza um prompt.
6. Ambos preservam rule id, localização e conteúdo Unicode/representações distintas,
   são determinísticos e não fazem I/O ou correção.

## Gate curto

Até seis propriedades independentes, usando traits/entidades públicas e tabelas finitas.
Não testar extração do parser nem filesystem: essas fronteiras possuem assessments
próprios. Resultado por alegação: `PASS`, `RED` ou `SPEC-GAP`; produção não é alterada.

## Segregação

- B escreve e executa o gate sem ler os dois alvos.
- C lê contrato e produção somente após o primeiro gate, sem ler testes de B.
- O orquestrador congela qualquer RED antes de saneamento.

## Parada

Se houver RED, registrar evidência e parar antes de modificar L1. Se tudo passar, emitir
laudo e avançar. Não fazer merge, instalação ou release.
