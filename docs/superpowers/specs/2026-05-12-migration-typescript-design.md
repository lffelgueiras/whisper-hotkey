# Design — Migração de `whisper-hotkey` de Python para TypeScript (Tauri)

**Data:** 2026-05-12
**Autor:** atlas@lucabeltda.com.br
**Status:** Aprovado para implementação

---

## 1. Contexto e motivação

O projeto atual (`whisper_hotkey.py`, 2.169 linhas) é uma app de bandeja para macOS Apple Silicon que captura uma hotkey global, grava áudio, transcreve localmente com **MLX + Qwen3-ASR**, opcionalmente refina com um LLM local (`mlx-lm` + Qwen3.5-4B), e cola o resultado no campo focado.

A migração foi motivada por três objetivos do mantenedor:

1. **Distribuição mais simples** — eliminar Python/venv/pip do caminho do usuário final; entregar um binário enxuto.
2. **UI moderna em web stack** — substituir Qt/PySide6 por React.
3. **Cross-platform (macOS Apple Silicon + Windows)** — abrir o app para Windows, mantendo macOS.

Esses objetivos têm uma consequência forte: **MLX é Apple-Silicon-only e Python-only**, então não sobrevive à migração. O motor de ASR muda.

**Requisito firme não-negociável:** 100% local/offline. Nenhuma feature pode depender de rede em runtime (apenas download inicial de modelos).

---

## 2. Decisões arquiteturais

| Eixo | Decisão |
|---|---|
| Framework desktop | **Tauri** (Rust backend + Webview TS/React) |
| Frontend | React + TypeScript + Vite + Tailwind + shadcn/ui + Zustand |
| Backend | Rust (tokio async runtime) |
| ASR | **whisper.cpp** via crate `whisper-rs` (Metal no Mac, CUDA/CPU no Windows) |
| Modelo ASR padrão | OpenAI Whisper `large-v3` quantizado (q5_0, ~1GB) — substituível |
| LLM pós-processamento | `llama.cpp` via crate `llama-cpp-2`, opcional, lazy-load |
| Modelo LLM padrão | Gemma 2 2B Instruct GGUF Q4_K_M (~1.6GB) ou Qwen2.5 3B — usuário escolhe |
| Hotkey global | crate `global-hotkey` |
| Tray | API nativa do Tauri |
| Captura de áudio | crate `cpal` |
| Persistência | JSON em `~/Library/Application Support/whisper-hotkey/` (mac) e `%APPDATA%/whisper-hotkey/` (win) |
| Plataformas v1 | macOS 12+ (Apple Silicon) e Windows 10+ (x64) |
| Estratégia de migração | **Big bang** — Python é aposentado quando TS atinge paridade |

**Abstrações principais (traits Rust):**
- `Transcriber { fn transcribe(&self, samples: &[f32], vocab: &[String]) -> Result<String>; }`
- `PostProcessor { async fn refine(&self, text: &str) -> Result<String>; }`
- `Paster { fn paste(&self, text: &str) -> Result<()>; }`

Essas traits isolam o motor (Whisper hoje, Qwen-ONNX amanhã), o SO (paste é a única coisa platform-specific) e o pós-processamento.

---

## 3. Arquitetura

