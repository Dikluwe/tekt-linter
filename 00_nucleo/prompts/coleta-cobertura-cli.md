# Coleta de execução das regras pela CLI

A ferramenta de laboratório coleta a execução das checagens do linter Rust
em alvos de teste explicitamente escolhidos. Compila com instrumentação LLVM,
executa cada alvo de integração separadamente e usa os perfis dos processos
do binário `crystalline-lint`. Mede entrada em funções de checagem dos arquivos
de regras. Contadores de funções auxiliares e do executável do teste não
afirmam execução de uma regra.

A coleta exige o sucesso dos testes selecionados. Emite JSON versão 1 com a
revisão Git, a lista dos arquivos de teste medidos e as associações entre regras
e testes. Inclui regras sem execução observada com lista vazia. A associação
indica que o código foi exercitado nesse alvo de teste; o significado das
asserções continua no código dos testes. A ferramenta é opcional, usa
`llvm-tools` da mesma toolchain do Rust e mantém artefatos no diretório de build.
