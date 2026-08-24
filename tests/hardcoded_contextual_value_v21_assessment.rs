//! Gate cego B1 do Assessment 0017 — V21 `HardcodedContextualValue`.
//!
//! Identidade segregada: `verifier/v21-l1/0017`.
//! Este gate foi derivado exclusivamente do Assessment 0017 e dos quatro insumos
//! L0 hash-pinned nele autorizados. Nenhum arquivo de produção, teste histórico ou
//! relatório foi consultado.
//!
//! Os insumos congelam as propriedades abaixo, mas não publicam a API necessária
//! para exercitá-las black-box:
//! - silêncio fora de Rust e para coleção vazia;
//! - produto estrito dos eixos scaling × context-var × geometric-sink;
//! - filtros por identidade para módulo de formato, origem de teste e data-table;
//! - literais triviais exatos e controles próximos;
//! - None/Spec/Rationale/Ref combinados com valid/stale/unknown;
//! - Warning padrão e Error somente em módulo strict;
//! - evidência e location preservadas, com ordem e multiplicidade;
//! - somente V21, sem I/O ou inferência por campos irrelevantes.
//!
//! Para substituir este bloqueio por testes executáveis, o L0 causal deve publicar
//! integralmente, antes da implementação:
//! 1. assinaturas e campos observáveis de `V21RuleConfig`, `SourceConstant`,
//!    `Citation` e `HasConstants`;
//! 2. o enum fechado de frescura `valid | stale | unknown`;
//! 3. como a frescura é injetada no classificador puro;
//! 4. o resultado diagnóstico observável para Ref stale e Ref unknown;
//! 5. construtores/assinaturas públicas suficientes para mocks sem filesystem.
//!
//! Classificação: SPEC-GAP bloqueante. Inventar essas decisões no gate faria do
//! teste uma segunda especificação e violaria a causalidade Tekt L0 → L1.

compile_error!(
    "SPEC-GAP Assessment 0017/B1: L0 não publica integralmente a API black-box de V21 nem a seam pura de frescura; gate permanece fail-closed"
);
