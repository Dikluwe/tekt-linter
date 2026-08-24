# Passo operacional — extração determinística de snapshots de refinamento

> **Natureza:** comando operacional temporário para o LLM; não é regra arquitetural
> **Estado:** Etapa B1 materializada e validada em branch dedicado
> **Identidade:** descritiva e não numerada
> **Destino:** conteúdo absorvido por ADR-0019 e `refinement-validator.md`

## Objetivo

Tornar `refine` autoaplicável sem introduzir Git, wrapper, SMT ou inferência semântica
por nomes. Adicionar um subcomando que extraia fatos Rust de um diretório para o formato
de snapshot v1 já consumido pelo comparador:

```bash
crystalline-lint snapshot \
  --contract refinement.toml \
  --artifact-id working-tree \
  --output working-tree.refinement.json \
  .
```

## Decisão mecânica

Cada `[[observable]]` do contrato declara:

- chave estável;
- linguagem (`rust` nesta etapa);
- arquivo relativo, sem fuga da raiz;
- query tree-sitter;
- nome da captura cujo texto vira o valor;
- cardinalidade esperada (`one` ou `many`);
- política explícita para zero matches (`unknown` ou `absent`).

O extrator normaliza apenas whitespace e, em cardinalidade `many`, ordena os valores.
Não interpreta nomes, tipos, chamadas ou relações. Query inválida, arquivo ilegível,
parse com erro, captura ausente ou cardinalidade ambígua geram `Unknown` tipado ou erro
de entrada conforme a evidência disponível — nunca `Known` inventado.

## Fixtures RED obrigatórias

1. uma captura única produz `Known`;
2. `many` independe da ordem de descoberta;
3. zero matches com `on_missing = "absent"` produz `Absent`;
4. zero matches com `on_missing = "unknown"` produz `Unknown(MissingObservable)`;
5. mais de um match sob cardinalidade `one` produz `Unknown(AmbiguousIdentity)`;
6. AST com erro produz `Unknown(OpaqueConstruction)`;
7. path que escapa da raiz é rejeitado;
8. duas execuções geram bytes idênticos;
9. snapshot do próprio linter refinado contra cópia idêntica produz `PRESERVED`;
10. os três oráculos históricos são reduzidos a fixtures locais de fatos/queries.

## Camadas

- L1: política pura que transforma capturas normalizadas em `ObservableValue`.
- L2: argumentos do subcomando e mensagem de sucesso.
- L3: leitura, tree-sitter Rust, query, normalização e escrita atômica do JSON.
- L4: composição e exit code; sem lógica de extração.

## Guardas

- não ler commits ou executar Git no produto;
- não executar comandos externos;
- não importar código de `lab/`;
- não compartilhar conclusões negativas de V23–V25 como se fossem fatos neutros;
- não aceitar path absoluto ou `..` no contrato;
- não incluir relógio no snapshot;
- não depender da ordem do filesystem ou da AST;
- não reservar regra `V*`;
- não atualizar o binário estável instalado no sistema durante o branch experimental.

## Gate de aceitação

- testes existentes e novos verdes;
- auto-lint sem warning/error;
- hashes L0 resselados;
- `snapshot → refine` executado sobre o próprio branch;
- relatório registra o que foi comprovado e o que segue inconclusivo;
- commit separado do checkpoint da Etapa A.

## Registro de execução

- 579 testes unitários, 83 fixtures gerais e 6 testes de caixa-preta passaram;
- duas extrações do próprio branch foram byte-a-byte idênticas;
- auto-refinamento idêntico retornou `PRESERVED`/exit 0;
- baseline pré-Etapa A `f8a0dae` retornou `UNKNOWN(MissingObservable)`;
- baseline Etapa A `18a9b6e` refinado contra B1 retornou `PRESERVED`;
- contexto, campo e autoridade aceitam correção e rejeitam regressão;
- auto-lint passou sem warning/error e sem deriva de hash.
