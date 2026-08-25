# Relatório P0105 — artefato Núcleo Tekt

**Estado:** READY WITH RESIDUAL AUDIT — não merge-ready por bloqueio P0104/P0106
**Data:** 2026-08-25
**Branch:** `codex/tekt-nucleus-artifact`

## Resultado

Tekt agora possui uma relação compartilhável diferente de ownership:

```text
Núcleo Tekt `.tekt` ── 0:N ──> prompts proprietários ⇄ códigos
```

`.tekt` v1 é TOML estrito, versionado, não executável e limitado. Claims possuem id,
modalidade e statement. Núcleos podem formar DAG hash-pinned. Código apontando diretamente
a `.tekt` gera V26.

## Causalidade e hash

- prompts sem núcleo preservam exatamente o V5 legado;
- prompts com núcleo incluem digests reais e ordenados no hash efetivo;
- núcleos incluem transitivamente seus dependencies;
- paths e comprimentos são framed; ciclos/missing não produzem digest;
- pins usam 64 hex; `@prompt-hash` continua com oito.

Uma mudança de um byte na fixture compartilhada produz duas V26 e duas V5. `--fix-hashes`
atualiza primeiro os bytes finais/pins dos prompts, calcula os hashes finais dos códigos e
aplica tudo pelo plano transacional. A segunda passagem fica V5/V26 limpa.

## Implementação

- ADR/contrato/Assessment: `2226e3b`;
- gates RED: `461414c`;
- parser/grafo/V26/hash: `6ee2463`;
- pins transacionais e cache fresco: `c64b785`;
- fronteiras fail-closed: `17358d4`.

## Validação

- 630 testes unitários: PASS;
- 83 fixtures históricas: PASS;
- B1: 4/4; B2: 3/3; B3: 3/3; B4 final: 4/4; B5: 2/2;
- SARIF V0–V26: PASS;
- prompt I/O/confinamento/limite: PASS;
- auto-lint V26: nenhuma violação;
- `git diff --check`: PASS;
- nenhum projeto externo foi modificado por P0105.

## RED adicional fechado

O reparador escrevia corretamente, mas revalidava usando o cache de hashes criado antes da
escrita e anunciava deriva residual falsa. A segunda passagem agora instancia leitor novo.

## Residuais e próximo passo

Não foi criado núcleo real no próprio linter. O piloto exigiria individualizar prompts de
um dos 13 grupos de P0104, decisão reservada a P0106. Também não houve verificador por agente
independente nesta execução.

O auto-lint V15 continua encontrando os mesmos 13 prompts compartilhados, e
`--fix-hashes --dry-run` bloqueia antes de qualquer write. Consequentemente, headers novos
não podem ser resselados pelo fluxo oficial ainda.

Próximo passo: P0106 deve classificar os 13 grupos, extrair somente claims comuns para
Núcleos Tekt, criar prompts proprietários 1:1 e então resselar V1/V5/V7/V15/V26. Somente
depois cabem merge, instalação e retorno ao Typst/Bateia.
