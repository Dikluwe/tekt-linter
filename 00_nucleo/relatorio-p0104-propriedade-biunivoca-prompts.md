# Relatório P0104 — propriedade biunívoca de prompts

**Estado:** BLOCKED
**Data:** 2026-08-25
**Branch:** `codex/v15-bijective-prompt-ownership`
**Implementação:** `d0fb12a`

## Resultado

V15 agora fecha as duas direções da propriedade:

```text
código ── exatamente um ──> prompt proprietário
prompt ── exatamente um ──> código produtivo
```

A regra global é pura em L1 e recebe a visão integral de L4. Identidades permanecem
case-sensitive; consumers são deduplicados e ordenados; há uma violação por prompt
compartilhado. Todos os parsers participam pela referência canônica.

`--fix-hashes` deixou de aplicar pares independentemente. O comando valida ownership e
preflight integralmente, usa um plano comum a dry-run/execução, restaura aplicações
anteriores em caso de falha e valida os bytes de código e prompt depois da escrita. Falha
ou rollback falho não são apresentados como fechamento.

## Evidência

- B1 bijeção pura: 8/8 PASS.
- B2 binário real/multi-parser: 7/7 PASS.
- B3 transação e reprodução P1179: 8/8 PASS.
- Suíte completa: 630 unitários e 83 fixtures PASS, além dos gates de integração.
- Typst Crystalline: nenhuma escrita; inventário P1179 revalidado.
- Self-lint V15: 13 diagnósticos, exatamente os 13 compartilhamentos de A.
- Dry-run do reparador: bloqueado antes de writes; estado Git idêntico antes/depois.

## Por que não está READY

Os 13 prompts compartilhados atuais governam 44 consumers. Uma bijeção exige 31 prompts
proprietários adicionais. Copiar mecanicamente os L0 produziria contratos formalmente
distintos mas semanticamente falsos; decidir como decompor `linter-core`,
`refinement-validator`, `wildcard-saturation`, `fix-hashes` e os demais requer autoria de
arquitetura.

O novo bloqueio também impede o resselamento oficial do próprio contrato V15 até essa
individualização. Isso é comportamento fail-closed esperado, não regressão do reparador.

Classificação final:

- C: implementado e verde;
- D: `SPEC-GAP` semântico confirmado;
- P0104: `BLOCKED`;
- merge, instalação e migração Typst: não autorizados e não executados.

## Próximo passo necessário

Escrever um passo separado de individualização semântica dos 13 grupos. Cada novo prompt
deve declarar somente a responsabilidade real de um código, ser confrontado antes do
resselamento e entrar em commits pequenos. Só depois V1/V5/V7/V15 e `--fix-hashes` podem
voltar a ficar integralmente verdes.
