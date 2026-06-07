# Prompt: verificar a cegueira cross-crate do V3 (a premissa da opção 3)

**Onde roda**: no **clone do `tekt-linter`** (de 0049), não no `tekt-cargo-dsm`.
**Criado em**: 2026-06-06
**Estado**: `PROPOSTO`
**Natureza**: **verificação, não conserto.** Confirmar ou refutar uma afirmação
antes de investir em mudá-la. Nada do linter de produção muda — só leitura de
fonte e um teste/fixture de verificação (descartável) e o laudo.
**Pré-requisito**: o clone local do `tekt-linter`.

---

## Por que este prompt existe

No 0050, o V14 subiu por imports **intra-L1** (`use lente_core::…`) e foi
contornado listando os seis `lente_*` no `[l1_allowed_external]`. A partir disso,
**eu (Claude) afirmei, por inferência**, que a mesma causa — o `resolve_layer` só
entender `crate::`/`super::`, tratando todo `use lente_*::` como `Layer::Unknown`
— deixa o **V3 (direção de import) cego a violações de direção ENTRE crates**.

Essa afirmação **não está confirmada**. É dedução. Antes de escolher a opção 3
(consertar o linter), confirmar se a premissa é verdadeira. Se for falsa, a opção
3 muda de escopo.

---

## A afirmação a testar (em partes)

1. **Mecanismo**: `resolve_layer` mapeia `use lente_X::…` (crate first-party de
   outra camada) para `Layer::Unknown` — só resolve `crate::`/`super::`.
2. **Consequência (o ponto central)**: o **V3** (`ForbiddenImport`) **não dispara**
   numa violação de direção entre crates — por exemplo, um arquivo **L3** com `use
   lente_wiring::Algo;` (sendo `lente_wiring` **L4**; L3 não pode importar L4).
3. **Corolário**: o que pega import cross-crate não autorizado no L1 é o **V14**,
   não o V3 — por isso a whitelist do 0050 virou o guarda de fato da fronteira do
   L1.

---

## O que fazer

### 1. Leitura de fonte (o mecanismo)

- `rs_parser::resolve_layer` — confirmar que retorna `Unknown` para um path de
  crate que não seja `crate::`/`super::` (ou seja, `use lente_*::`). Citar o
  trecho.
- `forbidden_import.rs` (V3) — **o ponto central**: ler **como** decide
  "proibido". Ele age sobre a **camada resolvida** do import? E quando a camada é
  `Unknown` — ignora, ou sinaliza? Citar o trecho que decide isso.
- a lógica do **V14**/external — confirmar que import cross-crate (`Unknown`) num
  L1 não-whitelisted dispara V14 (explica por que a whitelist resolve). Citar.

### 2. Teste empírico **com controle** (a parte decisiva)

A leitura de fonte sugere o comportamento; o teste **prova**. Escolher o **mais
leve que decida** — fixture ponta-a-ponta **ou** teste de unidade:

- **Fixture ponta-a-ponta** (preferido — testa o pipeline real): um mini-projeto
  Cristalino descartável (em `/tmp` ou como fixture do linter) com um crate **L3**
  e um crate **L4**, e no L3 um `use lente_<l4>::X;` — violação de direção L3→L4
  **entre crates**. Rodar `crystalline-lint`. **O V3 dispara?**
- **Alternativa (teste de unidade)**: alimentar o V3 (regra L1 pura) com um
  `ParsedFile` sintético cujo import tem camada `Unknown` (simulando cross-crate);
  checar se o V3 produz violação. Mais o teste do parser confirmando que `use
  lente_<l4>::X` vira `Unknown`.

**O CONTROLE é obrigatório** — sem ele, "o V3 não disparou" não distingue "cego a
cross-crate" de "V3 quebrado/desligado". Incluir, no mesmo teste, **uma violação
que o V3 CONSEGUE resolver** (ex.: um import **intra-crate** que o
`[module_layers]` mapeia para uma camada proibida — `use crate::shell::…` num
arquivo L1, se `shell`→L2). O V3 **deve** disparar nesse. Os dois lado a lado:

| Caso | Esperado se a premissa estiver certa |
|---|---|
| cross-crate proibido (L3 → L4 via `use lente_*::`) | V3 **silencioso** |
| intra-crate proibido (resolvível) — controle | V3 **dispara** |

### 3. Bônus (barato) — o falso positivo do `Kind`

Confirmar o mecanismo: um arquivo L1 com `use EnumLocal::*;` — o V14 dispara
lendo `EnumLocal` como pacote externo? Citar o trecho que causa isso.

