# Assessment 0011 — classificadores de declaração V12/V13

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvos:** `wiring_logic_leak.rs`, `mutable_state_core.rs`

## Hipótese

V12 e V13 recebem declarações já extraídas e aplicam tabelas finitas por camada. Esperamos
zero achados. Extração AST, aliases profundos e configuração TOML permanecem fora deste
lote; aqui validamos somente a transformação pura da evidência recebida.

## Alegações sob teste

1. V12 é vazia fora de L4. Em L4, Enum, Impl, Interface e TypeAlias sempre violam;
   Struct e Class violam exatamente quando `allow_adapter_structs=false`.
2. V12 produz uma `Warning` por ocorrência proibida e preserva ordem, multiplicidade,
   source path, linha, kind e nome na evidência, inclusive representações Unicode.
3. Alternar `allow_adapter_structs` afeta somente Struct/Class; nenhuma outra espécie é
   silenciada ou criada e o input não é mutado.
4. V13 é vazia fora de L1. Em L1, `is_mut=true` sempre viola; caso contrário viola se,
   e somente se, `type_text` contém um dos tokens públicos congelados.
5. V13 cobre integralmente os 18 tokens, preserva ordem/multiplicidade/path/linha/nome e
   reporta `mut` com precedência quando `is_mut=true`, sem normalizar o texto recebido.
6. Tipos imutáveis e representações próximas que não contêm token permanecem isentos;
   execuções repetidas produzem o mesmo vetor completo e nenhuma regra faz I/O.

## Gate curto

Até seis propriedades independentes com tabelas de camadas, kinds, tokens e permutações.
Produção não é alterada. Resultado por alegação: `PASS`, `RED` ou `SPEC-GAP`.

## Segregação

- B escreve e executa o gate sem ler os dois alvos.
- C lê contrato e produção após o primeiro gate sem ler testes de B.
- O orquestrador congela achados antes de qualquer correção.

## Parada

Se houver RED, registrar evidência e parar antes de modificar L1. Se tudo passar, emitir
laudo e avançar. Não fazer merge, instalação ou release.
