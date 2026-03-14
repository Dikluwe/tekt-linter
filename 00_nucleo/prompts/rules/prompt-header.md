# Prompt: Rule V1 - Missing Prompt Header (prompt-header)

**Camada**: L1 (Core - Rules)
**Regra**: V1
**Criado em**: 2025-03-13

## Contexto
Toda lógica em um arquivo do Tekt/Crystalline arquitetura em camadas executáveis (`L1`, `L2`, `L3` e `L4`) deve declarar sua linhagem causal - um cabeçalho apontando de onde ele originou em `00_nucleo`. Esta regra varre os arquivos de origem nestas camadas para garantir o cumprimento estrito de sua linhagem.

## Especificação
- A regra V1 assinala a ausência do cabeçalho `@prompt` apropriado nos arquivos de L1–L4.
- Ela verifica `ParsedFile.prompt_header == None` no arquivo parseado.
- Adicionalmente, também acusa violação se um path existir no prompt header, mas esse arquivo não estiver presente em `00_nucleo/` (no contexto restrito puro em L1, isso é delegado e o linter de L1 acusa erro caso falte referência fornecida na interface).

## Estrutura da Violação Gerada
- Rule ID: `V1`
- Level: `Error` (Bloqueante)
- Contexto da Mensagem: "Arquivo Cristalino sem linhagem causal @prompt encontrada".

## Restrições (L1 Pura)
A regra é uma função que recebe um `ParsedFile` válido e inspeciona de forma puramente funcional se o `prompt_header` existe. Não abre o arquivo `00_nucleo/` em disco — essa validação é delegada na construção do `ParsedFile` via L3.