```
┌─────────────────────────────────────────────────────┐
│  Tauri App (single binary)                          │
│                                                     │
│  ┌──────────────────────┐  ┌─────────────────────┐ │
│  │ Frontend (Webview)   │  │ Backend (Rust)      │ │
│  │ React + TS           │  │                     │ │
│  │ - Settings window    │◄►│ - Global hotkey     │ │
│  │ - Overlay window     │  │ - System tray       │ │
│  │ - History window     │  │ - Audio capture     │ │
│  │ - State (Zustand)    │  │ - whisper-rs (ASR)  │ │
│  │ - Tailwind + shadcn  │  │ - Clipboard + paste │ │
│  └──────────────────────┘  │ - llama.cpp (LLM)   │ │
│           IPC via Tauri    │ - Storage (JSON)    │ │
│           commands/events  │ - Model downloader  │ │
│                            └─────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

**Princípios:**
- Frontend não tem lógica de domínio. Só renderiza estado e dispara intents.
- Backend é única fonte da verdade. Mantém state machine `Idle | Recording | Transcribing`.
- Hardware/SO só é tocado pelo backend Rust.
- Trocar componente do backend não exige mexer no frontend (acoplado por contrato de IPC, não por implementação).

---

## 4. Estrutura de pastas

```
whisper-hotkey-typescript/
├── src/                          # frontend React/TS
│   ├── main.tsx
│   ├── App.tsx
│   ├── windows/
│   │   ├── settings/            # janela de settings (3 abas)
│   │   ├── overlay/             # janela floating de gravação
│   │   └── history/             # janela de histórico
│   ├── components/              # shadcn/ui + componentes próprios
│   ├── store/                   # Zustand stores
│   ├── ipc/                     # wrappers tipados de invoke/listen
│   └── types/                   # tipos compartilhados (gerados)
│
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/
│       ├── main.rs              # bootstrap, tray, registra comandos
│       ├── app_state.rs         # state machine + AppState injetado
│       ├── hotkey.rs            # global hotkey
│       ├── audio.rs             # cpal, captura mono 16kHz
│       ├── asr/
│       │   ├── mod.rs           # trait Transcriber
│       │   ├── whisper_cpp.rs   # impl whisper-rs
│       │   └── vocabulary.rs    # initial_prompt builder
│       ├── llm/
│       │   ├── mod.rs           # trait PostProcessor
│       │   └── llama_cpp.rs     # impl llama-cpp-2
│       ├── paste.rs             # clipboard + Cmd+V / Ctrl+V
│       ├── storage.rs           # config + history (JSON)
│       ├── models.rs            # download + verify SHA + resume
│       ├── replacements.rs      # regras de substituição
│       └── commands.rs          # #[tauri::command]
│
├── docs/superpowers/
│   ├── specs/
│   │   └── 2026-05-12-migration-typescript-design.md  (este arquivo)
│   └── plans/                   # implementation plans
│
├── icons/                       # reaproveitado do projeto atual
├── package.json
├── vite.config.ts
├── tsconfig.json
└── README.md
```

**Limite de tamanho por arquivo:** alvo 150–400 linhas. Acima disso, decompor.

---

## 5. Fluxo de dados (caminho crítico)

```
Usuário pressiona hotkey
    └► hotkey.rs envia intent ao app_state actor (tokio mpsc)
        └► state machine: Idle → Recording
            └► emit("recording-started") → overlay aparece
            └► audio.rs inicia stream cpal (16kHz mono f32)

Usuário pressiona hotkey de novo
    └► state machine: Recording → Transcribing
        └► audio.rs encerra stream, retorna Vec<f32>
        └► emit("transcribing") → overlay muda de cor
        └► spawn_blocking { asr::transcribe(samples, vocab) }
        └► replacements::apply(text)
        └► if config.post_processing { llm::refine(text).timeout(...) }
        └► storage::append_history(text)
        └► paste::paste(text)
        └► emit("transcription-ready", text)
        └► state machine: Transcribing → Idle
            └► overlay esconde
