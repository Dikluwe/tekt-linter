# P0077 — revisão adversarial final (Agente C)

## Veredito

**NÃO REABRIR.** Os dois REDs de V1 foram fechados: o escopo L1–L4 agora integra a trait pura e a referência inexistente preserva literalmente sua identidade em uma mensagem distinta. O probe independente passou em 6/6 ataques.

Escopo lido: passo P0077 integral, prompts causais finais de V1/V15, produção final de V1, traits e implementação necessária em `ParsedFile`. Não foi lido `tests/lineage_header_classifiers_assessment.rs`, nem mensagens ou artefatos do Agente B. Produção e gate não foram alterados.

## Ataques executados

| # | Ataque | Critério mecânico | Resultado |
|---|---|---|---|
| P1 | Produto cartesiano das sete camadas × três estados (`None`, `Some+missing`, `Some+exists`). | L1–L4 geram uma V1 nos dois estados inválidos e vazio no válido; L0/Lab/Unknown retornam vazio nos três estados. | **PASS** |
| P2 | Dublê das camadas isentas cujo acesso a header, existência ou path causa `panic!`. | As três chamadas retornam vazio sem panic, provando que a isenção antecede causa, path e política strict. | **PASS** |
| P3 | Comparar header ausente com referências inexistentes contendo NFC, NFD, caixa distinta, newline, tab e NUL representável. | As causas têm mensagens distintas e cada `prompt_path` hostil aparece literalmente na evidência, sem normalização. | **PASS** |
| P4 | Strict dir por componentes contra diretório exato, descendente e prefixos textuais próximos; repetir para ambas as causas. | Diretório/descendente são `Fatal`; `contracts_extra` e `contract` são `Error`, independentemente da causa. | **PASS** |
| P5 | Quatro camadas aplicáveis × path strict/non-strict. | Exatamente uma V1, rule id correto, path preservado, `(line,column)=(1,0)` e nível `Fatal`/`Error` correto. | **PASS** |
| P6 | Regressão V15 nas sete camadas e cardinalidades 0–2, mais três refs com duplicata e NFC/NFD. | Escopo e limiar permanecem corretos; uma única `Error` conserva quantidade, ordem, duplicata e Unicode. | **PASS** |

## Evidência da correção

`HasPromptFilesystem` agora expõe `layer()`, e `ParsedFile` o implementa retornando sua camada já classificada. V1 começa pelo `matches!` de L1–L4 e retorna imediatamente para as três camadas isentas.

Nas camadas aplicáveis, o `match` separa os estados:

- `None` mantém a mensagem histórica;
- `Some(header)` com existência falsa produz mensagem de referência inexistente contendo `header.prompt_path`;
- `Some(_)` com existência verdadeira retorna vazio.

A severidade continua derivada por `Path::starts_with`, que respeita fronteiras de componentes para os casos congelados. A construção única após o `match` conserva cardinalidade, localização e rule id.

## Reprodução

Probe preservado fora do auto-lint: `lab/p0077_adversarial_final_probe.rs.txt`.

```sh
cargo build --lib
cp lab/p0077_adversarial_final_probe.rs.txt /tmp/p0077_adversarial_final_probe.rs
rustc --edition=2021 /tmp/p0077_adversarial_final_probe.rs \
  -L dependency=target/debug/deps \
  --extern crystalline_lint=target/debug/libcrystalline_lint.rlib \
  -o /tmp/p0077_adversarial_final_probe
/tmp/p0077_adversarial_final_probe
```

Saída observada:

```text
PASS P1 V1 exhaustive 7 layers x 3 states
PASS P2 exemption precedes header/exists/path/strict evaluation
PASS P3 distinct causes and literal hostile prompt evidence
PASS P4 strict component policy for both causes
PASS P5 V1 cardinality/id/location/level
PASS P6 V15 scope/threshold/Unicode/duplicate regression
```

## Limite residual

O probe trata `prompt_file_exists` como evidência já resolvida, conforme a fronteira congelada. Confinamento, canonicalização e acesso ao filesystem pertencem a L3 e não foram reimplementados nesta revisão de V1.
