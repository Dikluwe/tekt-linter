//! Gate segregado do Assessment 0015 — preservação semântica V23–V25.
//!
//! Insumos L0 validados em 2026-08-24:
//! - V23 `context-erasure.md`:
//!   `09545244abfb7209cbd2d987322098365d5dd24e0b0569a0ca49b1f808ab9e3a`
//! - V24 `semantic-field-loss.md`:
//!   `f4b7593c8990a4817ce1c767d5f67c21856a908ce6220acb7774cc93edb84cab`
//! - V25 `decision-ownership.md`:
//!   `addf36ee1ede26f974f782e3a5c180344b99dd99af482a05c58c21f722681341`
//!
//! Propriedades congeladas antes de qualquer leitura de L1–L4:
//! 1. V23 diagnostica somente resolução contextual neutra não `absolute-only` e
//!    projeção apagadora que alcança sumidouro do mesmo contrato.
//! 2. V24 diagnostica somente slot obrigatório neutro sob `preserve`, sem
//!    dependência da origem; `drop-to-default`, ausência e opacidade são isentos.
//! 3. V25 diagnostica exatamente `duplicate-owner`, `proxy-reentry` e
//!    `canonicalizer-reentry`; chamada legítima ao owner, identidade distinta e
//!    operação anterior ao marco são isentas.
//! 4. As regras preservam ordem, multiplicidade e location das ocorrências
//!    elegíveis, mantêm isolamento por categoria e não inferem semântica de nomes.
//! 5. Campos irrelevantes, permutações equivalentes e duplicatas com semântica de
//!    conjunto não mudam o significado; ocorrências distintas não são colapsadas.
//! 6. Entrada vazia, Unicode, strings vazias, spans extremos e valores opacos são
//!    totais: não causam panic nem falso positivo.
//!
//! SPEC-GAP: o Assessment 0015 e os três L0 hash-pinned não publicam o caminho do
//! crate, os tipos/construtores do IR, as assinaturas dos classificadores, nem um
//! adaptador black-box autorizado. Materializar expectativas executáveis exigiria
//! ler produção/testes ou adivinhar uma API, ambos proibidos ao verificador.

compile_error!(
    "SPEC-GAP assessment 0015: falta API black-box publicada para V23/V24/V25; \n+     publique crate path, tipos/construtores do IR e assinaturas invocáveis antes \n+     de materializar o gate segregado"
);

