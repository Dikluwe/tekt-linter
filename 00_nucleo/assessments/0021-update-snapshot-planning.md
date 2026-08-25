# Assessment 0021 — planejamento e execução de update-snapshot

**Estado:** PREFLIGHT — SPEC-GAP congelado; produção ainda não confrontada
**Data:** 2026-08-24
**Passo:** P0092
**Baseline:** `aee1344`
**Commit do protocolo no branch:** `83ad45e`

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| fluxo fix/update | `00_nucleo/prompts/fix-hashes.md` | `7933d862fa3b27f4fb0cda36c654c96041c2b85ef5141e202717a770d3a138c7` |
| contrato snapshot | `00_nucleo/prompts/contracts/prompt-snapshot-reader.md` | `94ba7d51f32778d7bf74a89b16920e3b22078ebc79a7a6cd70cbad767f6add21` |
| tipos V6/IR | `00_nucleo/prompts/violation-types.md` | `147afa0d8f3f3e6e30e050590dad0b99c7da8486d3565e3f6c42f7fa883ea4dc` |
| apresentação/CLI | `00_nucleo/prompts/sarif-formatter.md` | `2d4ccf4d260337199146ce75b8b80a8b51d01cb685273b2aab5d7b9ae8d733a7` |
| arquitetura do pipeline | `00_nucleo/prompts/linter-core.md` | `908a00fd7e4eaa985b755682fb73984cbb886496ce988070f176ad307ec24446` |
| protocolo segregado | `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| ADR segregado | `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| protocolo P0092 | `00_nucleo/tekt-linter-passo-0092-auditoria-planejamento-update-snapshot.md` | `1a37cbf51d7a35f4d789c5985160b690c3e785bd7748b876b5a6a44228650deb` |

## Alegações candidatas

1. `plan` aceita somente `rule_id == "V6"`, produz uma entrada por violação e preserva
   ordem e duplicatas.
2. A associação com `ParsedFile` usa igualdade integral do path, sem normalização.
3. Ausência de arquivo parseado e ausência de header produzem razões distintas e não
   chamam serialização.
4. Entrada acionável preserva source path/prompt path e serializa exatamente a interface
   do arquivo associado uma única vez.
5. `dry_run` não escreve e continua observável por entrada.
6. Execução real escreve cada entrada acionável exatamente uma vez, na ordem recebida.
7. `Ok`/`Err` são preservados por entrada e falha não interrompe itens posteriores.
8. Entrada não acionável permanece observável também após `execute`.
9. L2 não acessa filesystem, ambiente, relógio, rede ou processo.

## Evidência normativa presente

- `fix-hashes.md` atribui decisão a L2 e escrita a L3.
- A norma exige filtro exato V6, dry-run sem toque no disco e `plan()` sem descarte por
  `filter_map`.
- Falhas de leitura devem carregar `unreadable_reason` em vez de desaparecer.
- `sarif-formatter.md` exige delegação da serialização/escrita via port L2/L4.
- `prompt-snapshot-reader.md` fixa a serialização canônica, mas não o planejamento.

## SPEC-GAPs congelados

### G1 — API e nome do port divergentes

`fix-hashes.md` publica `SnapshotWriter` com `read_interface`, `serialize` e
`write_snapshot(prompt_path, interface)`. O restante do L0 alterna `SnapshotWriter` e o
passo candidato usa `SnapshotRewriter` com serialização separada. Não há uma API única
hash-pinned para gate black-box.

### G2 — tipos de plano/resultado ausentes

O L0 não publica `SnapshotEntry` ou `SnapshotResult`, seus campos, igualdade nem estados
válidos. B1/B2 não podem inventar representação de erro, dry-run ou sucesso.

### G3 — duplicatas e associação ambíguas

Não há precedência para múltiplos `ParsedFile`s com o mesmo path. Também não está dito se
cada V6 duplica uma entrada ou se prompts/paths devem ser deduplicados.

### G4 — entrada não acionável após planejamento

“Nunca descartar” é explícito para `plan`, mas o L0 não decide se `execute` retorna falha
para essa entrada, a omite deliberadamente ou delega sua observabilidade apenas ao
formatter do plano.

### G5 — semântica de dry-run e continuidade

O L0 proíbe escrita em dry-run, mas não define o `SnapshotResult` correspondente. Ordem
de chamadas, chamada única e continuação após `Err` também não estão publicadas.

## Decisão de preflight

Não construir gates nem ler `02_shell/update_snapshot.rs` até G1–G5 serem saneados e os
hashes resselados. A decisão recomendada, sujeita ao adversário A, é:

- port único `SnapshotRewriter` em L2, com serialização e escrita explicitamente
  separadas;
- `SnapshotEntry` e `SnapshotResult` públicos, comparáveis e com estado explícito;
- uma entrada por V6, ordem/duplicatas preservadas e primeiro `ParsedFile` de path exato;
- ausência de parsed/header como entrada não acionável que vira resultado de falha sem
  chamada ao port;
- dry-run como resultado explícito distinto de escrita realizada;
- execução estável, uma chamada por acionável e continuação após falha;
- zero I/O em L2; apenas o port pode produzir efeito externo.

## Papéis

- A: adversário somente Assessment/L0 hash-pinned;
- B1: gate de planejamento após saneamento, sem produção;
- B2: identidade e arquivo separados para execução, sem B1/produção/L3;
- C: confronto somente após B1/B2 congelados;
- D: fechamento adversarial de causalidade, arquitetura, regressão e delta.

Resultados válidos: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. Fechamento somente como
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, sem merge ou push.
