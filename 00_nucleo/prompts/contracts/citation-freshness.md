# Prompt: porta de frescura de citação V21

Hash do Código: ausente

**Camada da porta:** L1 (`01_core/contracts/citation_freshness.rs`)
**Adapter inicial:** L3 (`03_infra/citation_freshness.rs`)
**Consumidor:** V21 `HardcodedContextualValue`

## Intenção

Permitir que V21 decida se uma citação `// ref:` está fresca sem importar filesystem em
L1. A porta expõe três estados fechados; falha externa nunca é convertida em validade.

## API L1

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationStaleReason { MissingFile, InvalidLine, EmptyLine }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationUnknownReason {
    OutsideRoot, Symlink, InvalidRoot, Io, InvalidUtf8,
    BudgetExceeded, ConcurrentMutation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationFreshness {
    Valid,
    Stale(CitationStaleReason),
    Unknown(CitationUnknownReason),
}

pub trait CitationFreshnessResolver {
    fn resolve(&self, path: &str, line: usize) -> CitationFreshness;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UnknownCitationFreshness;

impl CitationFreshnessResolver for UnknownCitationFreshness {
    fn resolve(&self, _path: &str, _line: usize) -> CitationFreshness {
        CitationFreshness::Unknown(CitationUnknownReason::Io)
    }
}
```

L1 usa mocks puros. A porta não expõe bool, PathBuf, erro de I/O concreto ou tipo L3.
`UnknownCitationFreshness` é o fallback fail-closed para consumidores sem adapter; nunca
autoriza silêncio de `Ref`.

## Semântica L3

Superfície pública do adapter:

```rust
// módulo crystalline_lint::infra::citation_freshness
pub struct FsCitationFreshnessResolver { /* estado privado */ }

impl FsCitationFreshnessResolver {
    pub fn new(root: PathBuf, max_bytes: u64) -> Self;
}
```

O construtor não faz I/O nem falha. `max_bytes == 0` faz toda resolução retornar
`Unknown(BudgetExceeded)`; raiz inválida é classificada durante `resolve`.

O adapter recebe raiz e orçamento máximo positivo em bytes. `path` é relativo, UTF-8 e
não vazio. Absoluto, prefix/root, `..` ou vazio são `Unknown(OutsideRoot)`; `.` é removido
lexicalmente. Root symlink ou qualquer componente symlink é `Unknown(Symlink)`, mesmo
quando aponta para dentro. Root ausente/não diretório é `Unknown(InvalidRoot)`.

- ausente: `Stale(MissingFile)`;
- linha zero/além de EOF: `Stale(InvalidLine)`;
- linha com `trim()` vazio: `Stale(EmptyLine)`;
- tamanho/leitura acima do orçamento: `Unknown(BudgetExceeded)`;
- conteúdo não UTF-8: `Unknown(InvalidUtf8)`;
- erro de metadata/open/read ou diretório: `Unknown(Io)`;
- metadata relevante divergente antes/depois: `Unknown(ConcurrentMutation)`;
- restante com linha existente não vazia: `Valid`.

Linhas são one-based por `str::lines()`; CRLF é removido logicamente e última linha sem
newline conta. O adapter é read-only, não usa rede, processo, hooks ou escrita.

## Fronteira

L3 não conhece listas, triviais, strict, Spec/Rationale, severidade ou mensagens V21.
L1 não conhece root, metadata, bytes, encoding ou symlink.

## Critérios

Mock L1 cobre os três estados. Gate L3 cobre valid/stale/unknown e razões, confinamento,
symlink, UTF-8, orçamento, determinismo e fingerprint inalterado.

## Histórico

| Data | Estado | Motivo |
|---|---|---|
| 2026-08-24 | Proposto pelo P0088 | Remover filesystem de V21/L1 sem falso silêncio |
