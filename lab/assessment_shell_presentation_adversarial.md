# Triagem adversarial — apresentação Shell

## Convenções de classificação

- **DEFEITO:** comportamento observável contradiz uma alegação congelada atual.
- **SPEC-GAP histórico:** prompt antigo e API atual divergem; não é RED automático sem
  decidir qual contrato vigora.
- **NÃO OBSERVÁVEL:** a API de apresentação não contém evidência suficiente para provar
  a alegação, mesmo que a saída pareça correta.

São propostas somente quatro propriedades/mutações de alto sinal.

## P1 — ordem total sob empates e permutações

Construir violações com o mesmo nível, path e linha, mas sentinelas distintas em
`column`, `rule_id` e `message`. Gerar todas as permutações da mesma coleção; em cada
uma, chamar `sort_violations`, depois comparar tanto a sequência estrutural quanto os
bytes de `format_text` e `format_sarif`.

Ordem exigida pelo assessment:

```text
level desc, path asc, line asc, column asc, rule_id asc, message asc
```

Mutações que a propriedade mata: remover qualquer tie-breaker; trocar asc/desc; usar
somente uma chave concatenada ambígua; depender da estabilidade do sort para empates.

**RED exato — DEFEITO:** duas permutações da mesma coleção terminam em sequências ou
saídas diferentes, ou a sequência viola a tupla acima. O alvo compara apenas nível,
path e linha; portanto violações empatadas nessas três chaves mantêm a ordem de entrada
e são um candidato RED direto.

**SPEC-GAP histórico:** o prompt afirma que o formatter apenas preserva ordem já
fornecida por L4 e documenta somente nível/path/linha; o assessment atual atribui à API
pública `sort_violations` uma ordem total com mais três desempates. O RED é real contra
o assessment congelado, mas a correção exige atualizar a responsabilidade histórica.

**NÃO OBSERVÁVEL:** determinismo do pipeline paralelo inteiro não é provado por uma
função ordenadora isolada; esta propriedade prova somente independência da permutação
recebida.

## P2 — strings hostis, Unicode, controles e path não UTF-8

Usar `rule_id` e `message` contendo, separadamente e combinados:

```text
aspas, barra e contrabarra, \n, \r, \t, NUL, ESC, U+2028/U+2029,
emoji, combining mark, bidi override e texto em NFC/NFD
```

No SARIF, parsear o JSON e exigir igualdade escalar exata entre strings de entrada e
`ruleId`/`message.text`; escapes de serialização não contam como perda. No texto,
exigir que a concatenação preserve os escalares exatos e registrar separadamente que
newline/ESC podem quebrar framing visual ou injetar controle terminal.

Em Unix, criar `Path` com bytes inválidos em UTF-8, por exemplo `b"a\xffb.rs"`, e
formatar nas duas saídas. Comparar contra os bytes originais, não contra
`to_string_lossy()`.

Mutações que a propriedade mata: interpolação JSON manual; double escaping; truncar
em NUL/newline; normalizar Unicode; substituir byte inválido por U+FFFD; omitir path,
rule ou message.

**RED exato — DEFEITO:** JSON deixa de parsear; valor parseado difere da string de
entrada; ou path não UTF-8 é emitido com U+FFFD e portanto não preserva os bytes. O
alvo usa `Path::display()` no texto e `to_string_lossy()` no SARIF, tornando a perda de
path não UTF-8 um candidato RED onde esse path é representável.

**SPEC-GAP histórico:** “preservar caracteres de controle” não define se texto humano
deve escapar controles para manter um diagnóstico por registro. Preservação literal e
framing seguro são objetivos incompatíveis sem uma regra de escape. Newline/ESC que
injetam linhas ou efeitos visuais devem ser classificados como `SPEC-GAP` até essa
política ser congelada, não automaticamente como perda de conteúdo.

**NÃO OBSERVÁVEL:** depois de conversão lossy não é possível reconstruir, pelo SARIF,
qual sequência original de bytes não UTF-8 existia; igualdade do texto substituído não
prova fidelidade do path.

