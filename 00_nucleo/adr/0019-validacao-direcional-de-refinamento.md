# ADR-0019 — Validação direcional de refinamento sobre fatos observáveis

**Status:** PROPOSTO — aguarda aprovação humana  
**Data:** 2026-08-23  
**Origem operacional:** `tekt-linter-passo-validacao-de-refinamento.md`  
**Escopo desta revisão:** decisão arquitetural; não autoriza materialização L1–L4

## Contexto

V23–V25 reconhecem perda de contexto, perda de campo e reentrada de autoridade em uma
única versão do código, sob contratos explícitos. ADR-0018 limita honestamente a
análise, mas o produto atual representa um caso não analisável pelo mesmo resultado
externo de um caso preservado: ausência de diagnóstico.

O Alive2 demonstra uma estratégia complementar: validar cada transformação concreta,
comparando fonte e alvo por uma relação direcional de refinamento, em vez de provar o
transformador inteiro. Sua utilidade depende de IR comum, semântica explícita,
contraexemplo e limites declarados. A semântica LLVM, execução simbólica e SMT não são
transferíveis diretamente ao Tekt Linter.

Um experimento descartável em `lab/refinement_probe.rs` comparou fatos normalizados sem
parser, filesystem ou solver. Nove casos cobriram preservação, perda, normalização
permitida/proibida, invenção de autoridade, direcionalidade e evidência desconhecida.
Todos passaram.

## Medição e oráculos

Investigação feita em `2026-08-23T22:50:44-03:00`.

- `tekt-linter`: HEAD `4c9583d8860236788ceaeb05d795d58453d0fc69`, working tree
  não commitado, 63 arquivos rastreados alterados, 2.813 inserções e 704 remoções,
  além de arquivos não rastreados.
- `typst-crystalline` atual: HEAD
  `ffd527c85dd7d547413d33cbc2d27a80e32a3f8c`, working tree rastreado limpo.
- fonte histórica defeituosa:
  `781b207b4a5de9c2bfbe5819918a193d1d9293e5`.

Os dois SHAs do oráculo fornecem transformações reais:

| Contrato | Antes | Depois | Relação esperada |
|---|---|---|---|
| raio contextual | `resolve_pt(0.0)`/`.abs.0` nos exportadores | resolução centralizada preserva `Length` até receber contexto | correção refina o estado defeituoso; apagar contexto não refina o contrato original |
| identidade de fonte | `FontVariations::default()` em `resolve_font_combo` | `style.variations.clone().unwrap_or_default()` | campo obrigatório preservado |
| autoridade `ssty`/math | classificador duplicado e proxies `contains("math")` | owner importado de `font_metrics` e uso de `style.math` | alvo deixa de inventar owners/proxies |

Esses pares devem ser congelados em fixtures mínimas; não se deve depender de o
repositório externo continuar retendo os SHAs.

## Decisão proposta

### 1. Adotar uma capacidade, não presumir nova regra numerada

Criar uma capacidade chamada provisoriamente `refinement`, cujo produto principal é um
`RefinementVerdict`. A decisão sobre exposição como check `V*`, subcomando ou ambos
fica para um ADR de interface depois das fixtures RED. Não alinhar o recurso a números
de passos.

### 2. Comparar conjuntos normalizados de observáveis

A unidade mínima é um `ArtifactFacts`: identificador de artefato, versão da extração e
mapa ordenado de observáveis com identidade estável. A primeira versão recebe snapshots
já extraídos. Comparação direta de ASTs ou commits não pertence ao núcleo.

Relações iniciais:

- `preserve(source, target)`: o valor observável deve sobreviver;
- `may_normalize(source, target, accepted)`: igualdade ou forma alvo declarada;
- `must_not_invent(target)`: o alvo não pode introduzir o fato;
- `must_exist(target)`: reservado para provar necessidade empírica antes de incluir.

Não criar um lattice genérico de estados no primeiro incremento. Estados como
`contextual`, `resolved`, `normalized` e `erased` são valores do domínio do contrato;
a ordem entre eles deve ser declarada quando necessária, não universalizada por
conveniência.

### 3. Resultado ternário é invariante do domínio

