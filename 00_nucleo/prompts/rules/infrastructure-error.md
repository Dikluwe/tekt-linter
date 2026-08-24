# Prompt L0 — projeções puras V0/PARSE

Hash do Código: ausente

**Camada:** L1 (`01_core/rules/infrastructure_error.rs`)
**Consumidor:** L4 (`04_wiring/main.rs`)

## Decisão

Transformar os erros de domínio L1 `SourceError` e `ParseError` na entidade L1
`Violation` é política pura e pertence a L1. L4 apenas encaminha cada erro ao projetor;
L2 formata a violação pronta e L3 somente produz os erros.

## API pública

```rust
pub fn source_error_to_violation(err: &SourceError) -> Violation<'static>;
pub fn parse_error_to_violation(err: ParseError) -> Violation<'static>;
```

## Mapeamento normativo exato

- `SourceError::Unreadable { path, reason }` → V0 `Fatal`, mensagem
  `Arquivo ilegível: {reason}`, `Cow::Owned(path.clone())`, linha/coluna zero.
- `ParseError::SyntaxError { path, line, column, message }` → PARSE `Error`, mensagem
  `Erro de sintaxe: {message}`, `Cow::Owned(path)`, posição preservada.
- `ParseError::UnsupportedLanguage { path, language }` → PARSE `Warning`, mensagem
  `Linguagem não suportada: {language:?}`, `Cow::Owned(path)`, linha/coluna zero.
- `ParseError::EmptySource { path }` → PARSE `Warning`, mensagem
  `Arquivo vazio ignorado`, `Cow::Owned(path)`, linha/coluna zero.

Para estas quatro modalidades, e somente para elas salvo norma posterior, `0:0`
representa localização indisponível no fonte; não é uma coordenada one-based real.

Cada entrada produz exatamente uma violação. Strings vazias, Unicode e conteúdo hostil
são preservados por interpolação, sem normalização. A projeção é determinística, não
modifica a entrada emprestada e não acessa filesystem, config, ambiente, relógio, rede
ou processo.

## Fronteira

O módulo não conhece parser, walker, rayon, CLI, formatter ou wiring. Não decide exit
code nem ordenação. L4 não replica IDs, mensagens, severidades ou posições.

## Critérios

Gate black-box instancia diretamente todas as variantes públicas, chama os dois
projetores e observa `rule_id`, nível, mensagem, path/ownership, linha e coluna. Deve
cobrir clones, repetição, Unicode e controles hostis sem filesystem real.

## Histórico

| Data | Estado | Motivo |
|---|---|---|
| 2026-08-24 | Proposto pelo P0089 | Remover política V0/PARSE de L4 e abrir seam cega |
