# ADR: Representação Intermediária AST com Tree-Sitter

## Contexto
Queremos criar um linter arquitetural, o qual julgará metadados de código-fonte (imports, headers, call-expressions) em relação à estratificação e restrições estabelecidas na Topologia Cristalina/Tekt. Diferentes abordagens existem para este enforcement: expressões regulares sobre texto (rápido, mas rústico e frágil em blocos indentados ou string literals), parsing via Syn (poderoso para macros, mas engessado a compiladores de Rust e inviável para TS/Python), e bibliotecas multi-idiomáticas de Análise de Sintaxe.

A decisão pauta que deve suportar múltiplas linguagens com as mesmas regras puras (L1), desde que as gramáticas alimentem os tipos.

## Decisão
A interface central de checagem do linter receberá uma Representação Intermediária agnóstica (`ParsedFile`), preenchida obrigatoriamente através do parser independente **Tree-Sitter**.

A lógica de parser L3 traduzirá os nós específicos da linguagem (em `tree-sitter-rust`) aos construtos universais de Arquitetura Cristalina do `ParsedFile` em L1.

## Justificativa
1. O Tree-Sitter cria a AST extremamente rápido sem um ciclo de build completo, suportando parsing incremental se necessário para o futuro (LSP server embedded).
2. Ele é perfeitamente agnóstico por design. A regra "Forbidden Import" tem uma essência independente. Apenas a gramática extratora L3 (TS parser vs PY parser) difere entre ecossistemas. O núcleo sobrevive intacto.
3. Não sofre as instabilidades e falsos positivos de `regex`, distinguindo facilmente comentários de lógicas reais ou nós semânticos (ex: invocação de IO proibida com `std::fs` mapeará perfeitamente à rule do Tekt, contra string literals ou imports sombreados).

## Consequências
1. **Pontos de Falha Adicionais (Bindings)**: Usar binding de C no core do utilitário linter obriga o deploy ter build estático limpo (Rust compila nativamente `tree-sitter-cli` sem sustos por padrão, mitigando este risco).
2. O L3 cresce em complexidade para cada linguagem injetada (precisa de map de cada arvore de TS no `ParsedFile`), enquanto o núcleo encolhe. Exatamente como a Casca Cristalina prescreve e previne que resíduos contaminem nossa prova matemática.
