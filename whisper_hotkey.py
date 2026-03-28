#!/usr/bin/env python3
"""
Whisper Hotkey — macOS Edition
────────────────────────────────
System tray app with floating recording indicator, transcription history, and settings.

Press your configured hotkey to start recording. Press it again to stop and transcribe.
The result is copied to clipboard and pasted into the active text field.

Requirements:
    pip install mlx-qwen3-asr mlx-lm sounddevice scipy pyperclip PySide6
"""

import os, sys, json, math, threading, datetime, subprocess
from pathlib import Path
from dataclasses import dataclass, asdict

import numpy as np
import sounddevice as sd
import pyperclip

import Quartz
from AppKit import NSWorkspace

# ── macOS text insertion (clipboard + Cmd+V via AppleScript) ─────────────────

def _type_text_direct(text: str) -> None:
    """
    Insert *text* into the currently focused field by copying to clipboard
    and simulating Cmd+V via AppleScript.
    """
    pyperclip.copy(text)
    subprocess.run([
        "osascript", "-e",
        'tell application "System Events" to keystroke "v" using command down'
    ], check=False)

# ─────────────────────────────────────────────────────────────────────────────

from PySide6.QtWidgets import (
    QApplication, QWidget, QDialog, QSystemTrayIcon, QMenu,
    QVBoxLayout, QHBoxLayout, QFormLayout, QLabel, QComboBox,
    QPushButton, QLineEdit, QScrollArea, QFrame, QSpinBox, QFileDialog,
    QLayout, QSizePolicy,
)
from PySide6.QtCore import (
    Qt, QThread, Signal, QTimer, QPropertyAnimation,
    QEasingCurve, QObject, QRect,
)
from PySide6.QtGui import (
    QIcon, QPixmap, QPainter, QColor, QBrush, QPen, QFont, QPainterPath, QAction,
)

# ──────────────────────────────────────────────────────────────────────────────
# THEME
# ──────────────────────────────────────────────────────────────────────────────


def _macos_is_light() -> bool:
    """Check macOS appearance. Returns True for light mode."""
    try:
        result = subprocess.run(
            ["defaults", "read", "-g", "AppleInterfaceStyle"],
            capture_output=True, text=True, check=False
        )
        return result.returncode != 0  # non-zero means no Dark mode key → light
    except Exception:
        return False


def get_style(is_dark: bool) -> str:
    _FONT = "'Helvetica Neue', 'Helvetica', sans-serif"
    _MONO = "'Menlo', monospace"

    if is_dark:
        bg       = "#1e1e1e"
        card_bg  = "rgba(255, 255, 255, 10)"
        card_brd = "rgba(255, 255, 255, 15)"
        card_hov = "rgba(255, 255, 255, 20)"
        inp_bg   = "rgba(255, 255, 255, 10)"
        inp_hov  = "rgba(255, 255, 255, 14)"
        border   = "rgba(255, 255, 255, 15)"
        border2  = "rgba(255, 255, 255, 20)"
        text     = "rgba(255, 255, 255, 217)"
        muted2   = "rgba(255, 255, 255, 102)"
        muted    = "rgba(255, 255, 255, 64)"
        sec_col  = "rgba(255, 255, 255, 64)"
        btn_bg   = "rgba(255, 255, 255, 10)"
        btn_hov  = "rgba(255, 255, 255, 15)"
        btn_pre  = "rgba(255, 255, 255, 6)"
        pri_bg   = "rgba(10, 132, 255, 0.85)"
        pri_hov  = "rgba(10, 132, 255, 1.0)"
        pri_pre  = "rgba(10, 132, 255, 0.7)"
        pri_txt  = "#FFFFFF"
        focus_b  = "#0a84ff"
        ghost_brd     = "rgba(255, 255, 255, 15)"
        ghost_hov_brd = "rgba(255, 255, 255, 25)"
        danger_border = "rgba(255, 69, 58, 0.3)"
        danger_txt    = "#ff453a"
        danger_hov    = "rgba(255, 69, 58, 0.1)"
        danger_hbrd   = "rgba(255, 69, 58, 0.5)"
        sep_col  = "rgba(255, 255, 255, 15)"
        scr_hdl  = "rgba(255, 255, 255, 38)"
        menu_bg  = "#2c2c2c"
        menu_sel = "rgba(255, 255, 255, 15)"
        menu_sep = "rgba(255, 255, 255, 15)"
        sidebar_bg = "rgba(255, 255, 255, 5)"
        sidebar_sel = "rgba(10, 132, 255, 0.15)"
    else:
        bg       = "#f0f0f0"
        card_bg  = "rgba(255, 255, 255, 200)"
        card_brd = "rgba(0, 0, 0, 12)"
        card_hov = "rgba(0, 0, 0, 18)"
        inp_bg   = "rgba(255, 255, 255, 220)"
        inp_hov  = "rgba(255, 255, 255, 255)"
        border   = "rgba(0, 0, 0, 12)"
        border2  = "rgba(0, 0, 0, 18)"
        text     = "rgba(0, 0, 0, 217)"
        muted2   = "rgba(0, 0, 0, 102)"
        muted    = "rgba(0, 0, 0, 64)"
        sec_col  = "rgba(0, 0, 0, 64)"
        btn_bg   = "rgba(0, 0, 0, 6)"
        btn_hov  = "rgba(0, 0, 0, 10)"
        btn_pre  = "rgba(0, 0, 0, 14)"
        pri_bg   = "rgba(0, 122, 255, 0.9)"
        pri_hov  = "rgba(0, 122, 255, 1.0)"
        pri_pre  = "rgba(0, 122, 255, 0.75)"
        pri_txt  = "#FFFFFF"
        focus_b  = "#007aff"
        ghost_brd     = "rgba(0, 0, 0, 12)"
        ghost_hov_brd = "rgba(0, 0, 0, 22)"
        danger_border = "rgba(255, 59, 48, 0.25)"
        danger_txt    = "#ff3b30"
        danger_hov    = "rgba(255, 59, 48, 0.08)"
        danger_hbrd   = "rgba(255, 59, 48, 0.4)"
        sep_col  = "rgba(0, 0, 0, 12)"
        scr_hdl  = "rgba(0, 0, 0, 38)"
        menu_bg  = "#FFFFFF"
        menu_sel = "rgba(0, 0, 0, 8)"
        menu_sep = "rgba(0, 0, 0, 12)"
        sidebar_bg = "rgba(0, 0, 0, 4)"
        sidebar_sel = "rgba(0, 122, 255, 0.1)"

    return f"""
QWidget {{
    background-color: {bg};
    color: {text};
    font-family: {_FONT};
    font-size: 13px;
}}
QDialog  {{ background-color: {bg}; }}
QLabel   {{ background: transparent; }}

QFrame#card {{
    background-color: {card_bg};
    border: 1px solid {card_brd};
    border-radius: 10px;
}}
QFrame#card:hover {{ border-color: {card_hov}; }}

QFrame#separator {{
    background-color: {sep_col};
    max-height: 1px;
    border: none;
}}

QLabel#section_label {{
    color: {sec_col};
    font-size: 10px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    padding-left: 2px;
}}
QLabel#muted      {{ color: {muted};  font-size: 11px; font-family: {_MONO}; }}
QLabel#body_text  {{ color: {muted2}; font-size: 13px; }}
QLabel#empty_hint {{ color: {muted};  font-size: 13px; padding: 48px 0; }}

QComboBox {{
    background-color: {inp_bg};
    border: 1px solid {border};
    border-radius: 6px;
    padding: 4px 28px 4px 10px;
    color: {text};
    min-height: 28px;
}}
QComboBox:hover {{ background-color: {inp_hov}; border-color: {border2}; }}
QComboBox:focus {{ border-color: {focus_b}; }}
QComboBox::drop-down {{ border: none; width: 24px; }}
QComboBox::down-arrow {{ image: none; width: 0; }}
QComboBox QAbstractItemView {{
    background-color: {menu_bg};
    border: 1px solid {card_brd};
    border-radius: 8px;
    color: {text};
    selection-background-color: {menu_sel};
    selection-color: {text};
    padding: 4px 0;
    outline: none;
}}
QComboBox QAbstractItemView::item {{
    padding: 6px 12px;
    border-radius: 4px;
    margin: 1px 4px;
}}

QLineEdit {{
    background-color: {inp_bg};
    border: 1px solid {border};
    border-radius: 6px;
    padding: 4px 10px;
    color: {text};
    min-height: 28px;
}}
QLineEdit:hover {{ background-color: {inp_hov}; border-color: {border2}; }}
QLineEdit:focus {{ border-color: {focus_b}; }}

QPushButton {{
    background-color: {btn_bg};
    border: 1px solid {border};
    border-radius: 6px;
    padding: 5px 16px;
    color: {text};
    min-height: 28px;
    font-size: 13px;
}}
QPushButton:hover   {{ background-color: {btn_hov}; }}
QPushButton:pressed {{ background-color: {btn_pre}; }}

QPushButton#primary {{
    background-color: {pri_bg};
    border: none;
    color: {pri_txt};
    font-weight: 500;
    border-radius: 6px;
}}
QPushButton#primary:hover   {{ background-color: {pri_hov}; }}
QPushButton#primary:pressed {{ background-color: {pri_pre}; }}

QPushButton#ghost {{
    background: transparent;
    border: 1px solid {ghost_brd};
    color: {muted2};
    font-size: 12px;
    min-height: 28px;
    padding: 4px 12px;
    border-radius: 6px;
}}
QPushButton#ghost:hover  {{ border-color: {ghost_hov_brd}; background: {btn_hov}; }}

QPushButton#danger {{
    background: transparent;
    border: 1px solid {danger_border};
    color: {danger_txt};
    font-size: 12px;
    min-height: 28px;
    padding: 4px 12px;
    border-radius: 6px;
}}
QPushButton#danger:hover {{ background-color: {danger_hov}; border-color: {danger_hbrd}; }}

QScrollArea {{ background: transparent; border: none; }}
QScrollBar:vertical {{
    background: transparent;
    width: 6px;
    border-radius: 3px;
    margin: 0;
}}
QScrollBar::handle:vertical {{
    background: {scr_hdl};
    border-radius: 3px;
    min-height: 32px;
}}
QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {{ height: 0; border: none; }}
QScrollBar::add-page:vertical,  QScrollBar::sub-page:vertical {{ background: none; }}

QSpinBox {{
    background-color: {inp_bg};
    border: 1px solid {border};
    border-radius: 6px;
    padding: 4px 10px;
    color: {text};
    min-height: 28px;
}}
QSpinBox:hover {{ border-color: {border2}; }}
QSpinBox:focus {{ border-color: {focus_b}; }}
QSpinBox::up-button, QSpinBox::down-button {{ width: 0; border: none; }}

QMenu {{
    background-color: {menu_bg};
    border: 1px solid {card_brd};
    border-radius: 8px;
    color: {text};
    padding: 4px 0;
}}
QMenu::item          {{ padding: 6px 16px 6px 12px; font-size: 13px; border-radius: 4px; margin: 1px 4px; }}
QMenu::item:selected {{ background-color: {menu_sel}; }}
QMenu::separator     {{ height: 1px; background: {menu_sep}; margin: 3px 0; }}

/* Sidebar for settings */
QFrame#sidebar {{
    background-color: {sidebar_bg};
    border-right: 1px solid {sep_col};
}}
QPushButton#sidebar_item {{
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 7px 12px;
    text-align: left;
    color: {muted2};
    font-size: 12px;
    min-height: 20px;
}}
QPushButton#sidebar_item:hover {{ background: {btn_hov}; }}
QPushButton#sidebar_active {{
    background: {sidebar_sel};
    border: none;
    border-radius: 6px;
    padding: 7px 12px;
    text-align: left;
    color: {focus_b};
    font-size: 12px;
    font-weight: 500;
    min-height: 20px;
}}
"""

