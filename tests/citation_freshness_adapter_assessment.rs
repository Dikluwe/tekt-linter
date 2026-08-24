//! Gate B2 segregado — adapter L3 de frescura de citações V21.
//!
//! Assessment: `00_nucleo/assessments/0017-hardcoded-contextual-value-v21.md`.
//! Identidade: verifier/v21-l3/0017.
//!
//! Este gate permanece deliberadamente fechado: os insumos L0 hash-pinned do
//! Assessment 0017 não publicam a porta nem a API completa de frescura. Em
//! particular, não existe contrato executável que permita instanciar o adapter,
//! observar `valid | stale | unknown`, configurar raiz/orçamento/encoding ou
//! distinguir erro explícito de `unknown` sem inventar política.
//!
//! Matriz B2 congelada para materialização após saneamento causal do L0:
//!
//! | Caso | Expectativa mínima já congelada |
//! |---|---|
//! | arquivo e linha existentes, conteúdo não vazio | `valid` |
//! | arquivo ausente | `stale` |
//! | linha zero | `stale` |
//! | linha além de EOF | `stale` |
//! | linha vazia | `stale` |
//! | entrada fora da raiz | nunca `valid`; `unknown` ou erro explícito conforme L0 |
//! | escape por symlink | nunca `valid`; `unknown` ou erro explícito conforme L0 |
//! | erro de leitura | nunca `valid`; `unknown` ou erro explícito conforme L0 |
//! | encoding/metadata não suportado | nunca `valid`; `unknown` ou erro explícito conforme L0 |
//! | Unicode suportado | resultado determinístico conforme conteúdo/linha |
//! | arquivo acima do orçamento | nunca `valid`; política exata pendente de L0 |
//! | duas resoluções da mesma entrada | mesmo resultado, sem mutação |
//! | qualquer resolução | somente leitura; zero rede, hooks ou escrita |
//!
//! Para abrir este gate, o L0 deve definir previamente: tipos públicos da porta,
//! assinatura de resolução, semântica fechada dos três estados, confinamento e
//! symlinks, orçamento, encoding/metadata e representação de erro. O teste então
//! deverá usar filesystem temporário e não poderá importar V21 como oráculo.

compile_error!(
    "SPEC-GAP B2/Assessment 0017: L0 ainda não publica porta e API completas de frescura de citações"
);