```text
Preserved
Violated(Witness)
Unknown(UnknownReason)
```

`Unknown` é distinto de `Preserved` e deve sobreviver até CLI e SARIF. Razões iniciais
devem ser fechadas e mecanicamente identificáveis: observável ausente/ambíguo, parser
sem suporte, construção opaca, contrato parcial e orçamento esgotado. Texto livre pode
complementar a razão, nunca substituí-la.

### 4. Testemunha é estruturada e reproduzível

Uma testemunha contém contrato, relação, artefatos fonte/alvo, chaves, valores, origem
das evidências e versões dos extratores. A testemunha não afirma uma entrada executável
do programa: na primeira versão, é um contraexemplo ao contrato sobre fatos observados.

### 5. Separação por camadas

- **L1:** fatos normalizados, contrato compilado, veredito, testemunha e comparador puro.
- **L2:** argumentos, apresentação e política explícita de exit code.
- **L3:** leitura/escrita de snapshots, extração por linguagem e, futuramente, acesso
  não destrutivo a revisões.
- **L4:** composição dos dois lados e ordenação determinística.
- **Lab:** protótipos; nunca importado pelas camadas principais.

### 6. Entrega em duas etapas

**Etapa A:** comparar dois snapshots explícitos, canônicos e versionados. Nenhum comando
externo e nenhuma manipulação de Git.

**Etapa B, decisão posterior:** extrair snapshots de duas revisões ou envolver um
comando. Só entra após política de efeitos, consentimento e recuperação ser decidida.

### 7. Relação com V6 e V23–V25

- V6 permanece drift de interface pública contra snapshot causal; não é substituída.
- V23–V25 permanecem lint local de um único estado.
- `refinement` compara estados e pode reutilizar os mesmos fatos normalizados.
- Um mesmo achado não deve ser emitido duas vezes no mesmo modo. A política de
  deduplicação deve escolher o diagnóstico com evidência mais forte, preservando a
  proveniência do check suprimido.

### 8. Sem SMT no primeiro incremento

Os três oráculos são decidíveis por comparação finita de fatos declarados. Solver,
execução simbólica, memória e análise interprocedural ficam fora do escopo até existir
um caso real que não possa ser expresso pelo modelo finito.

## Consequências

**Positivas:** distingue silêncio comprovado de incapacidade; oferece testemunha
estruturada; reutiliza fatos de V23–V25; permite validar reescritas de LLM sem provar o
LLM; mantém L1 puro e multi-linguagem.

**Negativas:** exige dois artefatos e identidades estáveis; aumenta o contrato público;
`Unknown` requer representação adequada em text/SARIF e política de CI própria; o modo
snapshot adiciona versionamento de formato.

**Riscos:** mapeamento nominal pode fingir identidade; snapshots podem divergir do
extrator; `may_normalize` amplo pode esconder perda. Mitigações: identidade declarada,
versão do extrator na chave, normalizações enumeradas e fixtures diferenciais.

## Alternativas rejeitadas

- Estender V24 para comparar commits: mistura lint local com verificação binária.
- Tratar ausência de observação como aprovação: falso senso de prova.
- Retornar apenas violation/não-violation: perde a insuficiência de evidência.
- Copiar a semântica de refinamento do LLVM: domínio errado.
- Introduzir SMT agora: custo e superfície sem oráculo que o exija.
- Executar `git checkout` para obter o estado anterior: mutação destrutiva desnecessária.
- Implementar wrapper na primeira entrega: amplia autoridade e efeitos antes de haver
  núcleo estável.

## Gate

Este ADR e o prompt associado devem ser aprovados pelo humano. Até então:

- nenhuma entidade é criada em L1;
- nenhuma CLI/configuração/SARIF é alterada;
- nenhum código do laboratório é promovido;
- nenhum número `V*` é reservado.

## Referências

- AliveToolkit, [`alive2`](https://github.com/AliveToolkit/alive2).
- Nuno P. Lopes et al., [*Alive2: Bounded Translation Validation for
  LLVM*](https://web.ist.utl.pt/nuno.lopes/pubs.php?id=alive2-pldi21), PLDI 2021.
- ADR-0001, ADR-0002 e ADR-0018 deste repositório.
