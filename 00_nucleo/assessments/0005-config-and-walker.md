# Assessment 0005 — configuração e descoberta de arquivos

**Estado:** CONGELADO PARA TRIAGEM
**Data:** 2026-08-24
**Alvos:** `03_infra/config.rs` e `03_infra/walker.rs`

## Hipótese

Configuração e walker executam tradução mecânica de TOML e filesystem. Esperamos poucos
achados após o saneamento P0072. Um RED reclassifica toda entrada que pode omitir arquivo
ou atribuir camada antes das regras puras.

## Alegações sob teste

1. Configuração presente e inválida bloqueia; configuração ausente só usa defaults onde
   a interface explicitamente autoriza isso. Campos e contratos ambíguos são rejeitados.
2. Duas chaves de layer não podem possuir o mesmo diretório, e uma chave desconhecida
   não pode disputar precedência com L0–L4/Lab. Resolução independe da ordem TOML/HashMap.
3. Walker nunca silencia erro de travessia ou leitura de arquivo elegível; excluídos são
   a única fonte de silêncio deliberado.
4. Permutar criação/diretório não altera o conjunto nem a ordem canônica dos SourceFiles.
5. Symlink de arquivo ou diretório não escapa da raiz; path não UTF-8 não é confundido
   com outro path nem excluído por conversão lossy.
6. Exclusões de diretório atuam por componente e `excluded_files` por path relativo
   exato; prefixos, sufixos e separadores não produzem exclusão acidental.
7. Layer desconhecida permanece observável como `Layer::Unknown`; extensão não suportada
   é a única filtragem por linguagem aceita sem erro.
8. Detecção de teste adjacente é consistente entre linguagens e nunca usa diretório,
   symlink ou arquivo não regular como prova de cobertura.

## Gate curto

Até seis propriedades black-box/API. Produção não muda durante a triagem. Casos que
dependam de permissões Unix devem restaurá-las no fixture. Diferença entre documentação
e schema sem política executável é `SPEC-GAP`, não assertion inventada.

## Continuidade

Este é o primeiro lote após P0072. Os resultados permanecem no branch
`codex/segregated-materialization`; nenhum merge será considerado até o inventário de
assessments cobrir todos os módulos do linter e os REDs legítimos estiverem fechados.
