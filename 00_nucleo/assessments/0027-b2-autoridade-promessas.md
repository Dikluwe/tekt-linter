# Assessment 0027/B2 — autoridade e promessas pós-P0097

**Papel:** B2, superfície normativa e documentação pública em somente leitura  
**Resultado:** `PASS WITH SPEC-GAPS`  
**Produção/testes lidos:** não  
**Parecer B1 lido:** não  
**Escolha de P0099:** proibida e não realizada

## Fronteira e método

Este parecer separa promessa vigente de exploração opcional para S1–S6 sem inferir
comportamento a partir da implementação. Prompts e ADRs decidem arquitetura e contrato;
`README.md` e `USAGE.md` tornam comandos, linguagens e exits promessas públicas;
fechamentos anteriores provam apenas o lote explicitamente encerrado. Uma ausência de
autoridade não foi convertida em obrigação, e uma contradição não foi convertida em
`ACCEPTED-RESIDUAL`.

## Autoridades consultadas e hash-pinned

| Autoridade | SHA-256 |
|---|---|
| `00_nucleo/prompts/refinement-validator.md` | `7061d609f14343f041bb28dbee4a89589a3d68161bdb9dfb63b3e461cafcae97` |
| `00_nucleo/adr/0019-validacao-direcional-de-refinamento.md` | `c2607ff2feb044487b454b3dc3115c9613d8124faebc415dc889eb717038e376` |
| `00_nucleo/prompts/segregated-materialization.md` | `366fd0855c6b04e533f4f4a477a73d7e5ec65f24c056720c61fca906bb5299a4` |
| `00_nucleo/adr/0020-piloto-materializacao-segregada.md` | `ee1a4a7f3665674b008d127373ed23fc6762d0ff13b2ca83efe5d2ace1539d23` |
| `00_nucleo/prompts/linter-core.md` | `9446277167f07dc5290617855cff456f061aa052ce8bd51ecf980530800b8c00` |
| `00_nucleo/prompts/contracts/language-parser.md` | `5d8a5db677dfba32be5228e643e1c1184905a0def86379aef40bab7640fa9588` |
| `00_nucleo/prompts/parsers/_template.md` | `4f253f4f7c277749a98ec3bf095c0b6602fc7797f8a20cfb0404d916c6a04563` |
| `00_nucleo/prompts/parsers/rust.md` | `80d1bb090717719befe293aba04b3ff22496f15caa5db1820827843c2fea796d` |
| `00_nucleo/prompts/parsers/typescript.md` | `18934531e5094b8269d9c5a9f65f7afeb7398129b11cc6f7cf462b6f742b5a62` |
| `00_nucleo/prompts/parsers/python.md` | `a899f55f5d5ef894a32ed4531e4f067ea8f7a40dc617bc27285b2e7dea3825f9` |
| `00_nucleo/prompts/parsers/c.md` | `3b5ca25f76dbb787a7d69be70d11c0f0689cf4abf7e17d8283d25a391d63e9fb` |
| `00_nucleo/prompts/parsers/cpp.md` | `f7d1956b72dfed5ea1784de2886697aa7a666b8a90974898dedf032a514f1c54` |
| `00_nucleo/prompts/parsers/zig.md` | `f622bc2c50ff63994315d71a71d5b14677a703c40853126d5e6a4bf298985bd8` |
| `README.md` | `3ff67521214cff672b54941e1d4392b2ab933c51ed69ecc9cf5e55e8989716d6` |
| `USAGE.md` | `245bc38db11a29467e7e72514f488fcb69fd471d401fa7eb1b6823355fa8d4f1` |
| Assessment P0097 | `26d94721dc5a0e6787f407859c27bf15e26e35b47b34611dd788e0cb3d4f30da` |
| fechamento P0097 | `fc3d130fb3794e47fbe7ed387c7d0160305faf098952fe0366c033d90b181057` |
| Assessment de fechamento S3 | `3d8d4d1aba216f03d3384ade951a9a46015411c6d4edb53e075e7f29813c5dd9` |
| relatório de fechamento S3 | `50ea883afe193cbf6cf6459123482582ea48a39ca146239f4a1f58be5ec13cde` |

Os nove insumos congelados pelo Assessment 0027 também foram respeitados. Nenhum hash
consultado divergiu da identidade registrada neste parecer.

## Matriz de autoridade e promessa

