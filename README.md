<p align="center">
  <img src="icons/whisper.png" alt="Whisper Hotkey Icon" width="128"/>
</p>

<h1 align="center">Whisper Hotkey for macOS</h1>

<p align="center">
Ditado por voz em qualquer app do Mac. Pressione uma hotkey, fale, pressione novamente — o texto aparece onde o cursor estiver.
</p>

<p align="center">
Sem nuvem. Sem API key. Sem assinatura. Tudo roda localmente no seu Mac com Apple Silicon.
</p>

<p align="center">
  <em>Baseado no <a href="https://github.com/dpejoh/whisper-hotkey">whisper-hotkey</a> de <a href="https://github.com/dpejoh">dpejoh</a> (versao Windows). Este fork foi reescrito do zero para macOS com Apple Silicon, usando MLX para aceleracao via GPU.</em>
</p>

---

## Funcionalidades

- **Hotkey global** — funciona em qualquer app (Pages, VS Code, WhatsApp, Slack, etc.)
- **Transcricao local** — usa [Qwen3-ASR](https://huggingface.co/Qwen/Qwen3-ASR-0.6B) via [MLX](https://github.com/ml-explore/mlx) no Apple Silicon GPU
- **Pos-processamento com LLM** (opcional) — corrige pontuacao, acentuacao e formatacao usando [Qwen3.5-4B](https://huggingface.co/mlx-community/Qwen3.5-4B-MLX-4bit)
- **Overlay de gravacao** — indicador visual discreto na tela
- **Historico de transcricoes** — acesse textos anteriores pela bandeja do sistema
- **Vocabulario personalizado** — adicione palavras que o modelo deve reconhecer
- **Substituicoes automaticas** — defina regras de substituicao de texto
- **Temas claro/escuro** — segue o tema do sistema ou escolha manualmente
- **Cola automaticamente** — o texto transcrito e colado direto no campo ativo

---

## Requisitos

- **Mac com Apple Silicon** (M1, M2, M3, M4 ou superior)
- **macOS 12.0** (Monterey) ou superior
- **~5GB de espaco livre** (modelos de IA + dependencias)
- **Conexao com a internet** (apenas para a instalacao)

---

## Instalacao

### Opcao 1: Instalador .dmg (recomendado)

1. Baixe o arquivo `WhisperHotkey.dmg` da pagina de [Releases](../../releases)
2. Abra o `.dmg` e clique duas vezes no `WhisperHotkey.pkg`
3. Siga o assistente de instalacao
4. Uma janela do Terminal vai abrir automaticamente para baixar as dependencias e os modelos de IA (~3.7GB). Aguarde a conclusao.
5. Abra o app pelo **Spotlight** (Cmd+Space) ou pela pasta **Applications**

### Opcao 2: Script de instalacao

```bash
git clone https://github.com/lffelgueiras/whisper-hotkey.git
cd whisper-hotkey
bash install.sh
```

O script instala tudo automaticamente:
- Homebrew (se nao tiver)
- Python 3 + venv
- PortAudio
- Pacotes Python (MLX, PySide6, PyObjC, etc.)
- Modelo de transcricao Qwen3-ASR-0.6B (~1.2GB)
- Modelo de pos-processamento Qwen3.5-4B (~2.5GB, opcional)
- App bundle em `/Applications/Whisper Hotkey.app`

### Opcao 3: Manual

```bash
git clone https://github.com/lffelgueiras/whisper-hotkey.git
cd whisper-hotkey

python3 -m venv venv
source venv/bin/activate

pip install numpy sounddevice pyperclip PySide6 \
    pyobjc-framework-Cocoa pyobjc-framework-Quartz \
    mlx-qwen3-asr mlx-lm transformers huggingface-hub

python whisper_hotkey.py
```

---

## Como usar

1. Abra o **Whisper Hotkey** (aparece um icone na bandeja do sistema)
2. Clique em qualquer campo de texto
3. Pressione **Cmd+Shift+Space** (hotkey padrao)
4. Fale
5. Pressione a hotkey novamente
6. O texto transcrito e colado automaticamente

Um overlay aparece no topo da tela indicando o estado (gravando / transcrevendo).

---

## Configuracoes

Clique no icone da bandeja do sistema > **Settings**

| Pagina | Opcoes |
|--------|--------|
| **General** | Hotkey, auto-paste, posicao do overlay, tema |
| **Model** | Modelo ASR, toggle de pos-processamento, modelo LLM |
| **Vocabulary** | Palavras personalizadas, regras de substituicao |

### Modelos

| Modelo | Tipo | Tamanho | Descricao |
|--------|------|---------|-----------|
| `Qwen/Qwen3-ASR-0.6B` | ASR | ~1.2GB | Transcricao de voz (padrao) |
| `mlx-community/Qwen3.5-4B-MLX-4bit` | LLM | ~2.5GB | Pos-processamento de texto (opcional) |

O pos-processamento com LLM corrige pontuacao, acentuacao e formatacao. Recomendado para Macs com 16GB+ de RAM.

---

## Permissoes

Na primeira execucao, o macOS vai pedir duas permissoes:

- **Acessibilidade** — necessaria para capturar a hotkey global e colar texto
- **Microfone** — necessaria para gravar audio

Aceite ambas em **Preferencias do Sistema > Privacidade e Seguranca**.

---

## Estrutura do projeto

```
whisper-hotkey/
├── whisper_hotkey.py    # Aplicacao principal (UI, hotkey, gravacao, transcricao)
├── launcher.sh          # Launcher do .app bundle
├── install.sh           # Script de instalacao completo
├── build-dmg.sh         # Script para gerar o .dmg
├── pkg/
│   ├── postinstall      # Script pos-instalacao do .pkg
│   └── distribution.xml # Configuracao do wizard do instalador
└── icons/               # Icones do app
```

---

## Solucao de problemas

**Hotkey nao funciona** — Verifique se o app tem permissao de Acessibilidade em Preferencias do Sistema > Privacidade e Seguranca > Acessibilidade.

**"Nada reconhecido"** — O audio pode estar muito curto ou silencioso. Fale por pelo menos 1 segundo.

**App nao aparece na bandeja** — Verifique se o Python encontrou todas as dependencias. Rode `~/.whisper-hotkey/venv/bin/python3 ~/.whisper-hotkey/whisper_hotkey.py` no Terminal para ver erros.

**Lento na primeira transcricao** — Normal. O modelo e carregado na memoria na primeira vez. As transcricoes seguintes sao rapidas.

---

## Creditos

Este projeto e um fork/reescrita do [whisper-hotkey](https://github.com/dpejoh/whisper-hotkey) criado por [dpejoh](https://github.com/dpejoh), originalmente feito para Windows com `faster-whisper`. Esta versao foi reescrita do zero para **macOS com Apple Silicon**, usando [MLX](https://github.com/ml-explore/mlx) para aceleracao via GPU.

---

## Licenca

MIT
