# Triagem adversarial — inventário e registro de crates

## Convenção

- **RED:** duas entradas logicamente equivalentes por permutação produzem resposta ou
  diagnóstico diferente, contrariando o assessment congelado.
- **SPEC-GAP:** existe ambiguidade ou colisão real, mas os prompts históricos não
  congelam qual resultado canônico ou política de rejeição deve vigorar.

São propostas quatro propriedades/mutações de alto sinal.

## P1 — permutação de membros, nome duplicado e lookup canônico

Construir membros com sentinelas de camada distintas:

```text
M1 = { name: "same", dir: "a", layer: L1 }
M2 = { name: "same", dir: "b", layer: L3 }
M3 = { name: "unique", dir: "c", layer: L4 }
```

Para todas as permutações de `[M1, M2, M3]`, comparar
`member_layer("same")`, `member_layer("unique")` e lookup ausente. Repetir com uma
duplicata byte-idêntica de M1 e com duplicata que difere apenas em layer/dir/deps.

Mutação que a propriedade deve matar: lookup por `find`, “first wins”, “last wins”,
dedup apenas adjacente ou transformação de `Vec` em mapa com overwrite silencioso.

**RED exato:** permutar a mesma coleção muda qualquer resposta. O alvo usa o primeiro
membro de nome igual, logo M1/M2 tornam `member_layer("same")` dependente da ordem e
são candidato RED direto. Uma duplicata idêntica pode ser idempotente, mas duplicata
conflitante nunca pode escolher silenciosamente pela posição.

**SPEC-GAP:** o prompt exige lookup por nome, mas não define se nomes duplicados devem
ser rejeitados, se duplicatas idênticas são coalescidas, nem qual API reporta erro —
`from_members` não retorna `Result`. O assessment aceita rejeição ou resolução
canônica; escolher entre elas exige decisão de contrato.

## P2 — owner mais profundo, empate e invariância de representação

Gerar uma tabela com:

- ancestral raso `ws/crates` e profundo `ws/crates/a` — o profundo deve vencer;
- dois membros com o mesmo `dir` e metadados distintos — empate real;
- dirs lexicalmente diferentes com mesma contagem de componentes, ambos capazes de
  casar somente após variações de `.`/`..` ou forma absoluta/relativa;
- file fora de todos os dirs — deve retornar `None`.

Executar todas as permutações dos membros e comparar a identidade integral do owner,
não apenas sua layer.

Mutação que a propriedade deve matar: escolher primeiro ancestral; usar comprimento
de string em vez de componentes; resolver empate pela ordem de input; comparar paths
sem uma política explícita de normalização.

**RED exato:** ancestral menos profundo vence; permutar membros muda o owner; ou dois
registros semanticamente iguais sob a política de path escolhida dão respostas
diferentes. O alvo usa `max_by_key` somente pela profundidade; dirs empatados que ambos
casam são resolvidos pela ordem de iteração e constituem candidato RED.

**SPEC-GAP:** prompts não definem canonicalização filesystem-aware, symlinks,
case-folding, path absoluto versus relativo ou componentes `.`/`..`. Sem essa decisão,
esses casos classificam a fronteira da especificação; o RED mínimo independente dela
é dois membros com `dir` exatamente igual e metadados conflitantes.

## P3 — colisões depois de `-` → `_`

Exercitar manifests e membros com pares que normalizam para a mesma chave:

```text
package: "foo-bar" versus "foo_bar"
dependencies: foo-bar e foo_bar
renames: dep-x = { package = "real-a" }
         dep_x = { package = "real-b" }
```

Permutar a ordem textual das duas dependências e a ordem dos membros. Observar
separadamente `name`, `deps`, `renames`, `member_layer` e, quando consumido, o pacote
real escolhido pela rename.

Mutação que a propriedade deve matar: normalizar somente package mas não deps; esquecer
dev-dependencies; inserir em `HashMap`/`HashSet` e aceitar overwrite/colapso sem validar
a origem; permitir dois membros normalizados iguais.

**RED exato:** permutar entradas muda o valor normalizado ou a rename escolhida;
package/dependency/rename equivalentes recebem normalizações diferentes; ou uma
colisão conflitante é silenciosamente reinterpretada como outra dependência. O alvo
colapsa deps em `HashSet`, sobrescreve renames em `HashMap` e mantém membros com nomes
colidentes, sem detecção explícita.

**SPEC-GAP:** `-`→`_` é obrigatório, mas o prompt não decide se colisões idênticas podem
ser coalescidas e não oferece tipo de erro para colisões conflitantes. A propriedade
deve registrar `SPEC-GAP` se o comportamento for estável porém silencioso; vira RED
inequívoco quando a ordem muda o significado.

## P4 — inventário sob permutação e location canônica

Criar dois módulos, cada um com pelo menos dois arquivos Rust em ordem lexical inversa.
Distribuir constantes sentinela para cobrir exatamente:

- elegível citada e elegível não citada;
- `is_test_origin`, `is_in_data_table` e literal trivial, que não contam;
- arquivo não Rust, que não conta;
- arquivo em `format_syntax_modules`, que não conta.

Calcular manualmente `(cited, total)` por módulo. Executar todas as permutações dos
arquivos e exigir bytes idênticos dos diagnósticos, módulos em ordem canônica, mesmas
frações/percentuais e mesma location. Uma regra canônica de alto sinal é o menor path
pela ordem total de `Path` entre os arquivos elegíveis que contribuíram ao módulo.

Mutação que a propriedade deve matar: fixar location no primeiro arquivo; contar
constante excluída; aplicar exclusão depois de somar; agrupar pelo arquivo; usar mapa
sem ordem para emitir; escolher location de arquivo que contribuiu zero constantes.

**RED exato:** permutar files muda qualquer byte, path de location, contagem ou ordem
dos diagnósticos; uma constante excluída altera numerador/denominador; uma elegível não
aparece exatamente uma vez; ou location aponta para arquivo sem contribuição. O alvo
usa `or_insert(..., path)` e portanto congela o primeiro path encontrado: permutações
de arquivos do mesmo módulo mudam a location mesmo quando a mensagem permanece igual.

**SPEC-GAP:** a motivação histórica de V22 exige uma linha por módulo, mas não define
qual arquivo representa a location. “Menor path contribuinte” é uma proposta mecânica,
não uma decisão já presente no prompt. Contra a alegação congelada de bytes invariantes,
qualquer dependência da ordem ainda é RED; o SPEC-GAP é somente qual canonicalização
deve substituir o primeiro arquivo.

## Prioridade

1. **P4:** RED provável imediato; permutar dois arquivos do mesmo módulo troca a
   location.
2. **P1:** RED provável imediato com nomes duplicados conflitantes.
3. **P2:** RED provável imediato com dirs exatamente iguais e owners distintos.
4. **P3:** maior risco semântico; distingue ordem-dependência (RED) de política de
   colisão ainda não congelada (SPEC-GAP).
