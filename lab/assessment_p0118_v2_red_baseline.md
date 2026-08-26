# Assessment P0118 — baseline RED de V2

Data: 2026-08-26  
Baseline do produto: `961030bb3520acfd74b53858cf55020bfebb8ffb`  
Passo protegido: `fa13c368525d952ee1c102c32e3f47673e5f98fa` antes do fechamento nominal das razões

## Comando

```sh
/home/dikluwe/.cargo/bin/crystalline-lint /repos/Antigravity/Tradução --checks v2
```

Resultado congelado antes do primeiro patch funcional: exit `1`, exatamente cinco V2:

1. `01_core/algorithms/test_entities.py`
2. `01_core/algorithms/test_glossary.py`
3. `01_core/algorithms/test_lexical.py`
4. `01_core/algorithms/test_quality_estimation.py`
5. `01_core/algorithms/test_veto.py`

SHA-256 da saída combinada stdout/stderr:  
`70b715a7c1e4e68c67f46b6f576d0a542e3f1a6c51bdbf576950f0f0ac3e3ea4`

## Hashes do corpus

| Arquivo | SHA-256 |
|---|---|
| `test_entities.py` | `d8517c832c01d357a8f356c261d6e23082c132319fb992abd842e59e74de3b6a` |
| `test_glossary.py` | `e5b7b2d34802bb8d01e36f71545c43196dea95c8a2415547da8081b3d1fdfee8` |
| `test_lexical.py` | `e380d8dbe0dcc868c9b3bb5a57b28389cb1819a94836ac8609c883ad444f1266` |
| `test_quality_estimation.py` | `243bc826b47030d761c6999e89c703a704adcdce1b308414cc1826a821272383` |
| `test_veto.py` | `e7d256f2c6509e5d33b445b95fffae5d8616f07f7256e42579df45c1f5f28d65` |

O repositório Tradução é evidência somente leitura. P0118 não autoriza nenhuma escrita nele.
