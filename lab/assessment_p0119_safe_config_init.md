# Assessment P0119 — inicialização segura de `crystalline.toml`

Data: 2026-08-26  
Baseline funcional: `7013ac76378f0800c6c35565081ec74e251757aa`  
Spec final: `611fabd`, SHA-256 `e9440017a99a301f1e042cff95f87e07603397eacd59f173175e56c28b1aa71e`  
Contrato: `0998ce8`, SHA-256 `ca400033dd82e2c02f9a75ea8b9473c0c8d245ae7a41b5e0e5b79eb3c2b07064`  
Estado: **APPROVE** adversarial e independente; candidata pronta para integração

## Cadeia causal

O passo foi congelado antes do código. O adversário encontrou gaps de alias, apresentação,
precedência, sequência transacional e cleanup; cada mudança invalidou o contrato anterior.
Após três revisões, contrato e adversário emitiram selo pré-implementação. Em seguida, um
autor distinto criou oráculos RED sem acesso ao patch: 9 falhas funcionais na CLI e ausência
da API L1.

Papéis foram nominalmente separados, mas compartilharam checkout/filesystem. A execução é
auditável e reproduzível, sem atestação forte de isolamento por capability.

O patch final recebeu `APPROVE` do adversário e do verificador independente após corrigir
os ataques de ownership do temporário, recibo L1 e proveniência de flags com valor default.

## Materialização

- `crystalline-lint init [PATH]` é subcomando explícito; lint comum permanece somente
  leitura;
- L1 decide `Present`, `Unsafe`, topologia reconhecida e `Unknown` de forma total e
  materializa `InitReceipt`/`PublicationFailure`;
- L3 inventaria a raiz sem seguir symlink e serializa bytes determinísticos;
- temporário recebe token aleatório de 128 bits via gerador do sistema, é escrito,
  sincronizado e validado pelo loader real;
- publicação usa hard link no-clobber, preservando o inode validado e o destino concorrente;
- cleanup só ocorre depois de a operação adquirir a posse do temporário com `create_new`;
- L2 mantém canais, mensagens e códigos de saída normativos;
- nenhum `--force`, criação de diretórios ou inferência de política específica foi incluído.

## Oráculos

- `tests/p0119_pure_assessment.rs`: precedência pura de destino;
- `tests/p0119_cli_assessment.rs`: 12 casos cobrindo goldens, 16 subconjuntos, ordem,
  relativo/absoluto, estados, symlink/FIFO, flags proibidas e lint sem escrita;
- teste de regressão local rejeita flags de lint posicionadas antes de `init`;
- testes locais adicionais: decisão total, projeção injetada de `ErrorKind`, corrida entre
  duas publicações, colisão de temporário alheio, cleanup e nome não UTF-8.

## Mutação dirigida

Comando final:

```sh
cargo mutants --in-place --baseline skip \
  --file 01_core/entities/config_init.rs \
  --file 02_shell/config_init.rs \
  --file 03_infra/config_init.rs \
  --exclude-re 'os_bytes' --timeout 90 -- \
  --lib \
  --test p0119_pure_assessment \
  --test p0119_cli_assessment
```

Resultado repetido sobre os bytes finais: 49 mutantes testados, 45 válidos mortos, 4
inviáveis em compilação, zero sobreviventes; score válido `45/45 = 1.0`.

`os_bytes` foi excluído porque o gerador também muta o ramo `#[cfg(not(unix))]`, inativo no
domínio Unix/Linux congelado. O ramo Unix de nome não UTF-8 permanece coberto por teste
funcional. Uma campanha anterior, preservada como diagnóstico, expôs projeções de erro e
cleanup não exercitadas; elas foram extraídas/testadas antes da campanha final.

Sobre os bytes finais, a campanha da proveniência de argumentos L2 matou `3/3`; a campanha
das duas mutações geradas por `cargo-mutants` no trecho inicial de L4 matou `2/2`. Após o
adversário encontrar que uma falha de `create_new` poderia acionar cleanup sem posse, foi
acrescentado um oráculo de colisão que preserva os bytes do arquivo alheio e impede a
criação do destino; essa obrigação também integra a campanha final L1/L2/L3 de 49 mutantes.

## Gates

- oráculos independentes P0119: 13/13;
- testes locais do fragmento: 7/7 (6 unitários e 1 regressão CLI);
- `cargo test --all-targets`: passou, incluindo 657 unitários e integrações;
- ratchet de complexidade: 2/2;
- `cargo fmt --all --check`: passou;
- `git diff --check`: passou;
- `--fix-hashes --dry-run`: `Nothing to fix`;
- auto-lint V1/V2/V5/V7/V15/V26: zero violações.

## Limites

Falhas persistentes de cleanup, raiz inacessível sob credenciais privilegiadas e crash
abrupto permanecem nos limites explícitos da spec. O veredito cobre somente o fragmento
P0119 e não prova segurança geral contra troca maliciosa de mount/ancestral após a
validação.
