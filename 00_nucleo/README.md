# Índice do núcleo do `tekt-linter`

Este diretório reúne contratos causais e material operacional. A numeração `0051`–`0070`
não pertence à Arquitetura Tekt nem ao produto: veio do roteiro do repositório-oráculo
`typst-crystalline` e serviu apenas para ordenar comandos enviados ao LLM.

Um passo não é regra, decisão, contrato nem unidade arquitetural. Seu nome atual só é
preservado provisoriamente para manter links e commits rastreáveis enquanto o conteúdo
útil é absorvido pelos artefatos permanentes.

Novos envelopes, quando inevitáveis para orientar uma execução, usam identidade
descritiva sem número, como `tekt-linter-passo-validacao-de-refinamento.md`. Isso não
inicia uma sequência substituta: o arquivo continua temporário e sem autoridade causal.

## Identificadores canônicos

| Família | Forma | Função | Numeração |
|---|---|---|---|
| Regra pública | `V0`, `V1`, … | diagnóstico emitido pelo linter | catálogo próprio e estável |
| Decisão | `ADR-0001`, … | escolha arquitetural | sequência local em `adr/` |
| Prompt | nome sem número | contrato causal de um componente | identidade pelo caminho em `prompts/` |
| Passo/laudo | nome histórico atual | envelope operacional para o LLM | não canônica; temporária |

Por exemplo, o arquivo chamado P0066 introduziu as regras V21 e V22, mas `P0066` não é
uma identidade da arquitetura. Nenhum número de passo deve ser alinhado com `V*` ou
usado como dependência semântica.

## Ciclo de trabalho

1. Um comando operacional pode existir temporariamente para orientar o LLM; ele não
   cria autoridade causal.
2. Quando houver decisão pública, criar o próximo ADR local.
3. Criar ou atualizar os prompts causais antes de alterar L1–L4.
4. Materializar, validar e registrar o resultado nos artefatos permanentes adequados.
5. Depois de absorvido o conteúdo, arquivar ou eliminar o comando operacional, desde
   que referências históricas relevantes tenham sido migradas.
6. Referenciar medições externas por repositório + SHA, nunca pela numeração corrida
   do outro repositório.

Não se cria uma nova sequência local para substituir a sequência herdada. O trabalho é
descoberto por contratos vigentes, decisões, código e histórico do Git — não pelo maior
número de passo.

## Inventário operacional provisório

| Faixa | Conteúdo | Destino esperado |
|---|---|---|
| P0051–P0062 | robustez, mutação, oráculos e primeiro release | auditar e absorver; depois arquivar/remover |
| P0063–P0065 | V16–V20 e reconciliação com `syn` | ADRs/prompts/laudos permanentes onde necessário |
| P0066–P0068 | V21/V22 e fundamentação | ADRs e prompts vigentes |
| P0069 | relatório N16 por módulo | documentação permanente do recurso ou histórico Git |
| P0070 | V23–V25 | absorvido por ADR-0018 e prompts vigentes; elegível para arquivo/remoção |
| Validação de refinamento | ADR-0019/L0 vigentes; Etapas A e B1 materializadas no branch dedicado | validar e manter Git/wrapper/SMT fora de escopo |
| Extração de snapshots | Etapa B1 autorizada e materializada no branch de refinamento | absorver em ADR-0019/L0; sem iniciar sequência numérica |

P0070 já foi absorvido pelos artefatos causais correspondentes e deixou de ser fonte
necessária. O passo descritivo de validação de refinamento é apenas uma proposta; ele
não autoriza código e contém sua própria parada obrigatória.

## Organização física

- `adr/`: decisões arquiteturais locais.
- `prompts/`: contratos causais vigentes, usados pelos cabeçalhos de linhagem.
- `tekt-lint-passo-*` e `tekt-linter-passo-*`: envelopes operacionais provisórios; os
  dois prefixos atuais são mantidos apenas durante a migração.

Não use o número do último passo para descobrir, nomear ou legitimar o próximo trabalho.
