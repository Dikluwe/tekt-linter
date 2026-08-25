# Assessment 0033 — artefato Núcleo Tekt

> **Nota de supersessão (P0107):** este Assessment preserva a evidência P0105 da extensão
> original `.tekt`. ADR-0022 Rev. 1 substitui apenas a representação física por `.toml`;
> claims, framing, hashes e gates permanecem históricos válidos.

**Estado:** READY WITH RESIDUAL AUDIT — implementação verde; integração bloqueada por P0104/P0106
**Data:** 2026-08-25
**Passo:** P0105
**Baseline:** `4765cd5`

## Autoridade

| Unidade | SHA-256 |
|---|---|
| P0105 | `0af7879e212715042351bd7bd45071f5486bb7bc8f2dccce5be10493f93ebf22` |
| ADR-0022 | `5652cc708edbd5bf943904dc20cd3caad4129418a909f72e8d31f241808db53e` |
| prompt proprietário | `0419750faf2b6e5b58ee879b6acb6d3e391550320cd1fb46bcde68529b3f4650` |

ADR-0022 define a relação permanente; o prompt define a materialização; P0105 define
protocolo, fronteiras e gates. Divergência futura bloqueia B; não há resselamento tácito.

## Fórmula byte-exata v1

`clean_prompt(bytes)` remove somente a linha autorizada `Hash do Código: `, usando a
mesma função vigente. Sem dependências, o digest permanece `SHA256(clean_prompt)[0..8]`.

Com dependências, usar frames com comprimentos unsigned big-endian de 64 bits:

```text
P = clean_prompt
effective_prompt_full = SHA256(
  P || 00 || "TEKT-PROMPT-NUCLEI-V1" || 00 ||
  for dep sorted by path bytes:
    u64be(len(path)) || path || 20 || effective_nucleus_full
)
@prompt-hash = first 8 lowercase hex chars of effective_prompt_full

effective_nucleus_full = SHA256(
  raw_tekt_bytes || 00 || "TEKT-NUCLEUS-DEPS-V1" || 00 ||
  for dep sorted by path bytes:
    u64be(len(path)) || path || 20 || effective_dependency_full
)
```

`effective_*_full` no frame é o digest binário de 32 bytes, não seus 64 caracteres hex.
Núcleo sem dependências ainda recebe o domínio `TEKT-NUCLEUS-DEPS-V1`; prompt sem
dependências usa o algoritmo legado sem domínio. Ciclo ou missing não possui digest.

O pin armazenado no Markdown é o SHA-256 efetivo completo do núcleo. Portanto mudança
transitiva invalida o pin mesmo se os bytes do núcleo diretamente citado não mudarem.

## Limites normativos

- arquivo `.tekt`: 1 MiB;
- nós no grafo: 16.384;
- dependências por núcleo: 256;
- profundidade: 256;
- claims por núcleo: 1.024;
- statement: 1..2.048 bytes;
- title: 1..160 bytes;
- id: `[a-z][a-z0-9-]{0,63}`;
- SHA pin: exatamente 64 hex ASCII minúsculos.

## Hipóteses RED

| ID | Hipótese |
|---|---|
| R1 | não existe parser/tipo estrito para `.tekt` |
| R2 | prompt walker não expõe referências de núcleo |
| R3 | não existe grafo/DAG/órfão V26 |
| R4 | V5 calcula somente bytes diretos do Markdown |
| R5 | fix-hashes não planeja pins transitivos |
| R6 | `.tekt` pode ser confundido com prompt ou ignorado pelo inventário |

## SPEC-GAP fechado

A fórmula acima decide domínio, framing, ordenação e digest binário. Claims não são lógica
executável. Nenhum implementador está autorizado a adicionar semântica além da validação
estrutural.

## Segregação

B1–B5 materializam expectativas exclusivamente deste Assessment, ADR e prompt. Produção
correspondente não pode ser usada como oráculo. REDs são congelados em commit anterior a
C. Projetos externos permanecem somente leitura.

## Gates congelados

Commit `461414c` precede toda produção:

| Gate | SHA-256 congelado | RED inicial |
|---|---|---|
| B1 formato | `43c51b576796f17052f2e6305f87b214f45df206d53c983c0c08fd7c428e4773` | módulo/parser ausente |
| B2 grafo | `cadae9cb763f2a9cc266a95328be494d9bc79b5c2a88a90f3ec53ed4caa7be84` | regra/tipos ausentes |
| B3 hash | `4c5b69ab0a2ce70d4aabaf204d4abb0fda28234db31efa2802a59e0141d9251f` | funções de digest ausentes |
| B4 wiring | `e0a8c9da3c0e73acdbad705df5213c5afc39d6cfebf1c9b92c8d638971a6b06b` | código→`.tekt` não rejeitado |
| B5 transação | `18bfc2cd01726e9c69bfa7d7a38ede998f02d5bb9eaab092d910c381118b47e8` | seam herdada já compilava; faltava integração com pins |

B4/B5 foram fortalecidos depois de C sem alterar expectativas congeladas: mudança de um
byte passou a exigir duas V5 + duas V26 e o reparo real passou a provar atualização de pins
e hashes numa transação. Hashes finais: B4
`51a94a55f6654aaff4eaf6f335cddc2fcd4c8a8c54d00af40762cdc46d9144db`, B5
`0cebbe06d5b8be7bc8ca105b5675953806ebea10dbabe2d463735579abc10bd9`.

## Implementação e classificação

- `6ee2463`: parser TOML estrito, hashing transitivo, grafo L1, V26, CLI/SARIF e wiring;
- `c64b785`: atualização transacional dos pins e leitor fresco na segunda passagem;
- `17358d4`: walker e symlinks fail-closed, bloco Markdown posicionado e não vazio.

R1–R6 foram confirmados como ausência de produção e fechados. A suíte integral passou com
630 unitários, 83 fixtures e todos os integration gates. V26 do próprio linter está limpa.

Durante o gate real foi encontrado RED adicional: o re-run de `fix-hashes` reutilizava o
cache anterior às escritas e reportava duas derivas falsas. Classificação: RED de produção
preexistente na segunda passagem; fechado por leitor fresco em `c64b785`.

## Residuais

1. Não houve agente cognitivamente independente; a segregação é causal/Git.
2. O piloto semântico real foi adiado: nenhum compartilhamento pequeno foi individualizado
   sem decidir P0106.
3. Os novos headers permanecem sem resselo oficial porque o reparador corretamente bloqueia
   nos 13 V15 históricos. P0106 deve individualizar esses prompts antes de merge/instalação.

P0105 está funcionalmente pronto, mas o branch não é merge-ready enquanto o bloqueio
herdado de P0104 não for sanado.
