# ⚖️ ADR-0002: Code-to-Prompt Feedback Direction

**Status**: `ACEITO`
**Data**: 2025-03-13

---

## Contexto

A Arquitetura Cristalina estabelece que o prompt em L0 é a origem
causal do código. O linter atual (V1–V5) verifica apenas a direção
prompt→código: detecta quando o código diverge de seu prompt de origem.

A direção inversa — código→prompt — não tem mecanismo de detecção.
Isso cria três vulnerabilidades estruturais:

1. **Modificações esquecidas**: edições pequenas (imports, assinaturas,
   correções) acumulam divergência silenciosa entre código e prompt até
   o prompt perder sua função de origem causal.

2. **Múltiplos agentes**: dois agentes trabalhando em paralelo podem
   criar contradições entre arquivos que individualmente passam no
   linter mas coletivamente representam um estado impossível.

3. **Assimetria de proteção**: o prompt protege o código via V5, mas
   o código não protege o prompt. A fonte de verdade declarada (L0)
   pode divergir do sistema real sem nenhum sinal visível.

A raiz do problema é que V5 compara hashes — detecta quando o prompt
mudou depois do código. Mas não existe verificação para o caso inverso:
código que mudou depois do prompt.

---

## Decisão

Introduzir **V6 — PromptStale**: verificação que detecta quando um
arquivo de implementação foi modificado sem revisão correspondente
no prompt que o originou.

A detecção opera via **diff semântico de AST** usando tree-sitter —
não diff de texto, não hash do arquivo de código. Mudanças cosméticas
(formatação, comentários, whitespace) não disparam V6. Mudanças
estruturais disparam.

**Threshold de disparo — mudanças estruturais detectáveis:**

| Tipo de mudança | Dispara V6? |
|-----------------|-------------|
| Interface pública alterada (`pub fn`, `pub struct`, `pub trait`) | ✅ sim |
| Import adicionado ou removido | ✅ sim |
| Assinatura de função alterada | ✅ sim |
| Novo símbolo proibido em L1 | ✅ sim (já coberto por V4) |
| Reformatação, whitespace, comentário | ❌ não |
| Renomeação interna sem mudança de interface | ❌ não |

**Fonte de verdade quando prompt e código divergem:**

O prompt é a origem causal declarada. Código divergente é sempre
um estado a ser resolvido — não necessariamente um erro. V6 é
`warning` por padrão, não `error`, porque a resolução correta
depende de julgamento humano:

- Se o código evoluiu corretamente → o prompt deve ser atualizado
- Se o código mudou incorretamente → a mudança deve ser revertida
- Em ambos os casos → V6 permanece até a decisão ser registrada

**Mecanismo de resolução:**

V6 é silenciado quando o arquivo de prompt tem sua data de
`@updated` posterior à data de modificação do arquivo de código,
ou quando o hash do prompt muda após a modificação do código.
Isso força uma decisão explícita — não permite ignorar passivamente.

**Arquitetura de entrega:**

V6 requer o linter operando como servidor MCP (Arquitetura 2) para
observar mudanças em tempo real em ambos os diretórios. O binário
CLI retém V6 como verificação estática — compara timestamps de
modificação de arquivo versus data `@updated` do prompt.

---

## Prompts Afetados

| Prompt | Natureza da mudança |
|--------|---------------------|
| `linter-core.md` | Adicionar V6 às verificações, flags `--watch` e MCP server |
| `violation-types.md` | Adicionar `CodeDelta` e `StructuralChange` à IR |
| `rs-parser.md` | Extrair interface pública do AST para popular `CodeDelta` |
| `sarif-formatter.md` | Adicionar V6 à tabela de regras SARIF |

Novo prompt a criar:
- `rules/prompt-stale.md` ← V6

---

## Consequências

**Positivas:**
- Protege a integridade bidirecional entre prompt e código
- Torna o custo de modificações diretas imediato e visível
- Permite coordenação entre múltiplos agentes via árbitro de consistência
- Diff semântico via AST elimina ruído de mudanças cosméticas

**Negativas:**
- Requer snapshot do estado público do arquivo no momento da geração
  para comparação futura — nova responsabilidade para L3
- V6 estático (CLI) depende de timestamps de filesystem — frágil em
  alguns ambientes CI e sistemas de arquivos remotos
- V6 dinâmico (MCP) requer nova camada de infraestrutura não presente
  na v1

**Neutras:**
- V6 como `warning` significa que não bloqueia CI por padrão —
  configurável para `error` via `crystalline.toml`
- A decisão de reverter vs evoluir o prompt permanece humana —
  V6 detecta, não decide

---

## Alternativas Consideradas

| Alternativa | Prós | Contras |
|-------------|------|---------|
| Hash do arquivo de código no prompt | Simples, sem AST | Qualquer mudança cosmética dispara, muito ruidoso |
| Git hooks comparando diff | Zero infra nova | Frágil fora de git, não funciona com múltiplos agentes simultâneos |
| Ignorar direção inversa | Zero complexidade | Deixa a vulnerabilidade estrutural sem solução |

---

## Referências

- ADR-0001: tree-sitter intermediate representation (base para diff semântico)
- Manifesto: seção "Limitações Declaradas" — divergência prompt/código
- `00_nucleo/prompts/rules/prompt-drift.md` — V5, direção oposta
