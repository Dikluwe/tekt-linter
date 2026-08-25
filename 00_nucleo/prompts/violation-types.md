# Prompt: tipos de violação
Hash do Código: 5cdf39c4

Owner exclusivo: `01_core/entities/violation.rs`.

Modelar severidade, localização e `Violation` como dados puros, clonáveis e ordenáveis.
Paths e linhas são preservados; apresentação, política de saída e regras ficam fora.

## Critério observável

Testes cobrem variantes, ordenação de severidade, clone e igualdade; nenhum formatter ou
regra é importado pela entidade.