---

## A conclusão a entregar

- **Se** o V3 ficar silencioso no cross-crate **e** disparar no controle →
  **premissa confirmada**: o `resolve_layer` é cego a cross-crate, o V3 tem o
  buraco, a whitelist é o contorno. A opção 3 se justifica.
- **Se** o V3 **disparar** no cross-crate → **minha afirmação estava errada**: o
  V3 já cobre cross-crate por outro mecanismo. A opção 3 encolhe (sobraria só o
  falso positivo do `Kind`). Dizer isso com todas as letras.
- **Se** o controle **não** disparar → o teste está mal montado (o V3 não estava
  ativo/resolvendo); refazer antes de concluir.

Reportar o **veredito com a evidência**: os trechos de fonte + a saída do linter
nos dois casos.

---

## O que NÃO fazer

- **Não consertar** o `resolve_layer`, o V3, o V14, nada do linter de produção.
- Não tirar conclusão sem o controle.
- Não tocar o `tekt-cargo-dsm` (este prompt é diagnóstico do linter).

---

## Critérios de Verificação

```
Dado o resolve_layer e o V3 (e o V14)
Quando lidos na fonte
Então o que cada um faz com import cross-crate (Unknown) está citado

Dado um caso cross-crate proibido E um caso resolvível (controle)
Quando o linter roda
Então a saída do V3 nos dois está registrada

Dado os dois resultados
Então o veredito sobre a premissa da opção 3 é declarado: confirmada (V3 cego) ou
refutada (V3 já cobre), com a evidência

Dado o controle
Então ele DISPAROU (senão o teste é inválido e foi refeito)

Dado o linter de produção
Então NADA mudou — só leitura + fixture/teste descartável
```

---

## Resultado esperado

- O trecho do `resolve_layer` (cross-crate → `Unknown`? confirmado/refutado).
- O trecho do V3 (o que faz quando a camada do import é `Unknown`).
- O trecho do V14 (por que cross-crate L1 dispara).
- A saída do linter no fixture: V3 no caso cross-crate **e** no controle.
- O **veredito** sobre a premissa da opção 3, com a evidência — incluindo "minha
  afirmação estava errada" se for o caso.
- O mecanismo do falso positivo do `Kind` (bônus).
- **Laudo** em `00_nucleo/lessons/0051-…` (do `tekt-cargo-dsm`, que é o registro
  do fluxo) descrevendo o diagnóstico do linter.

---

## Cuidados

- **Medir antes de afirmar e antes de consertar.** A cegueira do V3 é **inferência
  minha**, pode estar errada — o objetivo é confirmar ou refutar com honestidade,
  não validar uma conclusão pré-feita.
- **O controle é o que dá rigor** — "o V3 não disparou" sem um caso que dispara
  não prova nada.
- **Nada consertado** — só diagnóstico. O conserto (se a premissa se confirmar) é
  passo seguinte e decisão à parte.
- **Fixture descartável** — se ficar no repo, marcar como verificação.

---

## Histórico de Revisões

| Data | Motivo | Arquivos afetados |
|------|--------|-------------------|
| 2026-06-06 | Verificação (não conserto) da premissa da opção 3, antes de investir em mudar o linter. Afirmação de Claude (por inferência, a confirmar): o `resolve_layer` só resolve `crate::`/`super::`, então `use lente_*::` cross-crate vira `Layer::Unknown`, e em consequência o **V3 (direção de import) fica cego a violações de direção entre crates** (o que pega cross-crate no L1 é o V14, contornado pela whitelist do 0050). Método: leitura de fonte (`resolve_layer`, `forbidden_import.rs`/V3, lógica do V14) **mais** teste empírico **com controle** — caso cross-crate proibido (L3→L4 via `use lente_*::`, esperado V3 silencioso) lado a lado com caso intra-crate resolvível (controle, esperado V3 dispara); sem o controle disparar, o teste é inválido. Bônus: confirmar o mecanismo do falso positivo do `Kind` (`use EnumLocal::*` lido como externo). Veredito declarado com evidência — incluindo "afirmação errada" se o V3 cobrir cross-crate. **Nada do linter de produção mudado**; só leitura + fixture/teste descartável. Roda no clone do `tekt-linter`; laudo no `00_nucleo/lessons/` do `tekt-cargo-dsm`. | (no clone do `tekt-linter`) fixture/teste de verificação descartável; `00_nucleo/lessons/0051-verificar_v3_cross_crate.md` |
