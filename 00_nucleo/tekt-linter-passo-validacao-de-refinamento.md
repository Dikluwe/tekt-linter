# Passo operacional — validação de refinamento de contratos

> **Natureza:** comando operacional temporário para o LLM; não é regra da arquitetura  
> **Estado:** Etapa A materializada e validada em branch dedicado
> **Identidade:** descritiva e não numerada; não cria uma nova sequência de passos  
> **Destino:** absorver decisões em ADR/prompts e então arquivar ou eliminar  
> **Índice:** [`00_nucleo/README.md`](README.md)

**Repositório de implementação:** `tekt-linter`.  
**Referência conceitual:** Alive2, especialmente validação de tradução, refinamento
direcional, contraexemplos e resultado inconclusivo.  
**Base local:** V23 `ContextErasure`, V24 `SemanticFieldLoss` e V25
`DecisionOwnership`.

## Objetivo

Investigar e, somente após decisão humana e nucleação causal, materializar uma
capacidade genérica que compare fatos semânticos observáveis antes e depois de uma
transformação e responda:

```text
Preserved
Violated(witness)
Unknown(reason)
```

O linter deve verificar a transformação concreta, não tentar provar correto o programa
que a produziu. O primeiro incremento não deve incorporar LLVM, Alive2, SMT nem alegar
equivalência funcional geral.

Casos de uso pretendidos:

- refatorações e mudanças entre commits;
- geração ou reescrita de código por LLM;
- projeção de entidade para DTO, chave ou identidade;
- serialização, desserialização e normalização;
- migração entre camadas ou representações;
- execução de geradores e migradores sob um wrapper de validação.

## Hipótese

As regras V23–V25 reconhecem padrões sintáticos perigosos quando um contrato explícito
orienta o parser. Uma capacidade de refinamento pode generalizar parte desse trabalho:

1. extrair fatos observáveis de um estado-fonte;
2. extrair os mesmos fatos do estado-alvo;
3. aplicar uma relação direcional declarada;
4. produzir uma testemunha mínima quando o alvo não preserva o contrato;
5. declarar `Unknown`, e não sucesso, quando faltar evidência.

Essa capacidade não substitui automaticamente V23–V25. A investigação deve demonstrar
quais diagnósticos continuam sendo lint local, quais podem ser derivados de refinamento
e quais exigem mecanismos diferentes.

## Princípios importados, não copiados, do Alive2

### Validar a instância da transformação

Verificar `before → after`, em vez de provar correto todo gerador, compilador ou agente.

### Refinamento é direcional

O alvo deve preservar os observáveis exigidos pela fonte. Igualdade bidirecional só
pode ser exigida quando o contrato a declarar; normalizações e estados mais definidos
podem ser válidos.

### Contraexemplo é parte do veredito

`Violated` deve carregar uma testemunha reproduzível: contrato, observável, valor antes,
valor depois, origem das evidências e transformação analisada.

### Limites são explícitos

Análise limitada, parser sem suporte, macro opaca, chamada externa, fluxo
interprocedural ou orçamento esgotado devem resultar em `Unknown(reason)`. Proibido
converter ausência de prova em `Preserved`.

### Integração ocorre no caminho real

Além de fixtures, avaliar comparação entre revisões e um wrapper que capture estado
antes/depois de um comando. A integração não pode executar comandos arbitrários sem
autorização explícita do humano.

## Modelo conceitual mínimo a investigar

```text
Source artifact ──> source observations ─┐
                                        ├──> refinement relation
Target artifact ──> target observations ─┘          │
                                                    ▼
                              Preserved | Violated(witness) | Unknown(reason)
```

Um contrato experimental pode ter forma semelhante a:

```toml
[[refinement]]
id = "font-identity"
language = "rust"
source = "TextStyle"
target = "FontIdentity"

preserve = ["family", "variant", "variations"]
may_normalize = ["weight"]
must_not_invent = ["math"]
```

Essa sintaxe é apenas hipótese. Ela não é contrato público até ser testada, decidida em
ADR e absorvida por prompts L0.

## Fase 0 — Pesquisa e delimitação

1. Ler integralmente os prompts L0 de parser, configuração, regras, CLI e entidades
   que uma eventual solução tocaria.
