# Classificador — sarif-formatter

**Estado:** CLASSIFIED

| Consumer | OWNER |
|---|---|
| `02_shell/cli.rs` | argumentos, catálogo V0–V26, ordenação, text/SARIF e exit policy |
| `02_shell/path_encoding.rs` | projeções humanas e URI machine-safe de paths não UTF-8 |

`CONTEXT`: CLI usa encoding de path. A semântica de path é exclusiva do segundo; catálogo e
apresentação são exclusivos do primeiro. Sem núcleo e sem contradição.
