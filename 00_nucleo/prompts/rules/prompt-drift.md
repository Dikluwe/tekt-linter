# Prompt: Rule V5 - Prompt Drift (prompt-drift)

**Camada**: L1 (Core - Rules)
**Regra**: V5
**Criado em**: 2025-03-13

## Contexto
O princípio causa-efeito cristalino presume que o código final seja manifestação exata do design ditado nas diretrizes L0 (prompts). A regra de deriva (drift detection) previne contra a obsolescência das descrições.  Se um humano modificou um prompt no `00_nucleo/`, todos os arquivos (L1-L4) atrelados causalmente a esse prompt devem ser re-verificados pelo Agente (ou humano), e seu header `<sha256>` atualizado para o hash vigente do design atualizado.

## Especificação
- A regra V5 inspeciona `ParsedFile.prompt_header`.
- O hash `prompt_header.prompt_hash` do código implementador divergirá do `SHA256[0..8]` do prompt contido em disco, indicando derivação/desatualização silenciosa.
- O campo `prompt_hash` esperado do atual arquivo contido em disco foi previamente obtido pela camada infra L3 ao instanciar o `ParsedFile` (O L3 lê o `linter-core.md`, hasheia em sha256 e provê para o objeto da l1 no momento de _check_).
- Se os 8 bytes curtos listados no cabeçalho divergem desse hash real extraído pelo `PromptReader` inferido na L3, o drift é disparado.

## Estrutura da Violação Gerada
- Rule ID: `V5`
- Level: `Warning` (Não bloqueia o CI por default, configurável pelo `crystalline.toml`)
- Contexto da Mensagem: "Deriva detectada (Drift): o arquivo @prompt original foi modificado sem atualização condizente da implementação. Hash L0: <real>, Código: <encontrado>."

## Restrições (L1 Pura)
Assim como nas demais, V5 nunca checa bytes físicos de disco. Ela unicamente compara as `String` de hashes expostas em `ParsedFile` populadas previamente no ciclo infra da Fiação que as entrega à checagem do núcleo.
