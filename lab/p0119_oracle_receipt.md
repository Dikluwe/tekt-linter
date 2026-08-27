# Recibo de oráculos P0119

## Autoridade e isolamento

- Regime: protocolo completo da ADR-0020.
- Papel: autor de oráculos; sem autoridade para editar contrato, implementação, spec,
  prompts ou emitir veredito da implementação.
- Spec congelada: `00_nucleo/tekt-linter-passo-0119-inicializar-configuracao.md@611fabd`.
- SHA-256 normativo da spec: `e9440017a99a301f1e042cff95f87e07603397eacd59f173175e56c28b1aa71e`.
- Contrato congelado: `lab/p0119_contract.md@0998ce8`.
- Baseline funcional: `7013ac76378f0800c6c35565081ec74e251757aa`.
- Implementação candidata: não lida; informada como ainda inexistente.
- Escrita exercida: somente `tests/p0119_*_assessment.rs` novos e este recibo.
- Classificação: **executado sem atestação de isolamento**; checkout e filesystem são
  compartilhados, e a restrição foi observada operacionalmente, não imposta por sandbox.

## Oráculos produzidos

- `tests/p0119_pure_assessment.rs`: precedência pura de `Present` sem exigir topologia,
  usando apenas nomes integralmente definidos pela API L1 proposta.
- `tests/p0119_cli_assessment.rs`: goldens mínimo/completo, todos os subconjuntos
  opcionais, determinismo por ordem de criação, caminhos relativo/absoluto, `Present`
  válido/inválido, razões `Unknown`, symlinks de raiz/destino, diretório/FIFO, raiz
  inexistente/arquivo, Unicode irrelevante, lint sem escrita, flags proibidas e reserva
  de `init`.

Os goldens são literais derivados do contrato e não importam template/serializador do
alvo. Estados comparam código, canais e bytes exatos; sentinelas verificam não
sobrescrita/escape.

## RED causal

No baseline, a suíte pura deve falhar na compilação porque a API pública
`entities::config_init` ainda não existe. O teste CLI também deve falhar funcionalmente
porque `init` ainda não é subcomando. Esse RED é esperado e antecede a implementação.

Reprodução no checkout do baseline desta execução:

- `cargo test --test p0119_pure_assessment`: RED de compilação `E0432`, módulo
  `crystalline_lint::entities::config_init` ausente;
- `cargo test --test p0119_cli_assessment`: compila; 2 passam e 9 falham, com os casos de
  criação/recusa recebendo o parsing vigente em vez do subcomando futuro;
- `git diff --check`: PASS.

Hashes SHA-256 dos oráculos após `rustfmt`:

- `tests/p0119_pure_assessment.rs`:
  `a6702a05e400491d0ba26f0cf82861c6621a87d064a0d44441ea427db7f7236d`;
- `tests/p0119_cli_assessment.rs`:
  `25484573de8da088abb536e1b95b7c2b1b3cceec6b76bbae7fb8cdcc2aec8c1d`.

## Gaps deliberados

- Corrida entre plano e publicação e falhas create/write/sync/load/publish/cleanup exigem
  adapter injetável público ainda não especificado na API L1 proposta; não foi inventada
  uma API de teste.
- Raiz inacessível é dependente de credenciais (especialmente execução como root) e falha
  de inventário determinística também requer injeção; não entram como gate reproduzível.
- Socket Unix e nomes não UTF-8 não foram materializados nesta tranche; FIFO cobre um
  tipo especial obrigatório e Unicode não conflitante está coberto.
- Identidade do inode validado, ausência de releitura pós-publicação, imprevisibilidade do
  temporário e falha de cleanup requerem instrumentação do adapter de publicação.
- O contrato não congela o caminho do módulo Rust. O teste puro assume somente
  `entities::config_init`; caso a integração escolha outro caminho sem mudança do
  contrato, será necessário um re-export compatível ou decisão explícita da autoridade,
  não adaptação silenciosa do oráculo.
