# Passo 0067 (tekt-linter) — V22: corrigir heurística de citação (detectar `file:line`, não palavra-chave)

**Repositório**: `tekt-linter`.
**Precede este passo**: Passo 0066 (V21/V22). Achado: V22 devolve `0.0%` em todos os 4
módulos do `typst-crystalline` (5713+68+3599+438 literais) porque `rs_parser.rs:1700-1725`
só reconhece comentários com prefixo literal `// ref:`/`// spec:`/`// rationale:`. O
código real usa outra convenção, já consistente há dezenas de passos (`// P813 — ... —
lab/typst-original/.../container.rs:342`, `// vanilla resolve.rs:1173`), nunca reconhecida
pelo parser.
**Pré-condição**: `git status` limpo no `tekt-linter`.

---

## Princípio: o sinal certo é `file:line`, não a palavra-chave

A V22 não devia impor um vocabulário novo a um código que já tinha um — devia reconhecer
o que já lá está. O que faz um comentário ser "citação" não é começar por uma palavra
específica, é **conter uma referência real a `caminho:linha`** de outro ficheiro.

## Fase A — Ler o parser actual, confirmar o alcance do bug

`03_infra/rs_parser.rs:1700-1725` — confirmar que a lógica é mesmo `strip_prefix`
literal nas três palavras, sem fallback nenhum.

## Fase B — Nova heurística, permissiva ao vocabulário, estrita ao sinal

Um comentário conta como citação se contiver, em qualquer ponto do texto (não só no
início):

1. **Um padrão `file:line`** — regex tipo `[\w./\-]+\.(rs|md|typ):\d+` (cobre
   `container.rs:342`, `resolve.rs:1173`, `math/frac.rs:9`, etc.).
2. **OU** os prefixos já existentes `ref:`/`spec:`/`rationale:` (manter, para prompts/
   código futuro que os use deliberadamente).
3. **OU** a palavra `vanilla` seguida, na mesma linha ou até 2 linhas depois, de um
   padrão `file:line` (cobre `// vanilla resolve.rs:1173` sem `.rs` explícito antes do
   nome do crate, se for esse o formato real usado nalguns sítios — confirmar variantes
   reais por amostra do código antes de fechar a regex).

**Não exigir que a citação esteja na mesma linha do literal** — os exemplos reais
(`equation.rs`) têm o comentário 1-3 linhas acima do uso. Manter a janela de procura já
existente (se o parser já procura em linhas anteriores) ou alargar se não procurar.

## Fase C — Testar contra os casos já confirmados manualmente

Antes de aceitar a regex nova, confirmar que reconhece, no mínimo, estes casos já
verificados nesta frente:
- `equation.rs:117-119` (`P813`, `container.rs:342`)
- Qualquer citação de `math/layout/_comum.md` §P912 (formato `resolve.rs:1173`)
- Os L0s corrigidos no P1042 (`matrix.md`/`cases.md`, citação de `file:line` real)

Se a regex nova não apanhar estes 3, ainda não está pronta — não é para "melhorar o
número", é para reconhecer citação real.

## Fase D — Recorrer V22 no `typst-crystalline`

```
crystalline-lint --checks v22 .
```
Esperar percentagem muito mais alta que 0% — não presumir qual, medir. Se ainda sair
baixo, isso já seria informação real (não mascarada por bug de parser), não mais um falso
alarme.

---

## Resultado esperado

V22 reconhece a convenção de citação real do código (`P<NNN> — ... — file:line`, `vanilla
file:line`), não só um vocabulário novo que o código nunca usou. Métrica de proveniência
volta a ser um sinal real, utilizável para vigilância contínua per a Parte 2/3 do Passo
0066.
