# Prompt: regra V27 MergeableDecisionArms
Hash do Código: cd8d1f43

## Owner

`01_core/rules/mergeable_decision_arms.rs`, exclusivamente.

## Instrução

Detectar grupos maximais de braços adjacentes de um `match` Rust cuja consequência seja
estruturalmente idêntica e cuja união por `|` seja sintaticamente comprovável. O achado
indica possível erro de copiar/colar ou duplicação intencional consolidável.

## Restrições

- L1 decide somente sobre IR explícita e não conhece parser, filesystem ou compilador;
- `Unknown` nunca produz achado;
- preservar operadores, guards, bindings, atributos, macros e ordem;
- bloquear wildcard, range, corpo vazio, placeholder, macro, `cfg` e qualquer binding
  enquanto a IR não possuir autoridade de tipos;
- não comparar texto truncado nem usar fingerprint sem confirmação integral;
- não atravessar braço intermediário;
- emitir `Info`, sem autofix de produção e sem compensar V19/V20.

## Critérios

- um grupo adjacente maximal produz exatamente um V27 no segundo braço;
- diferenças de corpo, guard, modo de binding ou proveniência bloqueiam o grupo;
- saída é determinística por ordem do arquivo e da decisão;
- a mensagem oferece correção de copiar/colar ou composição por `|` sem decidir intenção.
