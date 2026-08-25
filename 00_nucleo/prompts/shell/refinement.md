# Prompt: use-case e apresentação de refinamento
Hash do Código: c79aa862

Owner exclusivo: `02_shell/refinement.rs`.

Orquestrar portas de extração/snapshot e apresentar veredito com protocolo de saída
0/1/2. Shell não executa Git diretamente nem redefine a comparação L1.

## Critério observável

Gates CLI demonstram exit 0/1/2 para preserved/violated/unknown e formatos text/SARIF
preservam testemunhas sem chamar Git fora da porta injetada.