## P3 — coerência SARIF entre resultados, catálogo, níveis e posição

Criar exatamente um resultado para cada `V0`–`V25`, com níveis cobrindo Fatal, Error,
Warning e Info e posições sentinela distintas. Parsear o documento e exigir:

1. `version == "2.1.0"`, um run, 26 regras com IDs únicos e conjunto exato
   `{V0, ..., V25}`;
2. todo `result.ruleId` referencia exatamente uma entrada de `driver.rules`;
3. Fatal/Error viram `error`, Warning vira `warning`, Info vira `note`;
4. `startLine == location.line` e `startColumn == location.column + 1`;
5. ordem dos `results` igual à ordem recebida pelo formatter.

Adicionar um `rule_id = "VX"` hostil. A propriedade não presume silenciosamente o que
fazer: ou o formatter rejeita, ou o catálogo inclui metadata coerente, ou a API declara
formalmente que aceita resultados sem descritor. Emitir `VX` apontando para nenhuma
regra é a observação que força a decisão.

Mutações que a propriedade mata: omitir/duplicar rule descriptor; catálogo V0–V12
congelado; mapear Fatal para valor SARIF inexistente; esquecer `+1` da coluna; reordenar
results; ligar resultado ao ID errado.

**RED exato — DEFEITO:** falta ou duplicidade em V0–V25; result conhecido sem descriptor;
nível divergente do mapeamento; posição divergente; JSON inválido; ou ordem recebida
alterada.

**SPEC-GAP histórico:** o prompt congela metadata V0–V12 e “exatamente 13”, enquanto o
assessment exige V0–V25. O alvo atual tem V0–V25; a divergência documental é
`SPEC-GAP`, conforme o próprio gate, não defeito atual. A política para rule IDs fora
do catálogo também não está especificada.

**NÃO OBSERVÁVEL:** coerência entre `defaultConfiguration.level` e o nível normativo de
cada regra não pode ser derivada apenas de um `Violation` arbitrário: a struct permite
qualquer par `rule_id`/level. O formatter pode preservar o par, mas não provar que a
regra produtora escolheu o nível correto.

## P4 — monotonicidade e tabela completa de `should_fail`

Avaliar todas as coleções formadas pelos quatro níveis, incluindo vazio, permutações e
duplicatas, nos modos `Error` e `Warning`. A tabela unitária exigida é:

| Nível | `Error` | `Warning` |
|---|---:|---:|
| Fatal | true | true |
| Error | true | true |
| Warning | false | true |
| Info | false | false |

Leis adicionais:

```text
should_fail(A) => should_fail(A union B)
should_fail(permutation(A)) == should_fail(A)
should_fail(A + duplicates(A)) == should_fail(A)
should_fail(A, Error) => should_fail(A, Warning)
should_fail([], mode) == false
```

Mutações que a propriedade mata: implementar por primeiro/último elemento; fazer Fatal
obedecer ao threshold; inverter Warning/Error; considerar Info; usar `all` em vez de
`any`; tornar duplicata relevante.

**RED exato — DEFEITO:** qualquer célula difere da tabela, adicionar uma violação muda
`true` para `false`, permutar/duplicar muda a decisão, ou uma coleção falha em `Error`
mas não em `Warning`.

**SPEC-GAP histórico:** o prompt contém frases conflitantes sobre quais regras Fatal
podem ser omitidas por `--checks`, mas `should_fail` recebe somente as violações já
produzidas. A política de habilitação não deve ser imputada a esta função.

**NÃO OBSERVÁVEL:** `should_fail` não observa regras suprimidas, execução do CLI nem o
exit code real. Um `true` correto aumenta confiança na decisão pura, mas não prova que
o processo efetivamente sai com código 1.

## Prioridade

1. **P1:** RED provável e diretamente reproduzível nos desempates ausentes.
2. **P2:** RED provável para paths Unix não UTF-8; controles expõem um SPEC-GAP útil.
3. **P3:** protege consumidores SARIF e separa claramente catálogo atual do histórico.
4. **P4:** propriedade barata, exaustiva e provavelmente PASS.