| Seam | Promessa obrigatória demonstrável | Exploração opcional explícita | Suficiência normativa | Classificação B2 |
|---|---|---|---|---|
| S1 — extractor/escritor de snapshot | `snapshot` Rust por query declarada, confinamento de path, cardinalidade/ausência, determinismo, formato v1 e escrita atômica | outras linguagens; novos formatos e relações | suficiente para extração estreita; insuficiente para contrato integral do writer | `SPEC-GAP` parcial |
| S2 — refinamento Git/subprocesso | prompt e adenda aprovam `refine-revisions`, OID imutável, Git local sem shell/rede/mutação, budgets e falha inconclusiva | `gix`, wrapper de comando, gravação diagnóstica de snapshots e SMT | contraditória quanto à vigência pública e ao envelope aprovado | `SPEC-GAP` bloqueante |
| S3 — manifesto/recibo/selo | manifesto v1, três categorias de oráculo, hashes/recibos, score, exits e publicação atômica do selo | sandbox atestável, assinatura, serviço remoto, orquestração e certificado posterior | forte e com fechamento específico | `PASS` como promessa fechada |
| S4 — pipeline principal | parse fail-closed, Map-Reduce, agregação, ordenação determinística, regras globais, apresentação e exit do lint | prova de todo ambiente/gramática e novos modos não publicados | suficiente somente quando decomposta por comando e saída observável | `PASS` parcial |
| S5 — parsers concretos | suporte público a nove linguagens; contrato comum de parser e extrações consumidas por V1–V6/V9–V13 | novas linguagens e fatos não consumidos/não prometidos | desigual: detalhada para Rust/TS/Python, média/baixa para C/C++/Zig e ausente como prompt concreto para Go/Java/Elixir | `SPEC-GAP` parcial |
| S6 — preflight/precedência CLI | flags documentadas, combinações inválidas, fatalidade V0/V8/V10, `fail-on`, formatos e exits dos subcomandos publicados | novas flags, formatos ou política não publicada | suficiente para casos estreitos; contraditória/incompleta como política global | `SPEC-GAP` parcial |

## S1 — extractor e escritor de snapshot

É promessa, não exploração: o comando `snapshot` aparece na documentação pública e o
L0 define Rust, `[[observable]]`, `one|many`, `unknown|absent`, query/path inválidos,
normalização, ordenação, formato v1, ausência de timestamp e escrita atômica. O loader
explícito tem schema, duplicatas, limites e classes de erro saneados, mas isso não fecha
automaticamente o extractor nem o writer.

Permanece `SPEC-GAP` no contrato integral de escrita: política de destino existente,
criação de diretórios, permissões/modo, durabilidade do diretório e recuperação de
temporário não são decididas. Também não está integralmente decidido, para o bloco
`[[observable]]` do extractor, o tratamento de chaves repetidas, tabelas/campos extras,
limites próprios de query/capturas e identidade entre observáveis duplicados. Há
autoridade suficiente para lotes estreitos de schema/extrator e publicação atômica; não
para um gate único da seam S1 inteira.

L3 possui a extração/persistência; L1 decide cardinalidade e ausência; L2 apresenta;
L4 compõe. Outras linguagens e novos formatos são opcionais até requisito explícito.

## S2 — refinamento Git e subprocesso

O corpo atual do prompt e a adenda do ADR descrevem B2 como aprovado e materializado,
com contrato detalhado para OIDs, framing, efeitos, symlink/submódulo, budgets e fallback.
Porém o cabeçalho e o gate anterior do ADR ainda dizem que Git não está autorizado, e o
`README.md` afirma que `refine` não lê Git nem executa comandos sem documentar
`refine-revisions`. Portanto a autoridade é internamente contraditória quanto à vigência
e à promessa pública. Isso é `SPEC-GAP` bloqueante antes de gate funcional.

Não são obrigatórios: backend `gix`, wrapper arbitrário, SMT, fetch, checkout, LFS,
submódulos ou persistência diagnóstica futura. Se a vigência de `refine-revisions` for
confirmada, o comportamento Git local delimitado torna-se obrigatório; se revogada, deve
ser removido das autoridades vigentes e tratado fora da campanha até novo requisito.

## S3 — manifesto, recibo e selo

Prompt e ADR-0020 são consistentes: manifesto fechado, hashes causais, produtores
nominais, oráculos `positive|negative|unknown`, recibos, `mutation_score = 1.0`, exits e
publicação atômica são promessas. O fechamento específico já confrontou entidade,
infraestrutura e wiring; B2 não encontrou causa normativa para reabertura.

Sandbox atestável, assinatura criptográfica, identidade remota, orquestração automática,
política de conflito e certificado pós-implementação estão explicitamente adiados. São
`ACCEPTED-RESIDUAL`, não bloqueadores, e só reabrem por requisito, incidente, mudança de
contrato/consumidor ou evidência de gate insuficiente. Durabilidade de diretório e
preservação explícita de modo permanecem resíduos operacionais já nomeados, não uma
reabertura genérica de S3.

## S4 — pipeline principal

