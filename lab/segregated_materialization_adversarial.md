# Revisão adversarial — materialização segregada

## Escopo e premissa

Esta revisão usa somente o prompt `segregated-materialization`, o ADR-0020 e o domínio
público existente de refinamento necessário para interpretar os resultados. Ela não
pressupõe nenhuma implementação de `seal-refinement`.

O selo é um recibo de execução de um contrato congelado; não é prova de independência
entre agentes nem, por si só, prova de que os oráculos exercitam a mudança alegada. A
aceitação deve portanto testar tanto rejeições quanto a impossibilidade de produzir um
selo semanticamente vazio.

No comparador público existente, um único `Witness` faz o resultado agregado ser
`VIOLATED`, mesmo quando outras relações geram `Inconclusive`. Isso cria uma superfície
de lavagem de mutação que o selador precisa fechar ou registrar explicitamente.

## Ataques a contratos e oráculos vacuosos

### A1 — conjunto de oráculos vazio ou categoria ausente

Um manifesto sem `[[oracle]]`, ou contendo somente positivos, pode obter score trivial
por divisão especial, convenção `0/0 = 1`, ou ausência de falhas. Também não demonstra
poder discriminatório.

**Aceitação mecânica:** rejeitar antes de executar se não houver pelo menos um
`positive`, um `negative` e um `unknown`; as três contagens do selo devem ser maiores
que zero. `mutation_score` só pode ser calculado com denominador positivo.

### A2 — contrato sem relações ou relação tautológica

Contrato vazio preserva tudo. Uma relação `Preserve` que compara o mesmo observável
estável nos dois lados, `MayNormalize` cuja lista admite todos os resultados usados, ou
`MustNotInvent` sobre chave sempre ausente pode não observar a transformação de
interesse.

**Aceitação mecânica:** contrato sem relações é entrada inválida. Um teste de
sensibilidade deve alterar/remover cada relação coberta e demonstrar que ao menos um
negativo deixa de ser `VIOLATED`; caso contrário, o contrato ou o corpus é
observacionalmente morto. O selo não deve alegar esse critério se o manifesto v1 não
possuir dados para medi-lo.

### A3 — negativos não vinculados à mutação anunciada

Um negativo pode conter uma perda relevante que resulte em `UNKNOWN` e, ao mesmo tempo,
uma divergência-decoy em outra relação. O resultado agregado continua `VIOLATED` e o
negativo parece morto, embora a mutação importante não tenha sido detectada.

**Aceitação mecânica:** um negativo só conta se o resultado for `VIOLATED` **e** não
contiver inconclusivos; alternativamente, o manifesto precisa identificar a relação e
o observável que a mutação deve violar, e o recibo precisa conter a testemunha
correspondente. Apenas checar o discriminante agregado é insuficiente.

### A4 — positivo estreito e negativo irrelevante

Um par de commits sem relação com o prompt ou com a implementação pode satisfazer o
contrato. IDs livres como `rejects-field-removal` não conferem semântica ao par.

**Aceitação mecânica:** definir e testar uma regra de vinculação dos OIDs ao baseline e
à transformação autorizada. Se v1 não tiver metadados suficientes para isso, o selo
deve ser descrito apenas como execução reproduzível dos pares fornecidos, não como
evidência de causalidade ou segregação.

### A5 — `unknown` obtido por chave fictícia

Um oráculo `unknown` pode apontar para observável inexistente e receber
`MissingObservable`, apesar de a opacidade pretendida nunca ter sido exercitada. O tipo
`unknown` atual não declara razão, relação ou observável esperado.

**Aceitação mecânica:** fixtures devem demonstrar que trocar a opacidade real por uma
chave inexistente não sela. Para uma garantia forte, o manifesto deve congelar a razão
e o observável esperados, e o recibo deve compará-los exatamente. Sem isso, registrar
no selo a lista completa de inconclusivos e não chamar qualquer `UNKNOWN` de “opacidade
demonstrada”.

## Mutações que obrigatoriamente devem ser `VIOLATED`

Estas mutações pertencem ao conteúdo dos pares negativos, não ao parser do manifesto.
Para cada uma aplicável ao contrato, o resultado aceitável é exclusivamente
`VIOLATED`; `PRESERVED`, `UNKNOWN`, erro de extração ou ausência de execução bloqueiam o
selo.

1. Remover do artefato alvo um valor protegido por `Preserve`.
2. Trocar o valor alvo protegido por `Preserve`, preservando todo o resto.
3. Trocar o alvo de `MayNormalize` por valor fora de `accepted_targets` e diferente da
   origem.
4. Inventar no alvo um valor coberto por `MustNotInvent`.
5. Renomear apenas a chave alvo de uma relação, tornando o observável ausente. Isso não
   pode ser aceito como morte se a política de ausência o converter em `UNKNOWN`.
6. Tornar ambígua a identidade de uma captura de cardinalidade `One`; deve bloquear e
   não ser contabilizado como negativo morto.
7. Forçar parser não suportado, construção opaca, contrato parcial ou orçamento
   esgotado no caminho da mutação; todos são `UNKNOWN`, portanto bloqueiam um negativo.
