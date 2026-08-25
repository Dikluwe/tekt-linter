# Assessment 0023 — relatório N16 por módulo

**Estado:** PREFLIGHT SANEADO — gates B1/B2 autorizados; produção proibida
**Data:** 2026-08-25
**Passo:** P0094
**Baseline:** `fee11de`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| taxonomia N16 | `00_nucleo/adr/0017-v16-v21-diferenca-categorica.md` | `79f406654aacf3693616232a4fdbb911e359486d089ffde841af5375625104dd` |
| relatório histórico | `00_nucleo/tekt-linter-passo-0069-relatorio-n16-por-modulo.md` | `c0fd4d64e2489b49994ce56abb3fd0e7139caec11d744eb4e0799b566c22f2fe` |
| V16/exceções | `00_nucleo/prompts/rules/wildcard-saturation.md` | `19f79428f1e7c9740ae7f2466f03bc82c22a5632a2388e5b2c587a3fa2588609` |
| arquitetura | `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| apresentação | `00_nucleo/prompts/sarif-formatter.md` | `bd0a915c775c97482b1890a67c83b993d62a6fd0decf1dbd0f5913ade0afefa0` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| protocolo P0094 | `00_nucleo/tekt-linter-passo-0094-auditoria-relatorio-n16-summary.md` | `4ae7934828bbdbc200d5e0bba0a8d9380f1a3bc4a6792dc11a47791df90b358f` |

## Alegações candidatas

1. A gramática de tags é total e não depende de busca textual ambígua.
2. Chaves `path:line` têm parser total e preservam `:` pertencente ao path.
3. Fonte e exceção compartilham identidade de localização determinística.
4. Conflito de tags na mesma localização possui precedência explícita.
5. Agregação é invariante à permutação e à ordem do `HashMap`.
6. Agrupamento usa componentes, não substrings de path.
7. γ absoluto é a chave primária; empates têm ordem total publicada.
8. Tabela, percentuais, zeros e arredondamento são normativos.
9. Aviso de amostra pequena possui condição exata e cálculo de `~pp` total.
10. O formato é opt-in, não cria regra V nem muda exit status.
11. L2 permanece puro; L3 lê; L4 seleciona e injeta; L1 não formata.

## SPEC-GAPs candidatos para o adversário A

### G1 — gramática e multiplicidade da tag

O ADR define as categorias e o prompt V16 aceita formas autorizadas quando o texto
contém `N16[`, mas não decide primeira versus múltiplas tags, decoys, lixo posterior ou
se o relatório deve repetir exatamente o parser de justificativa da regra.

### G2 — chave de localização

O prompt V16 descreve chave exata `path:line` para a regra por arquivo, mas o relatório
agrega fontes e mapa global. Não há parser publicado para paths com `:`, linha ausente,
zero, overflow ou sufixos adicionais, nem identidade comum entre path absoluto/relativo.

### G3 — duplicata divergente

P0069 exige evitar dupla contagem, mas não decide qual tag vence quando fonte e TOML
classificam a mesma localização de modo diferente.

### G4 — agrupamento

P0069 cita primeiro diretório dentro de `01_core/src/` ou equivalente e lista categorias
de referência, sem algoritmo total para `compiler`, arquivo na raiz, outras camadas,
separadores, componentes próximos ou paths fora do workspace.

### G5 — desempates

P0069 fixa somente γ absoluto decrescente. Empates não têm política, embora bytes
determinísticos exijam ordem total. Percentual ou total como desempate seria invenção.

### G6 — percentual e representação

O texto diz que percentual é secundário e vem sempre entre parênteses, enquanto a tabela
o publica em coluna própria. O caso α-only usa `—` na referência, mas zero geral usa
`0.0%`; a regra total não está publicada.

### G7 — aviso de amostra pequena

O requisito literal diz qualquer módulo com `total < min_sample_size`; a validação espera
avisos apenas para dois módulos com γ, omitindo `parse/` e `export/`, também pequenos.
Limites zero/um e arredondamento de `~pp` não são decididos.

### G8 — consumidor e exit status

P0069 diz formato opt-in e não gate, mas não publica comportamento combinado com checks
distintos de V16, conjunto vazio, falha de leitura upstream ou exit status exato.

## Protocolo ativo

- A lê somente este Assessment e os oito insumos hash-pinned;
- B1/B2 só começam após saneamento e resselamento;
- C só lê produção após os dois gates congelados;
- D fecha causalidade, arquitetura Tekt, regressão e residual.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge/push.

## Parecer A e saneamento

O adversário A validou os oito hashes e classificou G2 como decidido, G1/G3/G4/G5/G8
como `SPEC-GAP` e G6/G7 como contradições. B1/B2 permaneceram bloqueados até saneamento.

O adendo normativo P0094 ao Passo 0069 passou a decidir: token único e gramática exata;
parser pelo último `:` e identidade nominal; precedência da fonte; agrupamento por
componentes; γ descendente com nome como único desempate; percentuais half-up;
representação total do vazio; aviso para todo módulo abaixo do limiar; cálculo de `~pp`;
e seleção opt-in que exige V16 sem transformar o relatório em regra. `linter-core` e
`sarif-formatter` passaram a enumerar o formato e apontar ao contrato detalhado.

Todos os insumos foram resselados. B1/B2 podem materializar gates cegos; leitura de
produção continua proibida até ambos serem congelados.

Na primeira tentativa, B1 e B2 recusaram corretamente inventar nomes de API e
classificaram um `SPEC-GAP` nominal adicional. Nenhum gate foi criado. O adendo passou a
publicar `N16Tag`, `N16ModuleStats`, `N16Stats`, `extract_n16_tag`,
`extract_n16_module_name`, `collect_n16_stats` e `format_n16_summary`, todos no módulo L2
`crystalline_lint::shell::n16_summary`, além do tipo de entrada L1. Os hashes foram
novamente resselados; os mesmos verificadores podem retomar sem ler produção.

B1 recusou uma segunda invenção: a função publicada recebia `SourceFile`, mas o L0 não
expunha forma de construí-lo em fixture de memória. Nenhum gate foi criado. O adendo
passou a publicar seus cinco campos existentes e os caminhos/variantes mínimos de
`Language::Rust` e `Layer::L1`, sem autorizar descoberta ou leitura em L2. Novo
resselamento libera B1.
