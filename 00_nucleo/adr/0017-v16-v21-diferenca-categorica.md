# ADR — V16 (saturação de wildcard) não é silenciável por citação; V21 (escalar sem proveniência) é. A diferença é categórica, não uma inconsistência a corrigir.

**Estado**: proposto (repositório `tekt-linter`).
**Contexto**: achado ao reconciliar V16 (P1041/1046/1049, `typst-crystalline`) contra
V21/V22 (Passo 0066-0068). As duas regras tratam "algo sem justificação suficiente" —
mas uma (V21) silencia com citação `// ref:`, a outra (V16) não. Isto pareceu
inconsistência à primeira vista; não é.

---

## A distinção, formalizada

**V21 vigia um facto estático.** Um literal numérico (`1.2`, `0.5em`) tem um valor de
verdade fixo, determinado por uma fonte externa que não muda com o tempo do projecto
(a especificação do vanilla, uma tabela OpenType). Uma vez citada a fonte, a afirmação
"este valor está correcto" permanece verdadeira até a própria fonte mudar — o que é
raro e, quando acontece, é detectável por re-auditoria pontual (mesmo mecanismo que
apanhou as citações obsoletas do P1054, quando ficheiros foram movidos).

**V16 vigia um invariante sob evolução futura.** Um braço `_ => <default>` não é uma
afirmação sobre um valor fixo — é uma afirmação sobre **todas as variantes actuais** de
um enum. A correcção dessa afirmação depende do que o enum *tem hoje*, e o enum pode
crescer. Quando cresce, a nova variante cai no `_` sem qualquer sinal de que precisa de
ser revista — o wildcard, por desenho, esconde exactamente a informação (quantas
variantes, quais) de que a revisão precisaria.

## Fundamentação (literatura estabelecida, não convenção interna)

- **Wadler, "The Expression Problem"** (1998) — a tensão entre extensibilidade de casos
  e exaustividade de operações é conhecida há décadas; um catch-all resolve a tensão a
  favor da extensibilidade silenciosa, ao custo da exaustividade.
- **Verificação de exaustividade em tipos soma** (Rust `match` sem `_`, `sealed trait`
  em Scala, variantes em OCaml) — o mecanismo existe precisamente para forçar revisão em
  cada ponto de uso quando o tipo ganha um caso novo. Um `_ =>` remove essa obrigação
  deliberadamente.
- **`#[non_exhaustive]` do próprio Rust** — reconhece formalmente esta categoria:
  marca um tipo como "pode crescer", e o compilador exige `_` em qualquer `match` sobre
  ele — mas mesmo aí, a documentação da linguagem recomenda tratar esses `_` como pontos
  de vigilância, não como caso resolvido.

**Consequência directa**: silenciar V16 por citação estaria a tratar um invariante
sob-evolução como se fosse um facto estático — categoria errada. A citação em V16
(`// neutro: <razão>`) serve para **auditoria humana no momento em que uma variante
nova aparece**, não para desligar o aviso. O aviso é o próprio mecanismo de garantir que
essa auditoria aconteça.

## Decisão

1. **V16 nunca silencia por citação.** O nível (`warning`/`info`, conforme a
   sub-classe já existente em `wildcard_saturation.rs`) mantém-se sempre visível — é
   sinal permanente, não dívida a fechar.
2. **A anotação existente não é inútil** — não passa a ser decorativa. Serve de
   **taxonomia** para quem revê o aviso saber, sem reler o código, que classe de risco
   está ali. Formalizar essa taxonomia (secção seguinte) em vez de prosa livre.
3. **V21 mantém-se como está** (citação silencia) — a distinção fica documentada aqui
   para não voltar a ser lida como inconsistência a "corrigir" num passo futuro.

## Taxonomia simplificada para a anotação de V16

Substituir `// neutro: <razão de uma linha>` (prosa livre) por uma tag de classe +
razão curta, mais rápida de auditar em bloco:

```
// N16[α]: <razão> — impossibilidade estrutural (as variantes restantes são
  provavelmente inatingíveis neste ponto, por invariante já garantido a montante)
// N16[β]: <razão> — comportamento uniforme genuíno (as variantes restantes partilham
  o mesmo tratamento por desenho, não por coincidência — candidato a virar hub explícito
  se crescer, ADR-0104)
// N16[γ]: <razão> — fallback deliberado, aberto por natureza (a classe de maior risco
  — nunca "resolvida", só justificada para o conjunto actual; é aqui que o aviso mais
  vale a pena manter vivo)
```

`γ` é a classe onde o aviso é mais valioso — é exactamente onde uma variante nova é mais
provável de precisar de tratamento próprio, não de cair no default por acidente.

## Fora de âmbito deste ADR

Reclassificar as 204 anotações já existentes de `// neutro:` para a taxonomia
`N16[α/β/γ]` — passo próprio, mecânico, no `typst-crystalline`, depois deste ADR
aprovado.
