# Prompt: Selo de materialização segregada

> **Estado:** VIGENTE — piloto autorizado pelo ADR-0020
> **Camadas:** L1–L4

## Intenção

Verificar mecanicamente que um contrato de refinamento foi congelado antes da solução
e demonstrou poder discriminatório em oráculos independentes. O comando produz recibo,
não prova formal de independência entre agentes.

## Interface

```bash
crystalline-lint seal-refinement <repository-root> \
  --manifest 00_nucleo/refinement/manifests/run.toml \
  --output seal.json
```

## Manifesto v1

```toml
protocol_version = 1
prompt = "00_nucleo/prompts/example.md"
prompt_sha256 = "<64 hex>"
baseline_oid = "<commit oid completo>"
contract = "00_nucleo/refinement/contracts/example.toml"
contract_sha256 = "<64 hex>"
contract_producer = "contract-agent/session-id"
implementation_producer = "implementation-agent/session-id"
verifier_producer = "mechanical-verifier/id"
unknown_policy = "block"

[[oracle]]
id = "preserves-valid-rewrite"
kind = "positive"
before_ref = "<oid>"
after_ref = "<oid>"

[[oracle]]
id = "rejects-field-removal"
kind = "negative"
before_ref = "<oid>"
after_ref = "<oid>"

[[oracle]]
id = "reports-opacity"
kind = "unknown"
before_ref = "<oid>"
after_ref = "<oid>"
```

Paths internos são relativos à raiz, confinados sem symlink escape. Hashes declarados
de prompt e contrato são SHA-256 sobre os bytes exatos capturados uma única vez. O
baseline deve ser OID completo e resolver para o mesmo commit.

## Resultado

O selo JSON é determinístico, sem timestamp, e contém:

- `protocol_version`;
- hash da representação semântica canônica do manifesto e hashes exatos de prompt e
  contrato;
- baseline OID;
- produtores declarados;
- recibos ordenados por `id`, com OIDs e veredito;
- objeto `counts` com contagens `positive`, `negative` e `unknown`;
- `mutation_score` representado sem ponto flutuante ambíguo;
- `sealed: true`.

Falha de hash, produtor duplicado, oráculo sem id, tipo inválido, veredito divergente,
`UNKNOWN` num negativo, orçamento ou entrada Git inválida termina com exit 2 e não
publica selo parcial.

## Invariantes

- L1 não conhece TOML, JSON, Git, filesystem, SHA ou agentes concretos.
- O comparador e extrator de refinamento não são duplicados.
- Oráculos resolvem refs uma vez e registram OIDs completos.
- O manifesto e entradas nunca são reescritos.
- Saída é atômica.
- Ordem de campos e `[[oracle]]` no TOML não altera os bytes do selo. Para tornar isso
  compatível com a proveniência, `manifest_sha256` é calculado sobre uma representação
  semântica canônica (campos e oráculos ordenados), não sobre os bytes crus do TOML.
- Identidade nominal não é apresentada como prova de sandbox.
- Um negativo só é morto por `VIOLATED`.

## Fixtures RED

1. pacote válido com positivo, negativo e unknown produz selo;
2. negativo que retorna `PRESERVED` bloqueia;
3. negativo que retorna `UNKNOWN` bloqueia;
4. positivo violado bloqueia;
5. oráculo unknown preservado bloqueia;
6. hash de prompt ou contrato divergente bloqueia;
7. produtores repetidos bloqueiam;
8. baseline simbólico ou divergente bloqueia;
9. ordem diferente produz selo byte-idêntico;
10. falha não deixa arquivo temporário ou parcial;
11. working tree sujo permanece idêntico;
12. nenhum checkout, hook, filtro, LFS, submódulo ou rede é executado.

## Histórico

| Data | Estado | Motivo |
|---|---|---|
| 2026-08-24 | Vigente | Piloto autorizado para testar a ADR-0003 do Tekt no próprio linter |