`linter-core.md` promete o caminho público do lint: separação L1–L4, seleção pura do
parser, falha de leitura/parse projetada em violação, Map-Reduce, regras locais e globais,
ordenação determinística antes do formatter e política de exit. Logo a cadeia
entrada→IR→decisão→diagnóstico/exit é obrigatória.

A autoridade não sustenta auditar “o pipeline inteiro” como um único gate. Os comandos
`lint`, `fix-hashes`, `update-snapshot`, `refine`, `snapshot`, `refine-revisions` (se
vigente) e `seal-refinement` possuem efeitos e exits distintos. O gate deve ser recortado
por comando e caso observável, mantendo L1 como decisão, L2 como apresentação, L3 como
efeito e L4 como composição. Novos comandos e cobertura universal de ambientes são
opcionais.

## S5 — parsers concretos

O produto promete no `USAGE.md` suporte a Rust, TypeScript, Python, C, C++, Go, Zig,
Java e Elixir. O contrato comum exige seleção total dos nove slots, duas fases, FQN antes
de L1, erro tipado e extrações consumidas pelas regras. Isso torna obrigatória alguma
cadeia confrontada por linguagem oficialmente suportada e por característica publicada;
não torna obrigatório enumerar toda gramática.

Há três níveis de autoridade:

1. Rust, TypeScript e Python têm contratos concretos extensos;
2. C, C++ e Zig têm prompts concretos mais rasos que o template comum;
3. Go, Java e Elixir são promessas públicas e slots do contrato, mas não possuem prompt
   concreto em `00_nucleo/prompts/parsers/`.

Os níveis 2 e 3 são `SPEC-GAP` para gates integrais: faltam decisões uniformes sobre
gramática, resolução, fatos não suportados, duplicatas, localização, erro e limites. A
projeção numérica Rust fechada por P0097 é somente `CLOSED` naquele recorte. Citações,
demais campos de `SourceConstant`, `DecisionExpr`/V16, outros fatos Rust e todos os outros
parsers não herdam esse fechamento. Novas linguagens e fatos sem consumidor/promessa são
explorações opcionais.

## S6 — preflight e precedência CLI ampliada

São promessas obrigatórias: opções publicadas, defaults, rejeição de `--dry-run` isolado
e de reparadores simultâneos, fatalidade incondicional de V0/V8/V10, interação de
`--checks` com output, `--fail-on`, `--quiet`, formatos e exits 0/1/2 de refinamento.

Não existe, porém, uma tabela normativa única que ordene parse/preflight, carregamento de
configuração, reparadores, análise, formatação e cálculo de exit para todos os comandos.
Além disso, `USAGE.md` chama combinações inválidas de “exit 1 imediato”, enquanto os
contratos de refinamento reservam exit 2 para erro de entrada/configuração; a aplicação
da regra aos subcomandos não está decidida. A omissão pública de `refine-revisions` também
interage com S2. Assim, casos estreitos são delimitáveis, mas a precedência global é
`SPEC-GAP` e deve ser saneada antes de gate abrangente.

## Contradições e SPEC-GAPs consolidados

1. **SG-27-B2-01 — writer S1:** overwrite, diretórios, modo, durabilidade e recuperação
   não têm comportamento normativo fechado.
2. **SG-27-B2-02 — schema do extractor S1:** duplicatas, campos extras e budgets de
   `[[observable]]` não estão integralmente definidos.
3. **SG-27-B2-03 — vigência Git S2:** cabeçalho/gate antigo do ADR e documentação pública
   contradizem prompt e adenda aprovados para B2.
4. **SG-27-B2-04 — parsers S5:** C/C++/Zig ficam abaixo do template; Go/Java/Elixir são
   suporte publicado sem prompt concreto correspondente.
5. **SG-27-B2-05 — CLI S6:** falta matriz global de precedência e classe de exit por
   comando; “exit 1 imediato” conflita ou não esclarece erro de entrada exit 2.
6. **SG-27-B2-06 — fronteira pública:** `README.md` documenta `refine` como sem Git, mas
   não distingue nem publica claramente o comando Git que o L0 chama vigente.

## Resultado para reconciliação C

- S3 possui autoridade e fechamento suficientes; só resíduos explicitamente adiados
  permanecem opcionais.
- S4 contém promessa obrigatória, mas deve ser decomposta por comando/caso; não há
  autoridade para promovê-la inteira a um lote único.
- S1, S2, S5 e S6 contêm promessas obrigatórias, porém conservam os `SPEC-GAP` acima.
- P0097 fecha somente sua projeção numérica Rust; não reduz por associação as demais
  promessas de S5.
- Nenhuma recomendação ou escolha de P0099 foi feita.

**Veredito B2:** `PASS WITH SPEC-GAPS`.
