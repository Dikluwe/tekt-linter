# Prompt: regra V17 CompoundGuard
Hash do Código: 28ae2b6c

Owner exclusivo: `01_core/rules/compound_guard.rs`.

Detectar guards de braços decisórios com composição lógica que esconde decisões. Operar
sobre IR pura, preservar localização e não duplicar métricas de outras regras.

## Critério observável

V17 emite uma ocorrência por guard composto elegível, preserva localização e fica silenciosa
para guard simples, sem ativar V16/V18–V20.
