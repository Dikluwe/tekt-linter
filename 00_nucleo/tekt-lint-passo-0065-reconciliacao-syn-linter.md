# Passo 0065 — Reconciliação syn ↔ linter (fecho do gate do ADR-0016)

**Precede este passo:** Laudo 0064 (2026-08-14) — calibração de V16/V20
implementada, mas a tabela de concordância apresentada compara 0063 vs 0064
(linter vs linter), não linter vs ground truth syn. O gate do ADR-0016
(questão em aberto nº 1) ficou por demonstrar.
**Objetivo deste passo:** (A.1) produzir a tabela cruzada syn ↔ linter sobre
o **mesmo universo de casos** (os 291 wildcards não-erro identificados pela
varredura syn de 2026-08), com taxonomias mapeadas; (A.2) cada divergência de
categoria explicada numa linha (proxy diferente OU referência errada);
(A.3) decidir o fecho do ADR-0016 com base nessa tabela.

**Data:** a preencher pelo executor.
**Commit base:** `<sha>` tekt-linter (pós-0064) + `<sha>` typst-crystalline
(o mesmo usado na varredura syn — o universo de casos tem de coincidir).

**Natureza:** passo de investigação pura. Nenhum código de produção muda em
nenhum dos dois repositórios; o único artefacto de código é o script de
varredura syn adaptado (descartável, reverte/arquiva em `lab/`).

---

## Contexto (os números em tensão)

| Categoria V16 | Referência syn (2026-08) | Linter 0064 | Estado |
| :--- | :--- | :--- | :--- |
| DENY (saturação arbitrária) | ~8–10 | 8 | ✓ concorda |
| WARN neutro | ~18 (estimativa por amostragem) | 132 | ✗ diverge |
| WARN walker | 131 | 43 | ✗ diverge |
| INFO delegação | 1 | 6 | ✗ diverge |
| **Total** | 176 | 195 | Δ +19 |

Hipóteses registadas no laudo anterior: (a) o proxy de enum-candidato
(≥ 2 braços com prefixo qualificado comum) exclui walkers com um só braço
explícito; (b) as fronteiras neutro/walker/delegação do linter não coincidem
com as da triagem manual (que classificava por intenção semântica);
(c) a referência «~18 neutros» nunca foi uma contagem completa — foi
interpolada da amostra de 29.

## Fase A — Re-contagem syn com a taxonomia final

Adaptar o script syn da saga 2026-08 para classificar os 291 casos com as
categorias **finais do linter** (`BodyForm`): Saturação (`EnumPath`/
`LiteralOther`), Neutro (tabela §2.3 da calibração, incl. `None`, `vec![]`),
Walker (`{}`/`continue`), Delegação (`Call`), MessageProducer (isento),
Reincorporação (isento), Scrutinee aberto (isento), Erro (isento).

**Conclusão:** tabela de referência definitiva, substituindo as estimativas
«~18»/«~8–10» por contagens exactas. Se a referência mudar face a 2026-08,
registar — é ela que estava soft, não necessariamente o linter.

## Fase B — Tabela cruzada caso a caso

Para cada um dos 291 casos (ficheiro:linha): classificação syn ×
classificação linter. Produzir:

1. Matriz de contingência completa (não só marginais — os cruzamentos
   walker→neutro, delegação→neutro, etc. são onde a informação vive).
2. Lista dos casos fora da diagonal, cada um com uma linha de explicação:
   «proxy X classifica assim porque Y; referência classificava Z».
3. Veredito por divergência: **proxy correcto** (referência desactualizada),
   **proxy a ajustar** (com issue aberta no tekt-linter), ou
   **caso genuinamente ambíguo** (registar; não forçar).

**Conclusão:** percentagem de concordância caso-a-caso após mapeamento de
taxonomias, por categoria e global.

## Fase C — Fecho do ADR-0016

- Se concordância global ≥ 95% (ou divergências todas explicadas como
  referência-soft/proxy-documentado): escrever **ADR-0016 rev. 1** — status
  **ACEITO**, com a tabela cruzada anexa e as fronteiras de taxonomia
  declaradas («o linter conta neutros onde a triagem manual contava walkers
  quando …»).
- Se < 95% sem explicação: ADR permanece PROPOSTO; as divergências viram as
  novas questões em aberto (com proxies concretos a ajustar, não genéricas).

## Fase D — Higiene colateral (pequena, mesmo passo)

- Fechar os 3 V7 (prompts órfãos) detectados na corrida 0063: referenciar,
  remover, ou justificar em `[orphan_exceptions]`.
- Registar no relatório o worklist herdado para o passo seguinte no
  typst-crystalline: **8 DENY + 132 neutros + 43 walkers**.

## Critérios de aceitação

- [ ] (A.1) tabela cruzada sobre os 291 casos, contagens syn exactas (sem «~»)
- [ ] (A.2) 100% dos casos fora da diagonal com explicação de uma linha
- [ ] (A.3) ADR-0016 rev. 1 emitido (ACEITO ou PROPOSTO com novas questões)
- [ ] 0 V7 no typst-crystalline
- [ ] `crystalline-lint .` verde em ambos os repos ao fechar

## Comandos de validação

```bash
python3 tools/varredura_syn_v16.py /caminho/typst-crystalline   # Fase A (script lab)
crystalline-lint --checks v16 --format sarif /caminho/typst-crystalline
crystalline-lint .                                              # ambos os repos
```

## Esqueleto do relatório

`tekt-lint-passo-0065-relatorio.md`: Resumo executivo (resposta a A.1–A.3,
com o veredicto do ADR) → Metodologia (commits, universo de casos) →
Fase A (referência definitiva) → Fase B (matriz de contingência + lista de
divergências explicadas) → Fase C (texto do ADR-0016 rev. 1) → Fase D
(V7 fechados) → estado da árvore → Proveniência (script lab marcado
`(revertido/arquivado)`).

## Nota para o executor

A tentação neste passo é «corrigir» o proxy para bater a referência. Não:
o passo mede e **explica**; ajustes de proxy só com issue própria e depois
do ADR rev. 1. A referência também pode estar errada — foi construída por
amostragem e triagem manual, e a Fase A existe exactamente para a auditar.