8. Remover a própria relação que detectava a perda. O negativo deve deixar de contar;
   se o selo ainda passa, há testemunha-decoy ou contrato morto.

Teste metamórfico mínimo: para cada negativo válido, eliminar somente a divergência
esperada deve fazê-lo deixar de ser `VIOLATED`; reintroduzi-la isoladamente deve
restaurar `VIOLATED`. Isso distingue detecção causal de coincidência no fixture.

## Armadilhas de hash e identidade de bytes

- **TOCTOU:** calcular SHA-256 e depois reabrir prompt/contrato permite trocar bytes
  entre validação e execução. A mesma captura imutável de bytes deve alimentar hash e
  parse; mudança detectada bloqueia.
- **Hash do manifesto autocontaminado:** o selo inclui hash do manifesto, mas o
  manifesto não deve ser normalizado, reserializado nem reescrito. Hash é sobre seus
  bytes exatos, inclusive BOM, CRLF e newline final.
- **Representações frouxas:** rejeitar digest com tamanho diferente de 64 dígitos,
  caractere não hexadecimal, espaços, prefixo `0x` ou truncamento. Definir e testar se
  maiúsculas são aceitas; a saída deve usar uma única forma canônica.
- **Confusão de arquivos:** prompt e contrato devem ser abertos sob a raiz confinada e
  o hash deve corresponder ao papel correto; trocar os dois campos, mesmo com arquivos
  válidos, deve falhar.
- **Symlink e troca de symlink:** resolução lexical não basta. Symlink absoluto,
  `..`, symlink intermediário e troca concorrente não podem escapar da raiz nem mudar
  os bytes consumidos.
- **Colisão de destino:** `--output` não pode aliasar manifesto, prompt, contrato,
  `.git` ou entrada analisada, inclusive via symlink ou hard link. Falha não pode
  truncar o arquivo preexistente.

## Armadilhas de produtores

- Strings vazias ou só com whitespace não são identidades nominais úteis e devem ser
  rejeitadas.
- As três identidades devem ser comparadas após uma canonicalização documentada ou ser
  explicitamente byte-exatas. Casos `agent`, `agent `, `AGENT`, Unicode composto versus
  decomposto e separadores redundantes devem ter resultado definido e testado.
- Repetição não adjacente e repetição entre quaisquer dois dos três papéis devem
  bloquear.
- Strings diferentes (`agent/session-1`, `agent/session-2`) não provam executores
  diferentes. O selo e a mensagem de sucesso não podem usar termos como “isolamento
  verificado”, “sandbox provado” ou “independência certificada”.
- Produtores não devem participar do score nem ser usados como chave de ordenação que
  altere recibos.

## Armadilhas de Git e ordem

- `baseline_oid` deve ser OID completo, resolver para commit e ser idêntico ao OID
  declarado. Tag, branch, abreviação, árvore, blob e commit inexistente bloqueiam.
- Cada `before_ref` e `after_ref` deve ser resolvido uma única vez; o recibo contém os
  OIDs completos efetivamente usados. Ref móvel alterada durante a execução não pode
  produzir mistura de objetos.
- Ler objetos não pode executar checkout, hooks, filtros clean/smudge, LFS,
  submódulos, fetch ou qualquer rede. Um fixture com executáveis-sentinela deve provar
  contagem zero de invocações.
- Objeto ausente deve falhar localmente; não é permitido buscar promisor object.
- `before_ref == after_ref` deve ser rejeitado para negativos; caso contrário não há
  mutação. Para positivos/unknown, a permissão precisa ser deliberada e testada.
- IDs de oráculo vazios, só com whitespace ou duplicados devem bloquear. Ordenar
  duplicatas não resolve a ambiguidade.
- Permutar campos TOML, tabelas e oráculos deve produzir selo byte-idêntico. Isso inclui
  recibos, listas de inconclusivos, contagens e representação do score.
- A ordenação deve ser especificada sobre bytes/Unicode e independente de locale. IDs
  `a`, `A`, `á`, Unicode composto/decomposto e prefixos iguais exercitam o comparador.
- O JSON deve ter forma canônica única: ordem fixa de chaves, escapes definidos,
  newline final definida e score sem `float` (`"1/1"`, inteiros numerador/denominador,
  ou formato equivalente documentado).

## Armadilhas específicas de `UNKNOWN`

O domínio público possui razões distintas: `MissingObservable`, `AmbiguousIdentity`,
`UnsupportedParser`, `OpaqueConstruction`, `PartialContract` e `BudgetExhausted`. Elas
não são intercambiáveis para fins de evidência.

- Em positivo, qualquer inconclusivo impede `PRESERVED` e deve bloquear.
- Em negativo, qualquer `UNKNOWN` agregado bloqueia; um `VIOLATED` com inconclusivos
  também não deve lavar a parte inconclusiva sem vinculação explícita da testemunha.
- Em oráculo `unknown`, `PRESERVED` e `VIOLATED` bloqueiam. O recibo deve registrar todas
  as razões, relações e observáveis, em ordem determinística.
