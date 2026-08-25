# Assessment 0033 — artefato Núcleo Tekt

**Estado:** A CONGELADO — produção proibida até REDs B1–B5
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