2. Ler os ADRs de V23–V25 e identificar os limites já assumidos.
3. Estudar nas fontes primárias do Alive2:
   - validação de tradução;
   - refinamento direcional;
   - geração de contraexemplo;
   - tratamento explícito de limitações;
   - integração como wrapper/plugin;
   - cache e reprodutibilidade de consultas.
4. Inventariar mecanismos já existentes no linter que possam ser reutilizados:
   `ParsedFile`, observações semânticas, configuração, fixtures diferenciais, SARIF e
   seleção de checks.
5. Selecionar no máximo três transformações reais e pequenas como oráculos iniciais.
6. Registrar SHA, estado do working tree, comando e hora de toda medição externa.

### Perguntas que a pesquisa deve responder

- Qual é a unidade comparada: arquivo, função, entidade, contrato ou conjunto de fatos?
- Como fatos dos dois lados recebem identidade estável sem depender só de nomes?
- O que significa “preservar” para campo ausente, opcional, normalizado ou inventado?
- Quais estados formam o domínio: desconhecido, contextual, resolvido, normalizado,
  apagado ou outros?
- Como distinguir falha real de capacidade de análise insuficiente?
- É possível produzir testemunhas úteis sem SMT?
- O modo entre commits pode operar sobre snapshots explícitos sem alterar o worktree?
- O wrapper pertence ao produto inicial ou deve ficar para uma segunda etapa?

## Fase 1 — Experimento no laboratório

Antes de alterar L1–L4, construir no `lab/` um experimento descartável e não importável
pelas camadas principais.

O experimento deve:

1. consumir fatos normalizados de `before` e `after`, inicialmente fornecidos por
   fixtures, sem implementar ainda outro parser;
2. aplicar contratos pequenos de `preserve`, `may_normalize` e `must_not_invent`;
3. produzir os três resultados e uma testemunha estruturada;
4. provar que inverter fonte e alvo pode mudar o resultado;
5. provar que evidência ausente resulta em `Unknown`;
6. medir clareza do diagnóstico e taxa de falsos positivos nos oráculos escolhidos.

Não promover código do laboratório. Se a hipótese sobreviver, reescrever a solução a
partir do ADR e dos prompts aprovados.

## Fase 2 — Decisão arquitetural e parada obrigatória

Com os resultados do laboratório:

1. escrever um ADR que decida:
   - escopo e não objetivos;
   - unidade de comparação;
   - relação de refinamento;
   - modelo de evidência e testemunha;
   - semântica de `Unknown`;
   - posição de cada entidade por camada;
   - interface de configuração e compatibilidade;
   - relação com V23–V25;
   - política de orçamento, cache e reprodutibilidade;
2. criar ou atualizar os prompts L0 correspondentes;
3. definir se o recurso será uma regra pública, um modo de operação ou ambos;
4. definir seu nome público sem inferir automaticamente um próximo número `V*`;
5. apresentar ADR, prompts, resultados e alternativas ao humano;
6. **PARAR antes de alterar L1–L4.**

O laboratório e este passo não legitimam código de produto.

## Fase 3 — Fixtures RED

Somente após aprovação humana e resselo dos L0, escrever testes que falhem antes da
implementação.

### Preservação

- todos os campos obrigatórios permanecem iguais;
- campo autorizado é normalizado;
- informação opcional ausente continua ausente;
- alvo fica mais específico somente quando o contrato permite.

### Violação

- campo obrigatório é apagado ou substituído por neutro;
- alvo inventa decisão proibida;
- valor contextual é tratado como resolvido;
- normalização não autorizada muda um observável;
- comparação invertida demonstra a direcionalidade.

### Inconclusivo

- parser não suporta uma construção;
- identidade entre observáveis é ambígua;
- macro ou chamada opaca impede a prova;
- contrato é parcial;
- orçamento declarado se esgota.

Cada violação deve verificar a testemunha completa. Cada `Unknown` deve verificar uma
razão estável e acionável. A seleção isolada não pode ativar V23–V25 implicitamente.

## Fase 4 — Materialização mínima

Se autorizada:

1. criar em L1 tipos puros para observáveis, contratos, relação de refinamento,
   testemunhas e resultado ternário;
2. manter em L1 o comparador determinístico e independente de filesystem, relógio,
   processo, Git, parser e formato de saída;
