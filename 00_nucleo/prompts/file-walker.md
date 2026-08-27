# Prompt: walker do projeto
Hash do Código: d4ba8ee0

Owner exclusivo: `03_infra/walker.rs`.

Enumerar arquivos analisáveis deterministicamente, classificar camada/linguagem e separar
erros de infraestrutura. Não seguir symlink nem absorver falha como inventário completo.

Para V2, classificar cada fonte aceita como `Production` ou `Test` pela convenção nominal
fechada da linguagem. Se nome ou stem não for UTF-8, preservar `Unknown` e emitir
`SourceError::Unreadable` com razão explícita, projetável como V0, em vez de assumir produção. As
convenções consideram a extensão completa; em particular, prefixo/sufixo de teste C só
vale para `.c`, nunca para `.h`. A busca de teste adjacente usa o mesmo domínio aceito pelo walker:
arquivo excluído, diretório ou symlink não cobre produção. O conteúdo não participa da
classificação e artefatos `Test` não consultam adjacência própria.

## Critério observável

O mesmo conjunto de arquivos produz ordem idêntica; symlink/inacessível vira erro separado
e nunca desaparece como sucesso parcial.
