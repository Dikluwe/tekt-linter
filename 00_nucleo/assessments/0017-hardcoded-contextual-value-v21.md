# Assessment 0017 — HardcodedContextualValue V21

**Estado:** CONGELADO PARA TRIAGEM SEGREGADA
**Data:** 2026-08-24
**Passo:** P0088
**Baseline:** `ccca54941e4c2043f1e4557abcd09c08697a3a8b`
**Alvos previstos:** classificador V21 L1 e seam/adapter de frescura L3

## Insumos normativos autorizados

| Unidade | Caminho | SHA-256 |
|---|---|---|
| V21 | `00_nucleo/prompts/unsourced-constant.md` | `e179f9d134956c3bdaae8f57938d25096b2f4bc95bc85bdb246845ec4d9ed05a` |
| IR | `00_nucleo/prompts/contracts/rule-traits.md` | `53c622749e5447385b6bc47a7c4410a21d4b6d27ae459f2d307e3742a863c2c1` |
| porta de frescura | `00_nucleo/prompts/contracts/citation-freshness.md` | `7b2e4f285dada19b500d3d653bde3d12ff1fa0f05d1d6a34039deff70e7efb5c` |
| arquitetura/wiring | `00_nucleo/prompts/linter-core.md` | `ed44ffdda0a323df26a25cef40c0acb46bd692db6fdaef861a20a509adeb7029` |
| distinção V16/V21 | `00_nucleo/adr/0017-v16-v21-diferenca-categorica.md` | `79f406654aacf3693616232a4fdbb911e359486d089ffde841af5375625104dd` |
| protocolo | `00_nucleo/tekt-linter-passo-0088-triagem-v21.md` | `b1df39f121022b2b6d8a0a47f9e1c25202af055a4b9f50cea6c7c0839997b3a1` |

## Natureza e bloqueio inicial

Assessment retroativo. O L0 agora fecha o anti-apodrecimento de `// ref:` por porta
causal explícita e política executável `valid/stale/unknown`, preservando a proibição de
filesystem em L1. O I/O direto observado no classificador permanece `RED` arquitetural;
produção só pode mudar depois de gates cegos congelados contra estes hashes.

## Alegações L1 a congelar

1. V21 é silenciosa fora de Rust e em coleção vazia.
2. O predicado exige simultaneamente scaling binário, fonte contextual e sumidouro
   geométrico; ausência de qualquer eixo é silenciosa.
3. Format modules, test-origin, data-table e literais triviais são isentos exatamente
   segundo configuração/L0; paths e nomes parecidos não ganham isenção acidental.
4. Sem citação, ocorrência elegível gera V21 Warning ou Error em strict module.
5. `Spec` e `Rationale` válidas silenciam; `Ref` só silencia sob frescura `valid`.
6. `stale` gera diagnóstico explícito; `unknown` nunca vira válido nem silêncio.
7. Mensagem preserva literal, context var, sink e modalidade relevante; location vem da
   constante. Ordem e multiplicidade de ocorrências são preservadas.
8. Configuração e campos irrelevantes não causam inferência adicional; V21 nunca emite
   V22 nem acessa filesystem, rede, relógio, ambiente ou processo.

## Alegações da porta/adapter L3

1. A porta expõe estado fechado `valid | stale | unknown`, sem bool que confunda erro
   externo com validade.
2. Arquivo e linha existentes/não vazios são `valid`; path ausente, linha zero, além de
   EOF ou linha vazia são `stale`.
3. Entrada fora da raiz, symlink escape, erro de leitura, orçamento/encoding não
   suportado são bloqueados como `unknown` ou erro explícito conforme L0 saneado.
4. Resolução é read-only, confinada, determinística, sem rede, hooks ou escrita.
5. L3 implementa a porta; não duplica listas, predicado contextual, severidade ou
   formatação de diagnóstico V21.

## Gates segregados

- **B1 L1:** matriz de languages; produto dos três eixos; filtros/configuração; triviais;
  citações e três estados; strict; evidência; ordem/multiplicidade; mock puro sem I/O.
- **B2 L3:** filesystem temporário; linhas válida/vazia/ausente/extrema; confinamento,
  symlink, Unicode, orçamento e imutabilidade; sem importar classificador como oráculo.
- **C:** confronto somente após gates congelados.
- **D:** adversário final verifica causalidade L0→porta/L1/L3, gravidade, gates,
  regressão V22 e ausência de delta escondido em parser/config/CLI.

Resultados possíveis: `PASS`, `RED`, `SPEC-GAP`, `GATE-DEFECT`. O lote fecha somente com
`READY WITH RESIDUAL AUDIT` ou `BLOCKED`, relatório P0088 e nenhum merge.
