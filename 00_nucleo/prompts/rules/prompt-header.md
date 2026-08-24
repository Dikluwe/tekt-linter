# Prompt: Rule V1 - Missing Prompt Header (prompt-header)
Hash do Código: 46495747

**Camada**: L1 (Core - Rules)
**Regra**: V1
**Criado em**: 2025-03-13

## Contexto
Toda lógica em um arquivo do Tekt/Crystalline arquitetura em camadas executáveis (`L1`, `L2`, `L3` e `L4`) deve declarar sua linhagem causal - um cabeçalho apontando de onde ele originou em `00_nucleo`. Esta regra varre os arquivos de origem nestas camadas para garantir o cumprimento estrito de sua linhagem.

## Especificação
- A regra V1 assinala a ausência do cabeçalho `@prompt` apropriado nos arquivos de L1–L4.
- L0, Lab e Unknown estão fora do escopo e retornam vazio antes de qualquer outra avaliação.
- Ela verifica a ausência de um cabeçalho válido (via trait `HasPromptFilesystem`) no arquivo parseado.
- Adicionalmente, também acusa violação se um path existir no prompt header, mas esse arquivo não estiver presente em `00_nucleo/` (no contexto restrito puro em L1, isso é delegado e o linter de L1 acusa erro caso falte referência fornecida na interface).

Em camada aplicável, os estados são disjuntos e produzem no máximo uma violação:

- sem header: mensagem histórica `Arquivo Cristalino sem linhagem causal @prompt encontrada`;
- com header e prompt inexistente: mensagem distinta que inclui literalmente `header.prompt_path`;
- com header e prompt existente: nenhuma violação.

O path causal não é normalizado: caixa, Unicode e representação original são evidência.

## Estrutura da Violação Gerada
- Rule ID: `V1`
- Level: `Error` (Bloqueante)
- Contexto da Mensagem: "Arquivo Cristalino sem linhagem causal @prompt encontrada".
- Para referência inexistente, a mensagem distingue a causa e contém o path declarado.
- Path do arquivo, linha 1, coluna 0 e cardinalidade máxima de uma V1 são invariantes.
- Severidade `Fatal` em diretório estrito por componentes; `Error` nos demais paths aplicáveis.

## Restrições (L1 Pura)
A regra é uma função que recebe uma entidade (via trait `HasPromptFilesystem`) e inspeciona de forma puramente funcional camada, `prompt_header` e a evidência booleana de existência. Não abre o arquivo `00_nucleo/` em disco — essa validação é delegada na construção final via L3. A camada vem da entidade; V1 não a infere pelo path.

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-08-24 | Escopo explícito L1–L4 e estados disjuntos para header ausente e referência inexistente, preservando o path causal literal | prompt_header.rs, rule_traits.rs, parsed_file.rs |

## Fundamentação Teórica

1. **Vínculo Formal de Rastreabilidade (Explicit Trace Link):**
   * **Erata et al. (2017, 2024)** (*A Tool for Automated Reasoning about Traces Based on Configurable Formal Semantics*): A verificação estática de conformidade exige âncoras sintáticas explícitas conectando as unidades de código aos seus artefatos de especificação. Sem a declaração formal do `@prompt`, o grafo de derivação torna-se indecidível para ferramentas de análise estática.

2. **Where-Provenance e Linhagem Causal:**
   * **Buneman et al. (2001)** (*Why and Where: A Characterization of Data Provenance*): Todo artefato derivado em um sistema formal deve carregar a proveniência exata de sua fonte (`prov:wasDerivedFrom`). No Crystalline/Tekt, essa linhagem causal é materializada na anotação de cabeçalho `@prompt <caminho>`.