# ──────────────────────────────────────────────────────────────────────────────
# CONFIG
# ──────────────────────────────────────────────────────────────────────────────

_CFG_PATH = Path.home() / ".whisper_hotkey" / "config.json"

@dataclass
class Config:
    model:            str  = "Qwen/Qwen3-ASR-0.6B"
    language:         str  = "auto"
    hotkey:           str  = "cmd+shift+space"
    auto_paste:       bool = True
    post_process:     bool = True
    post_process_model: str = "mlx-community/Qwen3.5-4B-MLX-4bit"
    overlay_position: str  = "top-center"
    history_limit:    int  = 50
    theme:            str  = "auto"
    vocabulary:       list = None  # words the model should recognize
    replacements:     list = None  # [{"from": "X", "to": "Y"}, ...]

    def __post_init__(self):
        if self.vocabulary is None:
            self.vocabulary = []
        if self.replacements is None:
            self.replacements = []

    @classmethod
    def load(cls):
        if _CFG_PATH.exists():
            try:
                d = json.loads(_CFG_PATH.read_text())
                return cls(**{k: v for k, v in d.items() if k in cls.__dataclass_fields__})
            except Exception:
                pass
        return cls()

    def save(self):
        _CFG_PATH.parent.mkdir(parents=True, exist_ok=True)
        _CFG_PATH.write_text(json.dumps(asdict(self), indent=2))

# ──────────────────────────────────────────────────────────────────────────────
# HISTORY
# ──────────────────────────────────────────────────────────────────────────────

class History:
    _path = Path.home() / ".whisper_hotkey" / "history.json"

    def __init__(self, limit: int = 50):
        self.limit   = limit
        self.entries: list[dict] = []
        self._load()

    def _load(self):
        if self._path.exists():
            try: self.entries = json.loads(self._path.read_text())
            except Exception: self.entries = []

    def add(self, text: str):
        self.entries.insert(0, {
            "text": text,
            "ts":   datetime.datetime.now().isoformat(timespec="seconds"),
        })
        self.entries = self.entries[:self.limit]
        self._save()

    def _save(self):
        self._path.parent.mkdir(parents=True, exist_ok=True)
        tmp = self._path.with_suffix(".tmp")
        tmp.write_text(json.dumps(self.entries, indent=2))
        tmp.replace(self._path)

    def clear(self):
        self.entries = []
        self._save()

# ──────────────────────────────────────────────────────────────────────────────
# WORKERS
# ──────────────────────────────────────────────────────────────────────────────

class ModelLoader(QThread):
    loaded = Signal(object)
    failed = Signal(str)
    status = Signal(str)

    def __init__(self, cfg: Config):
        super().__init__()
        self.cfg = cfg

    def run(self):
        try:
            self.status.emit(f"Loading Qwen3-ASR '{self.cfg.model}' (MLX)...")
            from mlx_qwen3_asr import load_model
            model, _config = load_model(self.cfg.model)
            self.loaded.emit(model)
        except Exception as e:
            self.failed.emit(str(e))


class PostProcessor:
    """Lazy-loaded LLM for text post-processing via mlx-lm."""
    _instance = None
    _model = None
    _tokenizer = None

    _SYSTEM_BASE = (
        "You are a text correction assistant for speech transcriptions. "
        "Fix punctuation, capitalization, and formatting. "
        "Do not add, remove, or change the meaning of words. "
        "Output ONLY the corrected text, nothing else."
    )

    @classmethod
    def get(cls, model_name: str):
        if cls._instance is None or cls._model_name != model_name:
            from mlx_lm import load
            cls._model, cls._tokenizer = load(model_name)
            cls._model_name = model_name
            cls._instance = cls()
        return cls._instance

    def process(self, text: str, vocabulary: list = None) -> str:
        from mlx_lm import generate
        system = self._SYSTEM_BASE
        if vocabulary:
            words = ", ".join(vocabulary)
            system += (
                f"\n\nIMPORTANT: The following proper nouns and terms must be "
                f"spelled exactly as shown: {words}"
            )
        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": text},
        ]
        prompt = self._tokenizer.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True,
            enable_thinking=False,
        )
        result = generate(self._model, self._tokenizer, prompt=prompt, max_tokens=len(text) * 3)
        return result.strip()


class TranscribeWorker(QObject):
    """Runs transcription in a Python thread (avoids QThread GC crashes on Python 3.14)."""
    finished = Signal(str)
    failed   = Signal(str)
    _SR = 16000

    def __init__(self, model, audio: np.ndarray, language: str,
                 post_process: bool = False, post_process_model: str = "",
                 vocabulary: list = None, replacements: list = None):
        super().__init__()
        self.model    = model
        self.audio    = audio
        self.language = None if language == "auto" else language
        self._post_process = post_process
        self._pp_model = post_process_model
        self._vocabulary = vocabulary or []
        self._replacements = replacements or []

    def start(self):
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()

    def _run(self):
        try:
            # Skip transcription if audio is too short or silent
            rms = float(np.sqrt(np.mean(self.audio ** 2)))
            if len(self.audio) < self._SR * 0.3 or rms < 0.0005:
                self.finished.emit("")
                return

            vocab_context = " ".join(self._vocabulary) if self._vocabulary else ""

            from mlx_qwen3_asr import transcribe
            result = transcribe(
                (self.audio, self._SR),
                model=self.model,
                language=self.language,
                context=vocab_context,
            )
            text = result.text.strip()

            if self._post_process and text and self._pp_model:
                pp = PostProcessor.get(self._pp_model)
                text = pp.process(text, self._vocabulary)

            # Apply replacements
            for r in self._replacements:
                fr, to = r.get("from", ""), r.get("to", "")
                if fr:
                    text = text.replace(fr, to)

            self.finished.emit(text)
        except Exception as e:
            self.failed.emit(str(e))

# ──────────────────────────────────────────────────────────────────────────────
# TRAY ICON
# ──────────────────────────────────────────────────────────────────────────────

_ICON_DIR = Path(__file__).resolve().parent / "icons"

_ICON_FILES: dict[str, list[str]] = {
    "idle":      ["whisper-transparent.png", "whisper-transparent.svg"],
    "recording": ["whisper-transparent-red.png", "whisper-transparent-red.svg"],
    "loading":   ["whisper-transparent-blue.png", "whisper-transparent-blue.svg"],
}


def _make_icon(state: str) -> QIcon:
    for name in _ICON_FILES.get(state, []):
        path = _ICON_DIR / name
        if path.exists():
            return QIcon(str(path))

    _fallback_colours = {
        "idle":      QColor(50,  50,  50),
        "recording": QColor(196, 43,  28),
        "loading":   QColor(0,   120, 212),
    }
    sz  = 64
    px  = QPixmap(sz, sz)
    px.fill(Qt.GlobalColor.transparent)
    p   = QPainter(px)
    p.setRenderHint(QPainter.RenderHint.Antialiasing)
    mid = sz // 2
    p.setPen(Qt.PenStyle.NoPen)
    p.setBrush(QBrush(_fallback_colours.get(state, QColor(50, 50, 50))))
    p.drawEllipse(1, 1, sz - 2, sz - 2)
    p.setBrush(QBrush(QColor(235, 235, 235)))
    p.drawRoundedRect(mid - 9, 10, 18, 27, 9, 9)
    pen = QPen(QColor(235, 235, 235), 3.5)
    pen.setCapStyle(Qt.PenCapStyle.RoundCap)
    p.setPen(pen)
    p.setBrush(Qt.BrushStyle.NoBrush)
    p.drawArc(mid - 13, 29, 26, 16, 0, -180 * 16)
    p.drawLine(mid, 45, mid, 53)
    p.drawLine(mid - 8, 53, mid + 8, 53)
    p.end()
    return QIcon(px)