```

**Erros não-fatais (caem no caminho normal com graceful degradation):**
- ASR retorna vazio → não cola, overlay pisca "nada reconhecido", history não grava.
- LLM timeout/erro → usa texto original, log warning.
- Paste falha → texto fica no clipboard, history gravou, modal explica permissão.

**Erros fatais (param o pipeline, emit `error` event):**
- Mic indisponível.
- Modelo não carregado.
- Disco cheio durante gravação.

---

## 6. Modelo de dados

**`config.json`:**
```json
{
  "hotkey": "CmdOrCtrl+Shift+Space",
  "auto_paste": true,
  "overlay_position": "top-center",
  "theme": "system",
  "asr_model": "whisper-large-v3-q5_0",
  "post_processing_enabled": false,
  "llm_model": "gemma-2-2b-it-q4_k_m",
  "llm_timeout_ms": 8000,
  "vocabulary": ["LUCABE", "Ploomes", ...],
  "replacements": [
    { "from": "(?i)\\bmail\\b", "to": "email", "regex": true }
  ]
}
```

**`history.jsonl`** (append-only, uma linha por transcrição):
```json
{"ts":"2026-05-12T20:18:00Z","text":"...","model":"whisper-large-v3-q5_0","post_processed":true}
```

**`models/`** — diretório com `.gguf` baixados + `manifest.json` (nome, sha256, tamanho, versão).

---

## 7. Contratos de IPC

**Commands (frontend → backend):**

| Command | Args | Returns |
|---|---|---|
| `get_config` | — | `Config` |
| `update_config` | `Partial<Config>` | `Config` |
| `get_history` | `{ limit?: number }` | `HistoryEntry[]` |
| `clear_history` | — | `void` |
| `list_available_models` | — | `ModelInfo[]` |
| `download_model` | `{ id: string }` | `void` (progresso via event) |
| `delete_model` | `{ id: string }` | `void` |
| `test_microphone` | — | `{ ok: boolean, devices: string[] }` |
| `register_hotkey` | `{ accelerator: string }` | `{ ok: boolean, error?: string }` |
| `request_permissions` | — | `{ accessibility: bool, microphone: bool }` |
| `open_logs_folder` | — | `void` |

**Events (backend → frontend):**

| Event | Payload |
|---|---|
| `state-changed` | `"idle" \| "recording" \| "transcribing"` |
| `transcription-ready` | `{ text: string, ts: string }` |
| `model-download-progress` | `{ id: string, bytes: number, total: number }` |
| `error` | `{ kind: "mic" \| "model" \| "paste" \| "asr" \| "llm", message: string }` |

Tipos gerados via `ts-rs` no Rust e importados pelo frontend → contratos não divergem.

---

## 8. UI (paridade + melhorias)

**Janelas:**

1. **Overlay** — janela borderless, transparent, always-on-top, ~120×40px. Posição configurável. Mostra estado (gravando: vermelho pulsante; transcrevendo: spinner). Animação suave.
2. **Settings** — janela 800×600, três abas (paridade com Python):
   - **General**: hotkey (capturada visualmente, não só texto), auto-paste, posição do overlay, tema.
   - **Model**: seletor de modelo ASR, toggle de pós-processamento, seletor de modelo LLM, botões de download/remover modelo, indicador de espaço em disco.
   - **Vocabulary**: editor de palavras + editor de regras de substituição (com preview/teste).
3. **History** — janela 500×700, lista virtualizada, busca, copy, delete, export para .txt/.md.

**Melhorias além da paridade Python:**
- Tela de **onboarding** na primeira execução (download de modelos com progresso, pedido de permissões guiado).
- **Status do modelo** sempre visível na tray (✓ pronto / ⌛ carregando / ✗ erro).
- **Testar microfone** no Settings com visualizador de waveform.
- **Teste de regras de substituição** com preview em tempo real.
- **Tema**: respeita o sistema + opção manual claro/escuro.
- **Atalhos no overlay**: clique direito abre menu rápido (cancelar gravação, abrir history).

---

## 9. Permissões

**macOS:**
- **Accessibility** — necessária para hotkey global E para sintetizar Cmd+V. Onboarding abre `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`.
- **Microphone** — pedida via `AVCaptureDevice.requestAccess` na 1ª gravação.

**Windows:**
- **Microphone** — pedido automático pelo SO na 1ª captura (cpal lida).
- Hotkey global e SendInput não requerem permissão explícita.

App detecta permissão negada e mostra modal explicando como conceder, sem travar indefinidamente.

---

## 10. Tratamento de erros

Padrão de erros em Rust: cada módulo define seu `enum Error` com `thiserror`. Tipo de fronteira pro frontend é simples:

```rust
#[derive(Serialize, ts_rs::TS)]
pub struct AppError {
    pub kind: ErrorKind, // enum: Mic, Model, Paste, Asr, Llm, Storage, Hotkey, Permission
    pub message: String, // mensagem amigável já localizada
    pub recoverable: bool,
}
```

**Política:**
- Erros recuperáveis → log + emit `error` event + UI mostra toast/modal.
- Erros fatais (raros: panic em init) → log + tray mostra estado de erro + janela com botão "abrir logs e reportar".
- Toda transcrição é uma transação independente — uma falha não corrompe estado global.

---

## 11. Testes

**Backend Rust:**
- **Unit**: cada módulo com testes para lógica pura (vocabulary builder, replacements engine, state machine, storage I/O). Mocks com traits, sem libs pesadas.
- **Integração**: `tests/` rodando o pipeline ASR→replacements→LLM com um modelo whisper-tiny e um LLM pequeno. Roda em CI quando modelos estão em cache.
- **Smoke (manual)**: scripts em `scripts/smoke/` que rodam o app e exercitam o hotkey via accessibility API.

**Frontend:**
- **Vitest** para stores (Zustand) e helpers IPC. Mock do Tauri `invoke` via `@tauri-apps/api/mocks`.
- **Playwright** para fluxos de UI principais (settings, history, onboarding) com backend mockado.

**Não fazemos:**
- Testes de qualidade de transcrição (eval de WER) — escopo de pesquisa, não de migração.
- Testes E2E de hotkey global em CI (precisa de seat com acessibilidade habilitada). Manual.

---

## 12. Build e distribuição

**macOS:**
- `tauri build` produz `.app` + `.dmg`.
- Code signing com Apple Developer ID + notarization (configurar via env vars no CI).
- Universal binary x86_64 + aarch64? **Não** — só Apple Silicon (consistente com o Python atual). Reduz tamanho.
- Tamanho-alvo do .app: <30MB (sem modelos; modelos baixam pós-instalação).

**Windows:**
- `tauri build` produz `.exe` + instalador NSIS ou MSI.
- Code signing com certificado EV (opcional v1; sem ele o SmartScreen avisa).
- x64 only. Sem ARM por enquanto.

**Modelos**: nunca embarcados no instalador. Onboarding baixa do Hugging Face (URLs no `manifest.json`). Permite trocar modelo sem novo release.

**CI:** GitHub Actions com matrix `[macos-14, windows-2022]`. Cache de Rust target + Whisper.cpp build. Release automático em push de tag.

---

## 13. Plano de milestones (alto nível — detalhe vai no implementation plan)

1. **M0 — Scaffold**: projeto Tauri + Vite + TS + Tailwind/shadcn, CI mínima, tray "Hello".
2. **M1 — Pipeline core**: hotkey global → audio capture → whisper.cpp → clipboard → paste. Sem UI. Funcional via menu da tray.
3. **M2 — Overlay**: janela floating animada com state events.
4. **M3 — Settings (General + Model)**: persistência, troca de hotkey, download de modelo com progresso.
5. **M4 — History**: lista, busca, copy, delete.
6. **M5 — Vocabulary + Replacements**: editor, integração no pipeline ASR.
7. **M6 — LLM pós-processamento**: llama-cpp-2, toggle, timeout, modelo configurável.
8. **M7 — Onboarding + Permissões**: 1ª execução guiada, fluxos de permissão.
9. **M8 — Polish**: tema, animações, ícones, mensagens de erro, logging local rotativo em arquivo (sem envio externo — coerente com "100% offline").
10. **M9 — Distribuição**: code signing mac, instalador Windows, release automatizado.
11. **M10 — Aposentar Python**: deletar `whisper_hotkey.py`, `install.sh`, `launcher.sh`, `build-dmg.sh`, `pkg/`; atualizar README; tag de release.

Cada milestone fecha quando passa nos testes definidos e o autor consegue usar manualmente.

---

## 14. Riscos e mitigações

| Risco | Mitigação |
|---|---|
| `whisper-rs` build complexo no Windows (CUDA opcional) | Começar com build CPU-only no Windows; CUDA fica para milestone separada. |
| Acessibilidade no macOS bloqueia hotkey/paste | Onboarding deep-link para painel de permissões + retry. |
| Tamanho do download de modelos assusta usuário | Oferecer `whisper-base` (~150MB) como opção rápida no onboarding; `large-v3` é opt-in. |
| Qualidade do Whisper < Qwen3-ASR em PT-BR | Validar empiricamente em M1 com áudios de teste; se cair muito, considerar sherpa-onnx + Paraformer como alternativa. |
| `global-hotkey` crate sem suporte completo a algumas combos no macOS | Validar combos comuns na fase de scaffold; ter fallback documentado. |
| Code signing/notarization quebra release | Setup em M9 separado, não no caminho crítico das features. |

---

## 15. Fora de escopo (não-objetivos)

- Linux.
- iOS / Android.
- Tradução em tempo real (ASR de uma língua → texto em outra).
- Cloud sync de history/config.
- Diarization (separar falantes).
- Streaming/parcial transcription enquanto fala.
- Comandos de voz ("apaga isso", "nova linha") — escopo separado.
- Manter Python funcional após M10.

---

## 16. Próximo passo

Esta spec é o input para o **implementation plan** (gerado pelo skill `superpowers:writing-plans`), que vai detalhar cada milestone em tarefas executáveis com critérios de verificação.
