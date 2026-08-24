# Assessment 0014 — revisão adversarial segregada V17–V20

**Produtor:** `adversary/v17-v20/0014`
**Insumo L0 validado:** `00_nucleo/prompts/rules/wildcard-saturation.md`
**SHA-256:** `66c502de44ef21880a68fe798c74ef5f3a91b9fe7dd3e925c2722d99d25f6800`
**Resultado inicial:** `SPEC-GAP`

## Achados congelados

### SG1 — API black-box ausente

O assessment e o L0 nomeiam campos conceituais, mas não publicam a API Rust invocável:
construtores/visibilidade do IR, entrada com language/path, assinaturas dos quatro
classificadores e tipo/acessores de diagnóstico. O verificador não consegue materializar
um gate compilável sem ler L1–L4 ou adivinhar. O gate fica intencionalmente bloqueante.

### SG2 — pertinência de V18 ambígua e não representada

A seção de regras irmãs lista `lexer` e `numbering`; a fundamentação e o assessment
incluem também `syntax`. Além disso, o contrato de `HasDecisionArms` enumerado não mostra
path ou identidade de módulo, embora V18 dependa dela. Não estão definidos componente
integral versus substring, sensibilidade a case, paths absolutos/relativos ou separadores.

### SG3 — exceção de V20 não decidível pelo IR publicado

“Tabela regular de tuplas sobre os mesmos tipos” exige forma, aridade e identidade dos
tipos dos padrões, mas o IR publicado oferece apenas snippet, profundidade e atributos
não relacionados. Também não define como catch-all, guard ou um único braço irregular
afetam a isenção da expressão inteira.

### SG4 — fronteiras residuais

- `or_alternatives = 0` cabe no tipo, embora o L0 diga “1 = não-or”;
- `Span` não tem estrutura publicada suficiente para o gate provar path/linha/coluna;
- V17–V20 não possuem templates completos de mensagem, apesar de snippet verbatim,
  contagem/profundidade e nomenclatura nativa serem observáveis exigidos.

## Matriz de ataques preservada

| Unidade | Ataque mínimo | Oráculo |
|---|---|---|
| comum | Rust versus todas as demais linguagens; vazio; 2 expressões × vários braços | silêncio fora de Rust; cardinalidade e ordem expressão→braço |
| V17 | tabela verdade `00/01/10/11`; snippets com `&&`/`||` irrelevantes | somente `11`, Warning, um por braço |
| V18 | componente, substring, case, separadores e Unicode | somente ranges fora da allowlist normativa |
| V19 | `0,1,2,3,u16::MAX` | Info iff `>1`, uma ocorrência e N preservado |
| V20 | `0,1,2,3,u8::MAX`; tabela homogênea/heterogênea/quase-tabela | Info iff `>2` fora da exceção decidível |
| isolamento | mutar todos os campos não decisórios | saída integral invariante |
| evidência | snippets repetidos/Unicode e spans distintos | snippet e location do braço correto |

## Classificação

Nenhum `RED` de produção foi alegado nesta fase, porque o contrato ainda não permite um
oráculo executável completo. Converter qualquer um dos gaps acima em PASS seria defeito
do gate. O saneamento deve começar em L0/assessment, ser resselado por hash e então
entregue a um verificador novo.