- `BudgetExhausted` não deveria provar opacidade: é insuficiência operacional.
  `UnsupportedParser`, `PartialContract` e `MissingObservable` também podem representar
  fixture defeituoso. Se v1 aceitar qualquer razão, essa limitação precisa estar
  explícita e coberta por teste adversarial.
- Como violação prevalece sobre inconclusivo no veredito agregado existente, os testes
  precisam inspecionar a estrutura do veredito, não apenas exit code ou texto iniciado
  por `VIOLATED`.

## Atomicidade e não alteração

Critérios mecânicos para toda falha (parse, hash, Git, produtor, orçamento, veredito ou
escrita): exit `2`; nenhum selo parcial; nenhum temporário residual; destino anterior
permanece byte-idêntico. Para sucesso, gravar temporário no mesmo diretório, sincronizar
conforme a garantia declarada e renomear atomicamente sem tocar entradas.

Capturar antes/depois, no mínimo: `git status --porcelain=v2`, hashes dos arquivos de
entrada e destino preexistente, refs, índice, HEAD e lista de arquivos. Rodar também com
working tree previamente suja e confirmar igualdade byte a byte do snapshot de estado.
Falha de rename, falta de espaço, permissão negada e interrupção antes do rename devem
ser injetadas.

## Gate mecânico de aceitação

O piloto só passa se todos os itens abaixo forem observáveis em testes automatizados:

1. Fixture válida contém as três categorias, score exato de 100% e selo canônico.
2. Cada negativo isolado produz `VIOLATED` sem ser salvo por testemunha alheia; toda
   variante `PRESERVED` ou `UNKNOWN` termina em exit `2` sem publicação.
3. Positivo exige `PRESERVED`; unknown exige `UNKNOWN`; razões e inconclusivos são
   preservados no recibo.
4. Contrato vazio, categoria ausente, ID vazio/duplicado, produtor inválido/repetido,
   hash inválido e OID inválido são rejeitados antes de publicar.
5. Duas permutações semanticamente idênticas do TOML produzem exatamente os mesmos
   bytes de selo.
6. Hash e parse usam os mesmos bytes capturados; ataques de symlink, alias e TOCTOU não
   escapam do confinamento.
7. Git resolve cada ref uma vez, registra OID completo e não aciona nenhum mecanismo
   executável ou de rede.
8. Matriz de falhas de I/O prova atomicidade e preservação do destino anterior.
9. Snapshot do repositório e das entradas é idêntico antes/depois, tanto em sucesso
   quanto em falha e com working tree suja.
10. Mensagens e schema declaram produtores como recibos nominais, sem elevar strings a
    prova de isolamento.

Se A3, A4 ou A5 não puderem ser satisfeitos pelo schema v1, isso é limitação material:
o gate deve reduzir a alegação do selo em vez de inferir causalidade que os dados não
permitem verificar.

## Matriz de ataques priorizada

| Prioridade | Ataque | Falso resultado procurado | Oráculo mecânico de defesa |
|---|---|---|---|
| P0 | Lavagem: mutação relevante `UNKNOWN` + witness decoy | negativo contado como morto | exigir witness vinculado ou zero inconclusivos |
| P0 | Oráculos sem vínculo causal com baseline/prompt | selo válido sobre commits irrelevantes | regra verificável de vínculo; caso impossível, limitar alegação |
| P0 | Hash/parse em leituras diferentes (TOCTOU) | bytes selados diferem dos executados | uma captura de bytes alimenta hash e parse |
| P0 | Saída aliasa entrada ou destino é truncado na falha | manifesto/contrato/repositório alterado | rejeitar alias; snapshot byte-idêntico; rename atômico |
| P0 | Git aciona filtro, LFS, hook, submódulo ou fetch | efeito externo e entrada não imutável | sentinelas com zero invocações e rede desabilitada |
| P1 | Contrato vazio/categoria ausente | score trivial `1.0` | relações não vazias e ao menos um oráculo de cada tipo |
| P1 | `UNKNOWN` por chave fictícia ou orçamento | falsa prova de opacidade | razão/observável congelados ou limitação explícita |
| P1 | Ref móvel resolvida mais de uma vez | par híbrido não reproduzível | resolver uma vez e operar somente sobre OIDs registrados |
| P1 | IDs duplicados/vazios | recibos ambíguos e ordem instável | validar unicidade e não-vazio antes da execução |
| P1 | Symlink, `..`, hard link e troca concorrente | escape de raiz ou arquivo trocado | confinamento real e identidade estável do arquivo |
| P1 | Produtores equivalentes escritos diferente | falsa segregação nominal | canonicalização definida + disclaimer obrigatório |
| P2 | Permutação TOML/locale/Unicode | selo não determinístico | golden byte a byte sob permutações e locale distinto |
| P2 | Score em ponto flutuante | bytes/valor ambíguos | razão inteira ou representação decimal canônica |
| P2 | `before_ref == after_ref` em negativo | “mutação” inexistente | rejeição explícita |
| P2 | Falha de disco/rename/interrupção | temporário ou parcial residual | fault injection e diretório final sem resíduos |