def _app_icon() -> QIcon:
    for name in ("whisper.svg", "whisper.png"):
        path = _ICON_DIR / name
        if path.exists():
            return QIcon(str(path))
    return _make_icon("idle")

# ──────────────────────────────────────────────────────────────────────────────
# TOGGLE SWITCH
# ──────────────────────────────────────────────────────────────────────────────

class ToggleSwitch(QWidget):
    toggled = Signal(bool)

    def __init__(self, checked: bool = False):
        super().__init__()
        self._on = checked
        self.setFixedSize(42, 22)
        self.setCursor(Qt.CursorShape.PointingHandCursor)

    @property
    def checked(self): return self._on

    @checked.setter
    def checked(self, v):
        self._on = bool(v)
        self.update()

    def mousePressEvent(self, _):
        self._on = not self._on
        self.toggled.emit(self._on)
        self.update()

    def keyPressEvent(self, event):
        if event.key() in (Qt.Key.Key_Space, Qt.Key.Key_Return):
            self._on = not self._on
            self.toggled.emit(self._on)
            self.update()
        else:
            super().keyPressEvent(event)

    def paintEvent(self, _):
        p = QPainter(self)
        p.setRenderHint(QPainter.RenderHint.Antialiasing)
        w, h, r = self.width(), self.height(), self.height() / 2

        tpath = QPainterPath()
        tpath.addRoundedRect(0, 0, w, h, r, r)
        if self._on:
            p.setPen(Qt.PenStyle.NoPen)
            p.setBrush(QBrush(QColor(52, 199, 89)))       # macOS green #34c759
        else:
            p.setPen(QPen(QColor(255, 255, 255, 60), 1.2))
            p.setBrush(QBrush(QColor(255, 255, 255, 25)))
        p.drawPath(tpath)

        p.setPen(Qt.PenStyle.NoPen)
        m  = 3
        tx = w - h + m if self._on else m
        p.setBrush(QBrush(QColor(255, 255, 255) if self._on else QColor(180, 180, 180)))
        p.drawEllipse(int(tx), m, h - m * 2, h - m * 2)
        p.end()

# ──────────────────────────────────────────────────────────────────────────────
# RECORDING OVERLAY
# ──────────────────────────────────────────────────────────────────────────────

_FONT_UI   = "Helvetica Neue"
_FONT_MONO = "Menlo"

class _FlowLayout(QLayout):
    """Layout that arranges widgets in a horizontal flow, wrapping to new lines."""

    def __init__(self, parent=None, spacing=6):
        super().__init__(parent)
        self._items = []
        self._spacing = spacing

    def addItem(self, item):
        self._items.append(item)

    def count(self):
        return len(self._items)

    def itemAt(self, index):
        if 0 <= index < len(self._items):
            return self._items[index]
        return None

    def takeAt(self, index):
        if 0 <= index < len(self._items):
            return self._items.pop(index)
        return None

    def sizeHint(self):
        return self.minimumSize()

    def minimumSize(self):
        from PySide6.QtCore import QSize
        s = QSize(0, 0)
        for item in self._items:
            s = s.expandedTo(item.minimumSize())
        return s

    def setGeometry(self, rect):
        super().setGeometry(rect)
        self._do_layout(rect)

    def _do_layout(self, rect):
        x = rect.x()
        y = rect.y()
        line_h = 0
        for item in self._items:
            w = item.sizeHint().width()
            h = item.sizeHint().height()
            if x + w > rect.right() + 1 and line_h > 0:
                x = rect.x()
                y += line_h + self._spacing
                line_h = 0
            item.setGeometry(QRect(x, y, w, h))
            x += w + self._spacing
            line_h = max(line_h, h)


