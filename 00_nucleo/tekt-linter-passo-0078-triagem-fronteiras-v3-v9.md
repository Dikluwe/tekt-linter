# Passo operacional 0078 — triagem segregada das fronteiras V3/V9

> **Natureza:** envelope operacional temporário; não é regra arquitetural
> **Estado:** executado; RED congelado no assessment 0010
> **Branch:** `codex/segregated-materialization`
> **Contrato permanente:** assessment 0010

## Objetivo

Executar o assessment 0010 sobre a parte pura de V3/V9, mantendo fora do escopo a
resolução L3 de imports. Procurar especialmente assimetrias na matriz, interferência do
guard de testes, perda de multiplicidade e igualdade textual indevida em portas.

## Execução

1. Congelar este passo e o assessment antes do gate.
2. B escreve no máximo seis propriedades sem ler produção.
3. Após o primeiro gate, C procura contraexemplos próprios sem ler testes de B.
4. O orquestrador classifica cada alegação e preserva qualquer RED reproduzível.
5. Se tudo passar, executar suíte global, auto-lint e emitir relatório; se houver RED,
   parar antes de alterar L1 e separar um futuro saneamento.

## Critérios de fechamento

- matriz V3 7×7 e matriz V9 completas;
- ambos os estados do guard e todos os `ImportKind` observáveis;
- multiplicidade, ordem, linha, paths e representações Unicode preservadas;
- zero testes ignorados;
- produção intocada durante a triagem.

## Parada

Não fazer merge, instalação ou release.
