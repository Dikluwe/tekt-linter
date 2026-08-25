# Prompt: regra V26 — integridade de Núcleos Tekt
Hash do Código: PENDENTE_P0105

## Contexto

ADR-0022 separa ownership `prompt ⇄ código` da dependência compartilhável
`núcleo → prompts`. O grafo precisa de validação pura após extração.

## Instrução

Receber nós e usos já extraídos, validar dependências ausentes, ciclos e órfãos, e emitir
achados determinísticos V26 sem I/O ou normalização física.

## Restrições

- identidade é path lógico integral e case-sensitive;
- múltiplos prompts podem usar o mesmo núcleo;
- ordem de entrada não altera bytes de saída;
- ausência/ciclo é Error; órfão é Warning;
- não misturar V1/V5/V7/V15.

## Critérios de verificação

DAG compartilhado passa; missing, self-loop e ciclos falham; órfãos são observáveis;
permutação e caixa não escolhem owner implícito.

## Resultado esperado

Regra L1 `nucleus_integrity` pura e coberta por gate in-memory.