class RecordingOverlay(QWidget):
    """Floating card that shows recording/transcription state."""

    def __init__(self, position: str = "top-center"):
        super().__init__()
        self.position = position
        self.state    = "idle"
        self.elapsed  = 0
        self.preview  = ""
        self._phase   = 0.0
        self.is_dark  = True
        self._restore_app = None

        self.setWindowFlags(
            Qt.WindowType.FramelessWindowHint |
            Qt.WindowType.WindowStaysOnTopHint |
            Qt.WindowType.BypassWindowManagerHint
        )
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setAttribute(Qt.WidgetAttribute.WA_ShowWithoutActivating)
        self.setFixedSize(340, 76)

        self._tick  = QTimer()
        self._tick.setInterval(1000)
        self._tick.timeout.connect(self._on_tick)

        self._anim  = QTimer()
        self._anim.setInterval(48)
        self._anim.timeout.connect(self._on_anim)

        self._hide  = QTimer()
        self._hide.setSingleShot(True)
        self._hide.timeout.connect(self._fade_out)

        self._fade  = QPropertyAnimation(self, b"windowOpacity")
        self._fade.setDuration(380)
        self._fade.setEasingCurve(QEasingCurve.Type.InCubic)
        self._fade.finished.connect(self.hide)

    def _on_tick(self):
        self.elapsed += 1
        self.update()

    def _on_anim(self):
        self._phase += 0.1
        self.update()

    def _place(self):
        g = QApplication.primaryScreen().availableGeometry()
        pos_map = {
            "top-center":    (g.center().x() - self.width() // 2, g.top() + 20),
            "top-right":     (g.right() - self.width() - 20,      g.top() + 20),
            "top-left":      (g.left() + 20,                       g.top() + 20),
            "bottom-center": (g.center().x() - self.width() // 2,  g.bottom() - self.height() - 58),
        }
        x, y = pos_map.get(self.position, pos_map["top-center"])
        self.move(x, y)

    def _pin_above_fullscreen(self):
        """Make overlay visible above fullscreen apps via native macOS API."""
        try:
            from Cocoa import NSApp
            wid = int(self.winId())
            for w in NSApp.windows():
                if w.windowNumber() == wid:
                    # kCGMaximumWindowLevel — above everything including fullscreen
                    w.setLevel_((1 << 30) - 1)
                    # canJoinAllSpaces(1<<0) | fullScreenAuxiliary(1<<8) | canJoinAllApplications(1<<5)
                    w.setCollectionBehavior_((1 << 0) | (1 << 8) | (1 << 5))
                    w.orderFrontRegardless()
                    break
        except Exception:
            pass

    def show_recording(self):
        self._hide.stop(); self._fade.stop(); self.setWindowOpacity(1.0)
        self.state, self.elapsed, self._phase = "recording", 0, 0.0
        self._tick.start(); self._anim.start()
        self._place(); self.show(); self.update()
        self._pin_above_fullscreen()

    def show_transcribing(self):
        self._tick.stop()
        self._anim.stop()
        self.state = "transcribing"
        self.update()
        self._anim.start()

    def show_done(self, text: str):
        self._anim.stop()
        self.state   = "done"
        self.preview = (text[:38] + "...") if len(text) > 38 else text
        self.update()
        self._hide.start(2600)

    def show_error(self, msg: str):
        self._tick.stop(); self._anim.stop()
        self.state   = "error"
        self.preview = msg[:42]
        self.update()
        self._hide.start(3500)

    def _fade_out(self):
        self._fade.setStartValue(1.0)
        self._fade.setEndValue(0.0)
        self._fade.start()

    def paintEvent(self, _):
        p = QPainter(self)
        p.setRenderHint(QPainter.RenderHint.Antialiasing)
        W, H = self.width(), self.height()
        M = 8  # margin for shadow + padding

        # Drop shadow
        sh = QPainterPath()
        sh.addRoundedRect(M, M + 2, W - M * 2, H - M * 2, 14, 14)
        p.setPen(Qt.PenStyle.NoPen)
        p.setBrush(QColor(0, 0, 0, 50))
        p.drawPath(sh)

        # Card background
        bg = QPainterPath()
        bg.addRoundedRect(M, M, W - M * 2, H - M * 2, 16, 16)
        p.setBrush(QColor(30, 30, 30, 235) if self.is_dark else QColor(245, 245, 245, 245))
        p.setPen(QPen(QColor(255, 255, 255, 20) if self.is_dark else QColor(0, 0, 0, 15), 1))
        p.drawPath(bg)

        # Content area
        cx, cy = M + 24, H // 2
        icon_r = 18  # icon circle radius
        text_x = cx + icon_r + 14

        # State colors
        colors = {
            "recording":    (QColor(255, 69, 58),  QColor(255, 69, 58, 38)),     # red
            "transcribing": (QColor(10, 132, 255), QColor(10, 132, 255, 30)),    # blue
            "done":         (QColor(52, 199, 89),  QColor(52, 199, 89, 30)),     # green
            "error":        (QColor(255, 69, 58),  QColor(255, 69, 58, 30)),     # red
        }
        accent, accent_bg = colors.get(self.state, (QColor(128, 128, 128), QColor(128, 128, 128, 30)))

        # Circle background
        p.setPen(QPen(accent.lighter(120), 1.5))
        p.setBrush(accent_bg)
        p.drawEllipse(cx - icon_r, cy - icon_r, icon_r * 2, icon_r * 2)

        # Icon inside circle
        p.setPen(Qt.PenStyle.NoPen)
        if self.state == "recording":
            pulse = 0.6 + 0.4 * math.sin(self._phase)
            p.setBrush(QColor(255, 69, 58, int(255 * pulse)))
            p.drawEllipse(cx - 6, cy - 6, 12, 12)
        elif self.state == "transcribing":
            pen = QPen(QColor(10, 132, 255), 2.5)
            pen.setCapStyle(Qt.PenCapStyle.RoundCap)
            p.setPen(pen)
            p.setBrush(Qt.BrushStyle.NoBrush)
            span = 270
            start = int((-self._phase * 180 / math.pi) * 16) % (360 * 16)
            p.drawArc(cx - 8, cy - 8, 16, 16, start, span * 16)
        elif self.state == "done":
            pen = QPen(QColor(52, 199, 89), 2.5)
            pen.setCapStyle(Qt.PenCapStyle.RoundCap)
            pen.setJoinStyle(Qt.PenJoinStyle.RoundJoin)
            p.setPen(pen)
            p.drawLine(cx - 5, cy + 1, cx - 1, cy + 5)
            p.drawLine(cx - 1, cy + 5, cx + 6, cy - 4)
        elif self.state == "error":
            pen = QPen(QColor(255, 69, 58), 2.5)
            pen.setCapStyle(Qt.PenCapStyle.RoundCap)
            p.setPen(pen)
            p.drawLine(cx - 5, cy - 5, cx + 5, cy + 5)
            p.drawLine(cx + 5, cy - 5, cx - 5, cy + 5)

        # Text
        txt_color = QColor(255, 255, 255, 217) if self.is_dark else QColor(0, 0, 0, 217)
        sub_color = QColor(255, 255, 255, 90) if self.is_dark else QColor(0, 0, 0, 90)

        if self.state == "recording":
            p.setPen(txt_color)
            p.setFont(QFont(_FONT_UI, 14))
            p.drawText(text_x, M, W - text_x - 60, H // 2, Qt.AlignmentFlag.AlignVCenter, "Recording")
            p.setPen(sub_color)
            p.setFont(QFont(_FONT_UI, 11))
            p.drawText(text_x, H // 2 - 2, W - text_x - 16, H // 2, Qt.AlignmentFlag.AlignTop, "Press hotkey to stop")
            p.setPen(QColor(255, 255, 255, 77) if self.is_dark else QColor(0, 0, 0, 77))
            p.setFont(QFont(_FONT_MONO, 13))
            m, s = self.elapsed // 60, self.elapsed % 60
            p.drawText(0, 0, W - M - 14, H, Qt.AlignmentFlag.AlignVCenter | Qt.AlignmentFlag.AlignRight, f"{m:02d}:{s:02d}")
        elif self.state == "transcribing":
            p.setPen(QColor(255, 255, 255, 180) if self.is_dark else QColor(0, 0, 0, 180))
            p.setFont(QFont(_FONT_UI, 14))
            p.drawText(text_x, M, W - text_x - 16, H // 2, Qt.AlignmentFlag.AlignVCenter, "Transcribing...")
            p.setPen(sub_color)
            p.setFont(QFont(_FONT_UI, 11))
            p.drawText(text_x, H // 2 - 2, W - text_x - 16, H // 2, Qt.AlignmentFlag.AlignTop, "Processing audio")
        elif self.state == "done":
            p.setPen(QColor(255, 255, 255, 153) if self.is_dark else QColor(0, 0, 0, 153))
            p.setFont(QFont(_FONT_UI, 13))
            p.drawText(text_x, M, W - text_x - 16, H // 2, Qt.AlignmentFlag.AlignVCenter, self.preview)
            p.setPen(sub_color)
            p.setFont(QFont(_FONT_UI, 10))
            p.drawText(text_x, H // 2 - 2, W - text_x - 16, H // 2, Qt.AlignmentFlag.AlignTop, "Copied to clipboard")
        elif self.state == "error":
            p.setPen(QColor(255, 69, 58))
            p.setFont(QFont(_FONT_UI, 13))
            p.drawText(text_x, 0, W - text_x - 16, H, Qt.AlignmentFlag.AlignVCenter, self.preview)

        p.end()

# ──────────────────────────────────────────────────────────────────────────────
# HISTORY WINDOW
# ──────────────────────────────────────────────────────────────────────────────

class HistoryWindow(QWidget):
    def __init__(self, history: History, is_dark: bool = True):
        super().__init__()
        self.history = history
        self._is_dark = is_dark
        self.setWindowTitle("Whisper — History")
        self.setWindowFlags(Qt.WindowType.Window | Qt.WindowType.WindowCloseButtonHint)
        self.setMinimumSize(480, 420)
        self.resize(480, 580)
        self._build()

    def showEvent(self, e):
        super().showEvent(e)
        self._refresh()

    def _build(self):
        root = QVBoxLayout(self)
        root.setContentsMargins(16, 16, 16, 12)
        root.setSpacing(10)

        hdr = QHBoxLayout()
        title = QLabel("History")
        title.setFont(QFont(_FONT_UI, 13, QFont.Weight.Light))
        hdr.addWidget(title)
        hdr.addStretch()
        self._clear_btn = QPushButton("Clear all")
        self._clear_btn.setObjectName("danger")
        self._clear_btn.clicked.connect(self._clear)
        hdr.addWidget(self._clear_btn)
        root.addLayout(hdr)

        line = QFrame()
        line.setFrameShape(QFrame.Shape.HLine)
        line.setObjectName("separator")
        root.addWidget(line)

        self._scroll = QScrollArea()
        self._scroll.setWidgetResizable(True)
        self._scroll.setFrameShape(QFrame.Shape.NoFrame)
        self._inner = QWidget()
        self._vbox  = QVBoxLayout(self._inner)
        self._vbox.setContentsMargins(0, 4, 0, 4)
        self._vbox.setSpacing(8)
        self._vbox.addStretch()
        self._scroll.setWidget(self._inner)
        root.addWidget(self._scroll)

        self._refresh()

    def _refresh(self):
        while self._vbox.count() > 1:
            item = self._vbox.takeAt(0)
            if item.widget():
                item.widget().deleteLater()

        if not self.history.entries:
            lbl = QLabel("No transcriptions yet.\nUse the hotkey to record something.")
            lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
            lbl.setObjectName("empty_hint")
            self._vbox.insertWidget(0, lbl)
            return

        for i, entry in enumerate(self.history.entries):
            self._vbox.insertWidget(i, self._card(entry))

    def _card(self, entry: dict) -> QFrame:
        card = QFrame()
        card.setObjectName("card")
        lay = QVBoxLayout(card)
        lay.setContentsMargins(12, 8, 12, 10)
        lay.setSpacing(5)

        top = QHBoxLayout()
        ts  = QLabel(self._fmt(entry["ts"]))
        ts.setObjectName("muted")
        top.addWidget(ts)
        top.addStretch()

        btn = QPushButton("Copy")
        btn.setObjectName("ghost")
        btn.setFixedSize(56, 22)
        text = entry["text"]

        def on_copy(_, b=btn, t=text):
            pyperclip.copy(t)
            b.setText("Copied")
            QTimer.singleShot(1400, lambda: b.setText("Copy"))

        btn.clicked.connect(on_copy)
        top.addWidget(btn)
        lay.addLayout(top)

        body = QLabel(text)
        body.setWordWrap(True)
        body.setObjectName("body_text")
        lay.addWidget(body)
        return card

    def _fmt(self, ts: str) -> str:
        try:
            dt  = datetime.datetime.fromisoformat(ts)
            now = datetime.datetime.now()
            if dt.date() == now.date():
                return "Today  " + dt.strftime("%H:%M:%S")
            if (now.date() - dt.date()).days == 1:
                return "Yesterday  " + dt.strftime("%H:%M")
            return dt.strftime("%b %d  %H:%M")
        except Exception:
            return ts

    def _clear(self):
        self.history.clear()
        self._refresh()

    def refresh(self):
        if self.isVisible():
            self._refresh()

# ──────────────────────────────────────────────────────────────────────────────
# SETTINGS DIALOG
# ──────────────────────────────────────────────────────────────────────────────

class HotkeyCapture(QPushButton):
    """Button that captures keyboard shortcuts via Quartz event tap."""
    hotkey_changed = Signal(str)

    _MOD_FLAG_NAMES = [
        (Quartz.kCGEventFlagMaskCommand, "cmd"),
        (Quartz.kCGEventFlagMaskControl, "ctrl"),
        (Quartz.kCGEventFlagMaskAlternate, "alt"),
        (Quartz.kCGEventFlagMaskShift, "shift"),
    ]

    _VK_NAMES = None  # lazily initialized

    def __init__(self, hotkey: str = ""):
        super().__init__()
        self._hotkey = hotkey
        self._listening = False
        self._tap = None
        self._source = None
        self.setMinimumHeight(27)
        self.clicked.connect(self._toggle_listening)
        self._update_text()

    @property
    def hotkey(self) -> str:
        return self._hotkey

    @hotkey.setter
    def hotkey(self, v: str):
        self._hotkey = v
        self._update_text()

    def _update_text(self):
        if self._listening:
            self.setText("Press your shortcut...")
        else:
            self.setText(self._hotkey or "Click to set")

    def _toggle_listening(self):
        if self._listening:
            self._stop_listening()
        else:
            self._start_listening()

    def _start_listening(self):
        self._listening = True
        self._update_text()
        self._tap = Quartz.CGEventTapCreate(
            Quartz.kCGSessionEventTap,
            Quartz.kCGHeadInsertEventTap,
            Quartz.kCGEventTapOptionListenOnly,
            Quartz.CGEventMaskBit(Quartz.kCGEventKeyDown),
            self._on_cg_event,
            None,
        )
        if self._tap is None:
            self._listening = False
            self.setText(self._hotkey or "Click to set")
            return
        self._source = Quartz.CFMachPortCreateRunLoopSource(None, self._tap, 0)
        Quartz.CFRunLoopAddSource(
            Quartz.CFRunLoopGetMain(), self._source, Quartz.kCFRunLoopCommonModes
        )
        Quartz.CGEventTapEnable(self._tap, True)

    def _stop_listening(self):
        self._listening = False
        if self._tap:
            Quartz.CGEventTapEnable(self._tap, False)
            if self._source:
                Quartz.CFRunLoopRemoveSource(
                    Quartz.CFRunLoopGetMain(), self._source, Quartz.kCFRunLoopCommonModes
                )
                self._source = None
            self._tap = None
        self._update_text()

    def _on_cg_event(self, proxy, event_type, event, refcon):
        if not self._listening:
            return event
        keycode = Quartz.CGEventGetIntegerValueField(event, Quartz.kCGKeyboardEventKeycode)
        flags = Quartz.CGEventGetFlags(event)

        # Escape with no modifiers cancels
        if keycode == 53 and not (flags & (
            Quartz.kCGEventFlagMaskCommand | Quartz.kCGEventFlagMaskControl |
            Quartz.kCGEventFlagMaskAlternate | Quartz.kCGEventFlagMaskShift
        )):
            QTimer.singleShot(0, self._stop_listening)
            return event

        parts = []
        for flag, name in self._MOD_FLAG_NAMES:
            if flags & flag:
                parts.append(name)

        if self._VK_NAMES is None:
            HotkeyCapture._VK_NAMES = {v: k for k, v in _KEYCODE_MAP.items()}
        key_name = self._VK_NAMES.get(keycode)
        if key_name is None:
            return event  # unknown key

        parts.append(key_name)
        self._hotkey = "+".join(parts)
        QTimer.singleShot(0, self._stop_listening)
        QTimer.singleShot(0, lambda: self.hotkey_changed.emit(self._hotkey))
        return event


from PySide6.QtWidgets import QStackedWidget


class SettingsDialog(QDialog):
    saved = Signal(object)

    _POSITIONS = [
        ("Top center",    "top-center"),
        ("Top right",     "top-right"),
        ("Top left",      "top-left"),
        ("Bottom center", "bottom-center"),
    ]
    _LANGUAGES = [
        ("Auto-detect", "auto"), ("English",    "en"), ("Arabic",     "ar"),
        ("French",      "fr"),   ("Spanish",    "es"), ("German",     "de"),
        ("Chinese",     "zh"),   ("Japanese",   "ja"), ("Russian",    "ru"),
        ("Portuguese",  "pt"),   ("Italian",    "it"), ("Korean",     "ko"),
    ]

    def __init__(self, cfg: Config, history: History, parent=None, is_dark: bool = True):
        super().__init__(parent)
        self.cfg     = cfg
        self.history = history
        self._is_dark = is_dark
        self.setWindowTitle("Settings")
        self.setWindowFlags(Qt.WindowType.Dialog | Qt.WindowType.WindowCloseButtonHint)
        self.setFixedSize(520, 400)
        self._build()

    def _c(self, dark_val: str, light_val: str) -> str:
        """Return color based on current theme."""
        return dark_val if self._is_dark else light_val

    def _card(self) -> QFrame:
        f = QFrame(); f.setObjectName("card"); return f

    def _form(self, card: QFrame) -> QFormLayout:
        fl = QFormLayout(card)
        fl.setContentsMargins(14, 10, 14, 10)
        fl.setSpacing(10)
        fl.setLabelAlignment(Qt.AlignmentFlag.AlignLeft | Qt.AlignmentFlag.AlignVCenter)
        fl.setFieldGrowthPolicy(QFormLayout.FieldGrowthPolicy.ExpandingFieldsGrow)
        return fl

    def _build(self):
        root = QHBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(0)

        # Sidebar
        sidebar = QFrame()
        sidebar.setObjectName("sidebar")
        sidebar.setFixedWidth(140)
        sb_lay = QVBoxLayout(sidebar)
        sb_lay.setContentsMargins(8, 16, 8, 8)
        sb_lay.setSpacing(2)

        title = QLabel("Settings")
        title.setFont(QFont(_FONT_UI, 14, QFont.Weight.Light))
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        sb_lay.addWidget(title)
        sb_lay.addSpacing(12)

        self._sidebar_btns = []
        for label in ("General", "Model", "Audio", "Vocabulary", "History"):
            btn = QPushButton(label)
            btn.setObjectName("sidebar_item")
            btn.clicked.connect(lambda _, l=label: self._switch_page(l))
            sb_lay.addWidget(btn)
            self._sidebar_btns.append((label, btn))
        sb_lay.addStretch()

        root.addWidget(sidebar)

        # Content area
        right = QVBoxLayout()
        right.setContentsMargins(16, 16, 16, 12)
        right.setSpacing(10)

        self._stack = QStackedWidget()
        self._pages = {}

        self._build_general_page()
        self._build_model_page()
        self._build_audio_page()
        self._build_vocabulary_page()
        self._build_history_page()

        right.addWidget(self._stack, 1)

        # Bottom buttons
        btns = QHBoxLayout(); btns.setSpacing(8)
        btns.addStretch()
        cancel = QPushButton("Cancel"); cancel.clicked.connect(self.reject)
        btns.addWidget(cancel)
        save = QPushButton("Save"); save.setObjectName("primary")
        save.clicked.connect(self._save)
        btns.addWidget(save)
        right.addLayout(btns)

        root.addLayout(right, 1)

        self._switch_page("General")

    def _switch_page(self, name: str):
        if name in self._pages:
            self._stack.setCurrentWidget(self._pages[name])
        for label, btn in self._sidebar_btns:
            btn.setObjectName("sidebar_active" if label == name else "sidebar_item")
            btn.style().unpolish(btn)
            btn.style().polish(btn)

    def _build_general_page(self):
        page = QWidget()
        lay = QVBoxLayout(page)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(10)

        card = self._card(); fl = self._form(card)
        self._hotkey_edit = HotkeyCapture(self.cfg.hotkey)
        self._paste_toggle = ToggleSwitch(self.cfg.auto_paste)
        self._theme_cb = QComboBox()
        for label, data in (("Dark", "dark"), ("Light", "light"), ("Auto (follow system)", "auto")):
            self._theme_cb.addItem(label, data)
        theme_idx = next((i for i, (_, d) in enumerate((("Dark","dark"),("Light","light"),("Auto (follow system)","auto"))) if d == self.cfg.theme), 2)
        self._theme_cb.setCurrentIndex(theme_idx)
        self._pos_cb = QComboBox()
        for label, data in self._POSITIONS:
            self._pos_cb.addItem(label, data)
        cur_pos = next((i for i, (_, d) in enumerate(self._POSITIONS) if d == self.cfg.overlay_position), 0)
        self._pos_cb.setCurrentIndex(cur_pos)

        fl.addRow("Hotkey", self._hotkey_edit)
        fl.addRow("Auto-paste", self._paste_toggle)
        fl.addRow("Overlay position", self._pos_cb)
        fl.addRow("Theme", self._theme_cb)
        lay.addWidget(card)
        lay.addStretch()

        self._pages["General"] = page
        self._stack.addWidget(page)

    def _build_model_page(self):
        page = QWidget()
        lay = QVBoxLayout(page)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(10)

        # ASR Model
        lbl = QLabel("TRANSCRIPTION")
        lbl.setObjectName("section_label")
        lay.addWidget(lbl)

        card = self._card(); fl = self._form(card)
        self._model_input = QLineEdit(self.cfg.model)
        self._model_input.setPlaceholderText("e.g. Qwen/Qwen3-ASR-0.6B")
        fl.addRow("ASR model", self._model_input)
        lay.addWidget(card)

        lay.addSpacing(8)

        # Post-process model
        lbl2 = QLabel("POST-PROCESSING")
        lbl2.setObjectName("section_label")
        lay.addWidget(lbl2)

        card2 = self._card(); fl2 = self._form(card2)
        self._pp_toggle = ToggleSwitch(self.cfg.post_process)
        fl2.addRow("Enable", self._pp_toggle)
        self._pp_model_input = QLineEdit(self.cfg.post_process_model)
        self._pp_model_input.setPlaceholderText("e.g. mlx-community/Qwen3.5-4B-MLX-4bit")
        fl2.addRow("LLM model", self._pp_model_input)
        lay.addWidget(card2)

        lay.addStretch()

        self._pages["Model"] = page
        self._stack.addWidget(page)

    def _build_audio_page(self):
        page = QWidget()
        lay = QVBoxLayout(page)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(10)

        card = self._card(); fl = self._form(card)
        self._lang_cb = QComboBox()
        for label, code in self._LANGUAGES:
            self._lang_cb.addItem(label, code)
        idx = next((i for i, (_, c) in enumerate(self._LANGUAGES) if c == self.cfg.language), 0)
        self._lang_cb.setCurrentIndex(idx)
        fl.addRow("Language", self._lang_cb)
        lay.addWidget(card)
        lay.addStretch()

        self._pages["Audio"] = page
        self._stack.addWidget(page)

    def _build_vocabulary_page(self):
        page = QWidget()
        lay = QVBoxLayout(page)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(12)

        # ── Vocabulary ─────────────────────────────────────────────────────
        lbl = QLabel("VOCABULARY")
        lbl.setObjectName("section_label")
        lay.addWidget(lbl)

        hint = QLabel("Names, terms, and words the model should recognize correctly.")
        hint.setObjectName("body_text")
        hint.setWordWrap(True)
        lay.addWidget(hint)

        # Add row
        add_row = QHBoxLayout(); add_row.setSpacing(8)
        self._vocab_input = QLineEdit()
        self._vocab_input.setPlaceholderText("Type a word and press Enter")
        self._vocab_input.returnPressed.connect(self._add_vocab_word)
        add_row.addWidget(self._vocab_input, 1)
        add_btn = QPushButton("+"); add_btn.setObjectName("primary"); add_btn.setFixedSize(32, 32)
        add_btn.clicked.connect(self._add_vocab_word)
        add_row.addWidget(add_btn)
        lay.addLayout(add_row)

        # Tags container using FlowLayout
        self._vocab_tags_widget = QWidget()
        self._vocab_tags_layout = _FlowLayout(self._vocab_tags_widget, spacing=6)
        self._vocab_tags_widget.setLayout(self._vocab_tags_layout)

        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.Shape.NoFrame)
        scroll.setMinimumHeight(50)
        scroll.setMaximumHeight(120)
        scroll.setWidget(self._vocab_tags_widget)
        lay.addWidget(scroll)

        self._vocab_words = list(self.cfg.vocabulary)
        self._refresh_vocab_list()

        lay.addSpacing(4)

        # ── Replacements ───────────────────────────────────────────────────
        lbl2 = QLabel("REPLACEMENTS")
        lbl2.setObjectName("section_label")
        lay.addWidget(lbl2)

        hint2 = QLabel("Auto-replace words after transcription.")
        hint2.setObjectName("body_text")
        hint2.setWordWrap(True)
        lay.addWidget(hint2)

        # Add replacement row
        repl_row = QHBoxLayout(); repl_row.setSpacing(8)
        self._repl_from = QLineEdit()
        self._repl_from.setPlaceholderText("Original")
        repl_row.addWidget(self._repl_from, 1)
        arrow = QLabel("→")
        arrow.setFixedWidth(20)
        arrow.setAlignment(Qt.AlignmentFlag.AlignCenter)
        arrow.setObjectName("body_text")
        repl_row.addWidget(arrow)
        self._repl_to = QLineEdit()
        self._repl_to.setPlaceholderText("Replace with")
        self._repl_to.returnPressed.connect(self._add_replacement)
        repl_row.addWidget(self._repl_to, 1)
        add_repl = QPushButton("+"); add_repl.setObjectName("primary"); add_repl.setFixedSize(32, 32)
        add_repl.clicked.connect(self._add_replacement)
        repl_row.addWidget(add_repl)
        lay.addLayout(repl_row)

        # Replacement list
        self._repl_list_widget = QWidget()
        self._repl_list_layout = QVBoxLayout(self._repl_list_widget)
        self._repl_list_layout.setContentsMargins(0, 0, 0, 0)
        self._repl_list_layout.setSpacing(4)
        self._repl_list_layout.addStretch()

        scroll2 = QScrollArea()
        scroll2.setWidgetResizable(True)
        scroll2.setFrameShape(QFrame.Shape.NoFrame)
        scroll2.setMinimumHeight(50)
        scroll2.setMaximumHeight(120)
        scroll2.setWidget(self._repl_list_widget)
        lay.addWidget(scroll2)

        self._repl_items = list(self.cfg.replacements)
        self._refresh_repl_list()

        lay.addStretch()

        self._pages["Vocabulary"] = page
        self._stack.addWidget(page)

    def _add_vocab_word(self):
        word = self._vocab_input.text().strip()
        if word and word not in self._vocab_words:
            self._vocab_words.append(word)
            self._refresh_vocab_list()
        self._vocab_input.clear()

    def _remove_vocab_word(self, word: str):
        if word in self._vocab_words:
            self._vocab_words.remove(word)
            self._refresh_vocab_list()

    def _refresh_vocab_list(self):
        while self._vocab_tags_layout.count():
            item = self._vocab_tags_layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()
        for word in self._vocab_words:
            tag = QPushButton(f"  {word}  ✕")
            tag.setCursor(Qt.CursorShape.PointingHandCursor)
            bg = self._c("rgba(255,255,255,0.08)", "rgba(0,0,0,0.06)")
            border = self._c("rgba(255,255,255,0.12)", "rgba(0,0,0,0.1)")
            fg = self._c("rgba(255,255,255,0.8)", "rgba(0,0,0,0.8)")
            tag.setStyleSheet(f"""
                QPushButton {{
                    background: {bg};
                    border: 1px solid {border};
                    border-radius: 12px;
                    color: {fg};
                    font-size: 12px;
                    padding: 4px 10px;
                }}
                QPushButton:hover {{
                    background: rgba(255,69,58,0.2);
                    border-color: rgba(255,69,58,0.4);
                    color: #ff453a;
                }}
            """)
            tag.clicked.connect(lambda _, w=word: self._remove_vocab_word(w))
            self._vocab_tags_layout.addWidget(tag)

    def _add_replacement(self):
        fr = self._repl_from.text().strip()
        to = self._repl_to.text().strip()
        if fr and to:
            self._repl_items.append({"from": fr, "to": to})
            self._refresh_repl_list()
        self._repl_from.clear()
        self._repl_to.clear()

    def _remove_replacement(self, idx: int):
        if 0 <= idx < len(self._repl_items):
            self._repl_items.pop(idx)
            self._refresh_repl_list()

    def _refresh_repl_list(self):
        while self._repl_list_layout.count() > 1:
            item = self._repl_list_layout.takeAt(0)
            if item.widget():
                item.widget().deleteLater()
        for i, r in enumerate(self._repl_items):
            row = QFrame()
            row_bg = self._c("rgba(255,255,255,0.04)", "rgba(0,0,0,0.03)")
            row_border = self._c("rgba(255,255,255,0.08)", "rgba(0,0,0,0.08)")
            row_hover = self._c("rgba(255,255,255,0.15)", "rgba(0,0,0,0.15)")
            row.setStyleSheet(f"""
                QFrame {{
                    background: {row_bg};
                    border: 1px solid {row_border};
                    border-radius: 8px;
                }}
                QFrame:hover {{
                    border-color: {row_hover};
                }}
            """)
            rl = QHBoxLayout(row); rl.setContentsMargins(12, 6, 8, 6); rl.setSpacing(8)
            from_fg = self._c("rgba(255,255,255,0.85)", "rgba(0,0,0,0.85)")
            from_lbl = QLabel(r['from'])
            from_lbl.setStyleSheet(f"color: {from_fg}; font-size: 13px; border: none; background: transparent;")
            rl.addWidget(from_lbl)
            arrow_fg = self._c("rgba(255,255,255,0.3)", "rgba(0,0,0,0.3)")
            arrow = QLabel("→")
            arrow.setStyleSheet(f"color: {arrow_fg}; font-size: 14px; border: none; background: transparent;")
            arrow.setFixedWidth(20)
            arrow.setAlignment(Qt.AlignmentFlag.AlignCenter)
            rl.addWidget(arrow)
            to_lbl = QLabel(r['to'])
            to_lbl.setStyleSheet("color: #0a84ff; font-size: 13px; border: none; background: transparent;")
            rl.addWidget(to_lbl, 1)
            rm_fg = self._c("rgba(255,255,255,0.25)", "rgba(0,0,0,0.25)")
            rm = QPushButton("✕")
            rm.setFixedSize(24, 24)
            rm.setCursor(Qt.CursorShape.PointingHandCursor)
            rm.setStyleSheet(f"""
                QPushButton {{
                    background: transparent; border: none;
                    color: {rm_fg}; font-size: 12px;
                }}
                QPushButton:hover {{ color: #ff453a; }}
            """)
            rm.clicked.connect(lambda _, idx=i: self._remove_replacement(idx))
            rl.addWidget(rm)
            self._repl_list_layout.insertWidget(i, row)

    def _build_history_page(self):
        page = QWidget()
        lay = QVBoxLayout(page)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(10)

        card = self._card(); fl = self._form(card)
        self._hist_spin = QSpinBox(); self._hist_spin.setRange(10, 500)
        self._hist_spin.setValue(self.cfg.history_limit)
        fl.addRow("Keep last entries", self._hist_spin)
        lay.addWidget(card)

        lay.addSpacing(4)
        clr = QPushButton("Clear all history"); clr.setObjectName("danger")
        clr.clicked.connect(lambda: (self.history.clear(), clr.setText("Cleared")))
        lay.addWidget(clr)
        lay.addStretch()

        self._pages["History"] = page
        self._stack.addWidget(page)


    def _save(self):
        self.cfg.model            = self._model_input.text().strip() or "Qwen/Qwen3-ASR-0.6B"
        self.cfg.post_process_model = self._pp_model_input.text().strip() or "mlx-community/Qwen3.5-4B-MLX-4bit"
        self.cfg.language         = self._lang_cb.currentData()
        self.cfg.hotkey           = self._hotkey_edit.hotkey or "cmd+shift+space"
        self.cfg.auto_paste       = self._paste_toggle.checked
        self.cfg.post_process     = self._pp_toggle.checked
        self.cfg.overlay_position = self._pos_cb.currentData()
        self.cfg.history_limit    = self._hist_spin.value()
        self.cfg.theme            = self._theme_cb.currentData()
        # Vocabulary
        self.cfg.vocabulary = list(self._vocab_words)
        self.cfg.replacements = list(self._repl_items)
        self.cfg.save()
        self.saved.emit(self.cfg)
        self.accept()

# ──────────────────────────────────────────────────────────────────────────────
# HOTKEY LISTENER (Quartz CGEvent tap — main-thread safe on macOS)
# ──────────────────────────────────────────────────────────────────────────────

# Modifier flag masks from Quartz
_MOD_MAP = {
    "cmd": Quartz.kCGEventFlagMaskCommand,
    "command": Quartz.kCGEventFlagMaskCommand,
    "ctrl": Quartz.kCGEventFlagMaskControl,
    "control": Quartz.kCGEventFlagMaskControl,
    "alt": Quartz.kCGEventFlagMaskAlternate,
    "option": Quartz.kCGEventFlagMaskAlternate,
    "shift": Quartz.kCGEventFlagMaskShift,
}

# Virtual keycodes for common keys on macOS
_KEYCODE_MAP = {
    "space": 49, "return": 36, "enter": 36, "tab": 48,
    "escape": 53, "esc": 53, "delete": 51,
    "f1": 122, "f2": 120, "f3": 99, "f4": 118,
    "f5": 96, "f6": 97, "f7": 98, "f8": 100,
    "f9": 101, "f10": 109, "f11": 103, "f12": 111,
    "a": 0, "b": 11, "c": 8, "d": 2, "e": 14, "f": 3,
    "g": 5, "h": 4, "i": 34, "j": 38, "k": 40, "l": 37,
    "m": 46, "n": 45, "o": 31, "p": 35, "q": 12, "r": 15,
    "s": 1, "t": 17, "u": 32, "v": 9, "w": 13, "x": 7,
    "y": 16, "z": 6,
    "0": 29, "1": 18, "2": 19, "3": 20, "4": 21,
    "5": 23, "6": 22, "7": 26, "8": 28, "9": 25,
}


def _parse_hotkey(hotkey_str: str):
    """Parse 'cmd+shift+space' into (modifier_mask, keycode)."""
    parts = [p.strip().lower() for p in hotkey_str.strip().split("+")]
    mod_mask = 0
    keycode = None
    for part in parts:
        if part in _MOD_MAP:
            mod_mask |= _MOD_MAP[part]
        elif part in _KEYCODE_MAP:
            keycode = _KEYCODE_MAP[part]
        else:
            raise ValueError(f"Unknown key: {part}")
    if keycode is None:
        raise ValueError(f"No key specified in hotkey: {hotkey_str}")
    return mod_mask, keycode


class HotkeyListener:
    """Global hotkey using Quartz CGEvent tap with a QTimer polling the run loop."""

    def __init__(self, hotkey_str: str, callback):
        self._callback = callback
        self._mod_mask, self._keycode = _parse_hotkey(hotkey_str)
        self._tap = None
        self._timer = None
        self._source = None

    def _event_callback(self, proxy, event_type, event, refcon):
        if event_type == Quartz.kCGEventKeyDown:
            keycode = Quartz.CGEventGetIntegerValueField(event, Quartz.kCGKeyboardEventKeycode)
            flags = Quartz.CGEventGetFlags(event)
            clean_flags = flags & (
                Quartz.kCGEventFlagMaskCommand |
                Quartz.kCGEventFlagMaskShift |
                Quartz.kCGEventFlagMaskAlternate |
                Quartz.kCGEventFlagMaskControl
            )
            # Compare only the 4 standard modifier bits, ignoring device-level sub-bits
            _MOD_BITS = (
                Quartz.kCGEventFlagMaskCommand |
                Quartz.kCGEventFlagMaskShift |
                Quartz.kCGEventFlagMaskAlternate |
                Quartz.kCGEventFlagMaskControl
            )
            if keycode == self._keycode and (flags & _MOD_BITS) == self._mod_mask:
                self._callback()
        return event

    def start(self):
        self._tap = Quartz.CGEventTapCreate(
            Quartz.kCGSessionEventTap,
            Quartz.kCGHeadInsertEventTap,
            Quartz.kCGEventTapOptionListenOnly,
            Quartz.CGEventMaskBit(Quartz.kCGEventKeyDown),
            self._event_callback,
            None,
        )
        if self._tap is None:
            raise RuntimeError(
                "Failed to create event tap. "
                "Grant Accessibility permission in System Settings > Privacy & Security > Accessibility."
            )
        self._source = Quartz.CFMachPortCreateRunLoopSource(None, self._tap, 0)
        Quartz.CFRunLoopAddSource(
            Quartz.CFRunLoopGetMain(),
            self._source,
            Quartz.kCFRunLoopCommonModes,
        )
        Quartz.CGEventTapEnable(self._tap, True)

    def stop(self):
        if self._tap:
            Quartz.CGEventTapEnable(self._tap, False)
            if self._source:
                Quartz.CFRunLoopRemoveSource(
                    Quartz.CFRunLoopGetMain(),
                    self._source,
                    Quartz.kCFRunLoopCommonModes,
                )
                self._source = None
            self._tap = None


# ──────────────────────────────────────────────────────────────────────────────
# MAIN WINDOW
# ──────────────────────────────────────────────────────────────────────────────

class MainWindow(QWidget):
    """Dashboard-style main window with status and recent transcriptions."""

    def __init__(self, app_controller):
        super().__init__()
        self._app = app_controller
        self.setWindowTitle("Whisper Hotkey")
        self.setFixedSize(380, 360)
        self.setWindowFlags(Qt.WindowType.Window | Qt.WindowType.WindowCloseButtonHint)
        self._build()

    def _build(self):
        root = QVBoxLayout(self)
        root.setContentsMargins(20, 16, 20, 16)
        root.setSpacing(0)

        # Title + subtitle
        title = QLabel("Whisper Hotkey")
        title.setFont(QFont(_FONT_UI, 20, QFont.Weight.Light))
        title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        root.addWidget(title)

        self._subtitle = QLabel(f"Qwen3-ASR  ·  {self._app.cfg.hotkey}")
        self._subtitle.setObjectName("muted")
        self._subtitle.setAlignment(Qt.AlignmentFlag.AlignCenter)
        root.addWidget(self._subtitle)

        root.addSpacing(6)

        sep = QFrame(); sep.setFrameShape(QFrame.Shape.HLine); sep.setObjectName("separator")
        root.addWidget(sep)

        root.addSpacing(8)

        # Status
        self._status = QLabel("Loading model...")
        self._status.setObjectName("body_text")
        self._status.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self._status.setWordWrap(True)
        root.addWidget(self._status)

        root.addSpacing(10)

        # Recent transcriptions section
        recent_label = QLabel("RECENT")
        recent_label.setObjectName("section_label")
        root.addWidget(recent_label)

        root.addSpacing(4)

        self._recent_container = QVBoxLayout()
        self._recent_container.setSpacing(4)
        root.addLayout(self._recent_container)

        self._update_recent()

        root.addStretch()

        # Bottom separator
        sep2 = QFrame(); sep2.setFrameShape(QFrame.Shape.HLine); sep2.setObjectName("separator")
        root.addWidget(sep2)

        root.addSpacing(8)

        # Bottom buttons
        btn_row = QHBoxLayout()
        btn_row.setSpacing(8)
        hist_btn = QPushButton("All History")
        hist_btn.setObjectName("ghost")
        hist_btn.clicked.connect(self._app._open_history)
        btn_row.addWidget(hist_btn)
        settings_btn = QPushButton("Settings")
        settings_btn.setObjectName("ghost")
        settings_btn.clicked.connect(self._app._open_settings)
        btn_row.addWidget(settings_btn)
        root.addLayout(btn_row)

    def _update_recent(self):
        # Clear existing
        while self._recent_container.count():
            item = self._recent_container.takeAt(0)
            if item.widget():
                item.widget().deleteLater()

        entries = self._app.history.entries[:3]
        if not entries:
            hint = QLabel("No transcriptions yet")
            hint.setObjectName("muted")
            hint.setAlignment(Qt.AlignmentFlag.AlignCenter)
            self._recent_container.addWidget(hint)
            return

        for entry in entries:
            card = QFrame()
            card.setObjectName("card")
            lay = QHBoxLayout(card)
            lay.setContentsMargins(12, 8, 12, 8)

            text = entry["text"]
            preview = (text[:40] + "...") if len(text) > 40 else text
            lbl = QLabel(preview)
            lbl.setObjectName("body_text")
            lay.addWidget(lbl, 1)

            # Time ago
            time_lbl = QLabel(self._time_ago(entry.get("ts", "")))
            time_lbl.setObjectName("muted")
            lay.addWidget(time_lbl)

            self._recent_container.addWidget(card)

    def _time_ago(self, ts: str) -> str:
        try:
            dt = datetime.datetime.fromisoformat(ts)
            diff = datetime.datetime.now() - dt
            if diff.total_seconds() < 60:
                return "now"
            if diff.total_seconds() < 3600:
                return f"{int(diff.total_seconds() // 60)}m"
            if diff.total_seconds() < 86400:
                return f"{int(diff.total_seconds() // 3600)}h"
            return f"{diff.days}d"
        except Exception:
            return ""

    def set_status(self, text: str):
        self._status.setText(text)
        self._update_recent()

    def update_hotkey(self, hotkey: str):
        self._subtitle.setText(f"Qwen3-ASR  ·  {hotkey}")

    def closeEvent(self, event):
        event.ignore()
        self.hide()


# ──────────────────────────────────────────────────────────────────────────────
# MAIN APP CONTROLLER
# ──────────────────────────────────────────────────────────────────────────────

class WhisperApp(QObject):
    _toggle_sig        = Signal()
    _rec_started_sig   = Signal()
    _audio_ready_sig   = Signal(object)

    def __init__(self):
        super().__init__()
        self.cfg     = Config.load()
        self.history = History(self.cfg.history_limit)
        self.model   = None
        self._recording   = False
        self._chunks: list[np.ndarray] = []
        self._stream = None
        self._lock   = threading.Lock()
        self._worker = None
        self._hotkey_listener: HotkeyListener | None = None
        self._saved_app = None

        self.overlay     = RecordingOverlay(self.cfg.overlay_position)
        self._hist_win: HistoryWindow | None = None

        # Main window
        self._main_win = MainWindow(self)

        # Tray
        self.tray = QSystemTrayIcon()
        self.tray.setIcon(_make_icon("loading"))
        self.tray.setToolTip("Whisper Hotkey — loading...")
        self._build_tray_menu()
        self.tray.show()

        # Thread-safe signal bridges
        self._toggle_sig.connect(self._on_toggle)
        self._rec_started_sig.connect(self._on_rec_started)
        self._audio_ready_sig.connect(self._on_audio_ready)

        # Theme
        self._last_dark: bool | None = None
        self._is_dark: bool = True
        self._apply_theme()
        self._theme_timer = QTimer()
        self._theme_timer.setInterval(5000)
        self._theme_timer.timeout.connect(self._apply_theme)
        self._theme_timer.start()

        # Show main window
        self._main_win.show()
        self._main_win.raise_()
        self._main_win.activateWindow()

        # Load model
        self._start_loader()

    # ── Theme ────────────────────────────────────────────────────────────────

    def _apply_theme(self):
        t = self.cfg.theme
        if t == "dark":
            is_dark = True
        elif t == "light":
            is_dark = False
        else:
            is_dark = not _macos_is_light()
        if is_dark == self._last_dark:
            return
        self._last_dark = is_dark
        self._is_dark = is_dark
        style = get_style(is_dark)
        QApplication.instance().setStyleSheet(style)
        self.overlay.is_dark = is_dark
        self.overlay.update()
        if self._hist_win:
            self._hist_win.setStyleSheet(style)

    # ── Tray ─────────────────────────────────────────────────────────────────

    def _build_tray_menu(self):
        menu = QMenu()
        show_a = QAction("Show Window", menu); show_a.triggered.connect(self._show_main)
        menu.addAction(show_a)
        menu.addSeparator()
        hist_a = QAction("History...", menu); hist_a.triggered.connect(self._open_history)
        menu.addAction(hist_a)
        sett_a = QAction("Settings...", menu); sett_a.triggered.connect(self._open_settings)
        menu.addAction(sett_a)
        menu.addSeparator()
        quit_a = QAction("Quit", menu); quit_a.triggered.connect(self._quit)
        menu.addAction(quit_a)
        self.tray.setContextMenu(menu)
        self.tray.activated.connect(
            lambda r: self._show_main() if r in (
                QSystemTrayIcon.ActivationReason.Trigger,
                QSystemTrayIcon.ActivationReason.DoubleClick,
            ) else None
        )

    # ── Model loading ─────────────────────────────────────────────────────────

    def _start_loader(self):
        self._loader = ModelLoader(self.cfg)
        self._loader.loaded.connect(self._on_loaded)
        self._loader.failed.connect(self._on_load_failed)
        self._loader.status.connect(lambda s: (self.tray.setToolTip(f"Whisper Hotkey — {s}"), self._main_win.set_status(s)))
        self._loader.start()

    def _on_loaded(self, model):
        self.model = model
        self.tray.setIcon(_make_icon("idle"))
        self.tray.setToolTip(f"Whisper Hotkey — {self.cfg.model}  ·  {self.cfg.hotkey}")
        self._main_win.set_status(f"Ready — model '{self.cfg.model}' loaded\nPress {self.cfg.hotkey} to record")
        self._register_hotkey()

    def _on_load_failed(self, err: str):
        self.tray.setIcon(_make_icon("idle"))
        self.tray.showMessage("Whisper Hotkey", f"Failed to load model:\n{err}", QSystemTrayIcon.MessageIcon.Critical, 6000)
        self.tray.setToolTip("Whisper Hotkey — load error")
        self._main_win.set_status(f"Failed to load model:\n{err}")

    # ── Hotkey ───────────────────────────────────────────────────────────────

    def _register_hotkey(self):
        try:
            self._hotkey_listener = HotkeyListener(
                self.cfg.hotkey,
                lambda: self._toggle_sig.emit()
            )
            self._hotkey_listener.start()
            self._hotkey_retries = 0
        except Exception as e:
            # Retry silently every 3 seconds — permission may be granted later
            if not hasattr(self, '_hotkey_retries'):
                self._hotkey_retries = 0
            self._hotkey_retries += 1
            QTimer.singleShot(3000, self._register_hotkey)

    def _unregister_hotkey(self):
        if self._hotkey_listener:
            self._hotkey_listener.stop()
            self._hotkey_listener = None

    # ── Toggle (main thread) ─────────────────────────────────────────────────

    def _on_toggle(self):
        if self.model is None:
            self.tray.showMessage("Whisper Hotkey", "Model is still loading...", QSystemTrayIcon.MessageIcon.Information, 2000)
            return
        with self._lock:
            if not self._recording:
                # Save the currently focused app so we can restore it before pasting
                front_app = NSWorkspace.sharedWorkspace().frontmostApplication()
                self._saved_app = front_app
                self._recording = True
                threading.Thread(target=self._start_recording, daemon=True).start()
            else:
                self._recording = False
                self.overlay.show_transcribing()
                threading.Thread(target=self._stop_recording, daemon=True).start()

    # ── Recording threads ─────────────────────────────────────────────────────

    def _start_recording(self):
        self._chunks = []
        self._stream = sd.InputStream(
            samplerate=16000, channels=1, dtype="float32",
            callback=lambda data, *_: self._chunks.append(data.copy())
        )
        self._stream.start()
        self._rec_started_sig.emit()

    def _on_rec_started(self):
        self.tray.setIcon(_make_icon("recording"))
        self.overlay._restore_app = self._saved_app
        self.overlay.show_recording()

    def _stop_recording(self):
        if self._stream:
            self._stream.stop(); self._stream.close(); self._stream = None
        if not self._chunks:
            self._audio_ready_sig.emit(np.array([]))
            return
        audio = np.concatenate(self._chunks, axis=0).flatten()
        self._audio_ready_sig.emit(audio)

    # ── Transcription (main thread) ───────────────────────────────────────────

    def _on_audio_ready(self, audio: np.ndarray):
        self.tray.setIcon(_make_icon("idle"))
        if audio.size == 0:
            self.overlay.show_error("No audio captured")
            return
        self._worker = TranscribeWorker(
            self.model, audio, self.cfg.language,
            post_process=self.cfg.post_process,
            post_process_model=self.cfg.post_process_model,
            vocabulary=self.cfg.vocabulary,
            replacements=self.cfg.replacements,
        )
        self._worker.finished.connect(self._on_transcribed)
        self._worker.failed.connect(lambda e: (setattr(self, '_worker', None), self.overlay.show_error(e[:48])))
        self._worker.start()

    def _on_transcribed(self, text: str):
        self._worker = None
        if not text:
            self.overlay.show_error("Nothing recognized")
            return
        self.history.add(text)
        if self.cfg.auto_paste:
            # Restore focus to the app that was active before recording
            if self._saved_app:
                self._saved_app.activateWithOptions_(0)
            QTimer.singleShot(150, lambda: _type_text_direct(text))
        self.overlay.show_done(text)
        if self._hist_win:
            self._hist_win.refresh()

    # ── Window management ──────────────────────────────────────────────────

    def _show_main(self):
        self._main_win.show()
        self._main_win.raise_()
        self._main_win.activateWindow()

    def _open_history(self):
        if self._hist_win is None:
            self._hist_win = HistoryWindow(self.history, self._is_dark)
            self._hist_win.setStyleSheet(QApplication.instance().styleSheet())
        self._hist_win.show(); self._hist_win.raise_(); self._hist_win.activateWindow()

    def _open_settings(self):
        # Unregister hotkey listener so HotkeyCapture can create its own event tap
        self._unregister_hotkey()
        dlg = SettingsDialog(self.cfg, self.history, is_dark=self._is_dark)
        dlg.setStyleSheet(QApplication.instance().styleSheet())
        self._settings_saved = False
        dlg.saved.connect(lambda cfg: (setattr(self, '_settings_saved', True), self._on_settings_saved(cfg)))
        dlg.exec()
        # Re-register only if settings were NOT saved (saved handler already re-registers)
        if not self._settings_saved:
            QTimer.singleShot(500, self._register_hotkey)

    def _on_settings_saved(self, new_cfg: Config):
        old = self.cfg
        self.cfg = new_cfg
        self.overlay.position = new_cfg.overlay_position
        self.history.limit    = new_cfg.history_limit

        self._apply_theme()

        model_changed = new_cfg.model != old.model

        # Always re-register hotkey to pick up changes immediately
        self._unregister_hotkey()

        if model_changed:
            self.model = None
            self.tray.setIcon(_make_icon("loading"))
            self.tray.setToolTip("Whisper Hotkey — reloading...")
            self._start_loader()
        elif self.model:
            QTimer.singleShot(500, self._register_hotkey)

        # Update main window and tray with new settings
        self._main_win.update_hotkey(new_cfg.hotkey)
        if self.model:
            self._main_win.set_status(f"Ready — model '{new_cfg.model}' loaded\nPress {new_cfg.hotkey} to record")
            self.tray.setToolTip(f"Whisper Hotkey — {new_cfg.model}  ·  {new_cfg.hotkey}")

    def _quit(self):
        self._unregister_hotkey()
        QApplication.quit()

# ──────────────────────────────────────────────────────────────────────────────
# ENTRY POINT
# ──────────────────────────────────────────────────────────────────────────────

def _request_permissions():
    """Request Accessibility and Microphone permissions at startup."""
    # ── Accessibility: use PyObjC to call AXIsProcessTrustedWithOptions ──
    try:
        from Cocoa import NSDictionary
        from ctypes import cdll, c_bool, c_void_p
        import objc
        # Use objc bridge to create the options dict safely
        opts = NSDictionary.dictionaryWithObject_forKey_(True, "AXTrustedCheckOptionPrompt")
        asf = cdll.LoadLibrary('/System/Library/Frameworks/ApplicationServices.framework/ApplicationServices')
        asf.AXIsProcessTrustedWithOptions.restype = c_bool
        asf.AXIsProcessTrustedWithOptions.argtypes = [c_void_p]
        asf.AXIsProcessTrustedWithOptions(objc.pyobjc_id(opts))
    except Exception:
        pass

    # ── Microphone: trigger permission dialog by briefly accessing audio ──
    try:
        import sounddevice as sd
        stream = sd.InputStream(samplerate=16000, channels=1, dtype="float32")
        stream.start()
        stream.stop()
        stream.close()
    except Exception:
        pass


def main():
    _request_permissions()

    app = QApplication(sys.argv)
    app.setQuitOnLastWindowClosed(False)
    app.setApplicationName("Whisper Hotkey")
    app.setWindowIcon(_app_icon())
    cfg = Config.load()
    _is_dark = (cfg.theme == "dark") or (cfg.theme == "auto" and not _macos_is_light())
    app.setStyleSheet(get_style(_is_dark))

    controller = WhisperApp()
    app._controller = controller

    # Reopen main window when app is reactivated (e.g. click Dock icon)
    def on_state_changed(state):
        from PySide6.QtCore import Qt
        if state == Qt.ApplicationState.ApplicationActive:
            # Don't steal focus while recording/transcribing
            if controller._recording or (hasattr(controller, '_worker') and controller._worker is not None):
                return
            controller._show_main()
    app.applicationStateChanged.connect(on_state_changed)

    sys.exit(app.exec())


if __name__ == "__main__":
    main()