3. adaptar parsers em L3 apenas para produzir fatos comprováveis;
4. manter leitura de configuração e snapshots externos em L3;
5. colocar tradução de argumentos e apresentação em L2;
6. limitar L4 à composição;
7. adicionar saída textual e SARIF sem reduzir `Unknown` a sucesso;
8. garantir ordenação determinística e identificadores reproduzíveis;
9. implementar cache somente depois de demonstrar necessidade e definir chave que
   inclua contrato, fatos, versão do analisador e limites;
10. não implementar SMT na primeira versão.

## Fase 5 — Integração progressiva

Avaliar separadamente, sem assumir que todos pertencem à primeira entrega:

### Comparação de snapshots

```bash
crystalline-lint refine --before <snapshot-a> --after <snapshot-b>
```

É o modo preferido inicial por ser determinístico e não executar processos externos.

### Comparação entre revisões

```bash
crystalline-lint refine --before-ref <sha-a> --after-ref <sha-b>
```

Deve usar leitura não destrutiva. Proibido trocar o checkout ou alterar o worktree para
obter as revisões.

### Wrapper de comando

```bash
crystalline-lint refine-run -- <comando>
```

Só implementar mediante decisão explícita posterior. Deve exigir consentimento claro,
preservar o estado anterior, reportar efeitos e não ampliar autoridade do comando.

## Guardas contra falsa formalidade

É proibido:

- chamar comparação sintática de equivalência semântica;
- declarar `Preserved` porque nenhum padrão conhecido foi encontrado;
- inferir tipos, owners ou identidade apenas por semelhança de nomes;
- esconder limites em logs de debug;
- transformar timeout ou construção não suportada em aprovação;
- copiar as semânticas de `undef`, `poison` ou memória do LLVM para outro domínio;
- adicionar Z3 ou outra dependência pesada antes de um oráculo exigir isso;
- fundir `Unknown` com warning genérico sem identidade de causa;
- executar transformações externas durante lint comum;
- tornar este passo ou seu nome uma dependência do produto.

## Critérios de aceitação

1. ADR e L0 são aprovados antes da materialização L1–L4.
2. A relação é direcional e documenta quais observáveis preserva, normaliza ou proíbe
   inventar.
3. `Preserved`, `Violated` e `Unknown` são estados distintos no domínio e nas saídas.
4. Toda violação contém testemunha reproduzível; todo inconclusivo contém razão.
5. A primeira versão funciona sem SMT e não promete equivalência funcional geral.
6. Fixtures demonstram positivos, negativos, direcionalidade e insuficiência de prova.
7. O núcleo permanece puro e analisadores/configuração não vazam para L1.
8. Resultados são determinísticos e independem da ordem de descoberta dos arquivos.
9. A integração com V23–V25 é explícita e não duplica diagnósticos silenciosamente.
10. Testes, auto-lint, hashes L0, CLI e SARIF passam conforme os prompts aprovados.

## Relatório final exigido

Separar claramente:

- hipótese confirmada ou refutada;
- contratos e observáveis realmente suportados;
- violações com testemunhas;
- casos inconclusivos e motivo;
- falsos positivos e falsos negativos conhecidos;
- limites de linguagem, escopo e orçamento;
- relação final com V23–V25;
- itens deliberadamente adiados, especialmente wrapper, SMT e análise
  interprocedural.

## Referências primárias

- AliveToolkit, [`alive2`](https://github.com/AliveToolkit/alive2): arquitetura,
  ferramentas, integração, limites e execução de validação de tradução.
- Nuno P. Lopes et al., [*Alive2: Bounded Translation Validation for
  LLVM*](https://web.ist.utl.pt/nuno.lopes/pubs.php?id=alive2-pldi21), PLDI 2021.

Essas referências orientam princípios. Não são contratos causais do Tekt Linter e não
autorizam transposição direta da semântica LLVM.

## Registro da investigação

A Fase 0 e o experimento da Fase 1 foram executados em 2026-08-23. A hipótese de
comparação finita foi confirmada por nove testes no laboratório. O resultado completo
está em [`relatorio-validacao-de-refinamento.md`](relatorio-validacao-de-refinamento.md).

Foram propostos:

- [`ADR-0019`](adr/0019-validacao-direcional-de-refinamento.md);
- [`refinement-validator.md`](prompts/refinement-validator.md).

**Gate aprovado:** o humano autorizou a Etapa A em branch dedicado em 2026-08-23.
Continuam fora de escopo Git, wrapper, SMT e análise interprocedural.
