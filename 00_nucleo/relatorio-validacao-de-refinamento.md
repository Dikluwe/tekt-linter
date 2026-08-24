# Relatório de investigação — validação de refinamento

**Data:** 2026-08-23  
**Resultado:** hipótese confirmada para comparação finita de fatos; produto não implementado  
**Gate:** ADR-0019 e prompt `refinement-validator.md` aguardam aprovação humana

## Trabalho executado

- leitura dos contratos L0 de núcleo, IR, parser, regras V23–V25, SARIF e oráculo
  diferencial, além dos ADRs 0001, 0002, 0016–0018;
- inventário de `ParsedFile`, `SemanticObservation`, V6/`InterfaceDelta`, configuração,
  snapshots, pipeline e SARIF;
- estudo das fontes primárias do Alive2;
- seleção de três pares reais antes/depois no `typst-crystalline`;
- experimento puro e descartável em `lab/refinement_probe.rs`;
- execução de nove testes;
- redação do ADR-0019 e do prompt proposto.

## Resultado do experimento

```text
9 passed; 0 failed
```

Foram demonstrados:

- preservação de campo obrigatório;
- testemunha de apagamento;
- normalização autorizada e não autorizada;
- proibição de inventar nova autoridade;
- perda de estado contextual;
- direcionalidade;
- `Unknown` para macro opaca;
- ausência conhecida de fato proibido.

## Hipótese confirmada

Não é necessário SMT para o primeiro incremento. Um comparador puro sobre fatos finitos
consegue representar os três oráculos e produzir testemunha útil. A principal mudança
arquitetural não é um algoritmo complexo: é tornar insuficiência de evidência um estado
de domínio explícito.

## Descobertas de arquitetura

1. `SemanticObservation` atual já chega a conclusões negativas; para refinamento será
   melhor separar `Observable/Evidence` neutro de `Violation`.
2. V6 já contém um comparador antes/depois, mas específico de interface e com ausência
   de snapshot tratada como silêncio. Ele é precedente técnico, não núcleo reutilizável
   direto.
3. ADR-0018 afirma “não analisável”, porém o produto ainda não transporta esse estado.
   `Unknown` fecha essa discrepância.
4. A primeira interface segura é snapshot contra snapshot. Git e wrapper introduzem
   I/O, autoridade e recuperação desnecessários para validar o núcleo.
5. Comparar o repositório inteiro é cedo demais. A unidade inicial deve ser um conjunto
   nomeado de observáveis ligado a um contrato.

## Falsos positivos e negativos

O experimento não usa inferência: só avalia relações declaradas, portanto não produziu
falso positivo nas nove fixtures. Isso não mede a precisão do futuro extrator AST.

Falsos negativos conhecidos do modelo experimental:

- identidade errada fornecida pelo extrator;
- relações ausentes no contrato;
- normalização semanticamente inválida listada como aceita;
- efeitos não representados pelos fatos;
- aliases, macros e chamadas interprocedurais.

Esses casos devem ser `Unknown` quando detectáveis. Contrato incompleto continua sendo
um risco humano e não pode ser eliminado pelo comparador.

## Limites

- Rust é o primeiro candidato de extração, embora o comparador seja neutro;
- nenhuma equivalência funcional foi demonstrada;
- testemunha é contraexemplo ao contrato de fatos, não ao programa executável;
- não há solver, execução, memória ou fluxo interprocedural;
- não há ainda formato canônico de snapshot nem política de exit code para `Unknown`;
- a precedência entre várias relações precisa de fixture RED adicional: `Violated`
  deve vencer `Unknown`, sem apagar os detalhes inconclusivos.

## Continuação recomendada

Após aprovação humana:

1. resselar o prompt aprovado;
2. congelar os três oráculos em fixtures locais mínimas;
3. escrever fixtures RED para agregação, determinismo e formato versionado;
4. materializar somente o comparador L1 e adapters de snapshot;
5. adiar Git, wrapper e SMT.

Nenhuma dessas ações está autorizada por este relatório.

## Estado do auto-lint no gate

O protótipo possui linhagem experimental para o prompt proposto e não produz V1. Essa
referência do laboratório também basta para V7 não considerar o prompt órfão, embora
continue não existindo materialização L1–L4. O auto-lint final passou; o estado
`PROPOSTO` do prompt e o gate deste relatório são, portanto, as travas que impedem sua
promoção indevida.
