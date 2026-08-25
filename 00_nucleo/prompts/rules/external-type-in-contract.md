# Prompt: regra V14 ExternalTypeInContract
Hash do Código: PENDING_P0106

## Owner

`01_core/rules/external_type_in_contract.rs`, exclusivamente.

## Instrução

Em L1, inspecionar imports externos (`target_layer == Unknown`) e emitir V14 Error para
dependências não autorizadas pela entidade `L1AllowedExternal` injetada.

## Restrições

- regra pura, sem ler configuração;
- aplicar somente a arquivos L1 e ignorar imports internos;
- reportar localização e pacote proibido sem modificar a allowlist.

## Critérios

Permitidos e stdlib ficam silenciosos; externo não permitido gera uma violação estável;
outras camadas não recebem V14.
