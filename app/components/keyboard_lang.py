import ctypes
from ctypes import wintypes

from PySide6.QtCore import Qt, QTimer
from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel, QPushButton, QScrollArea

from .common import ClickableLabel

user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32
imm32 = ctypes.windll.imm32

KLF_ACTIVATE = 0x00000001
KLF_SETFORPROCESS = 0x00000100
LOCALE_SLOCALIZEDDISPLAYNAME = 0x00000002

IME_CMODE_NATIVE = 0x0001
IME_CMODE_KATAKANA = 0x0002
IME_CMODE_FULLSHAPE = 0x0008
IME_CMODE_ROMAN = 0x0010

WM_IME_CONTROL = 0x0283
IMC_GETOPENSTATUS = 0x0005

LANG_ID_MAP = {
    0x0401: "AR", 0x0402: "BG", 0x0404: "ZH", 0x0405: "CS",
    0x0406: "DA", 0x0407: "DE", 0x0408: "EL", 0x0409: "EN",
    0x040A: "ES", 0x040B: "FI", 0x040C: "FR", 0x040D: "HE",
    0x040E: "HU", 0x040F: "IS", 0x0410: "IT", 0x0411: "JA",
    0x0412: "KO", 0x0413: "NL", 0x0414: "NO", 0x0415: "PL",
    0x0416: "PT", 0x0418: "RO", 0x0419: "RU", 0x041A: "HR",
    0x041B: "SK", 0x041D: "SV", 0x041E: "TH", 0x041F: "TR",
    0x0422: "UK", 0x0425: "ET", 0x0426: "LV", 0x0427: "LT",
    0x0804: "ZH", 0x0809: "EN", 0x080A: "ES", 0x080C: "FR",
    0x0810: "IT", 0x0813: "NL", 0x0816: "PT", 0x081A: "SR",
    0x0C0A: "ES",
}

IME_MODE_INDICATORS = {
    0x0412: [
        (IME_CMODE_NATIVE, "가"),
    ],
    0x0411: [
        (IME_CMODE_KATAKANA, "カ"),
        (IME_CMODE_NATIVE, "あ"),
    ],
    0x0804: [
        (IME_CMODE_NATIVE, "中"),
    ],
    0x0404: [
        (IME_CMODE_NATIVE, "中"),
    ],
}

IME_OPENSTATUS_LANGS = {0x0412}

non_ime_default = {
    0x0411: "A",
    0x0804: "A",
    0x0404: "A",
}

user32.GetForegroundWindow.argtypes = []
user32.GetForegroundWindow.restype = wintypes.HWND

user32.GetWindowThreadProcessId.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.DWORD)]
user32.GetWindowThreadProcessId.restype = wintypes.DWORD

user32.GetKeyboardLayout.argtypes = [wintypes.DWORD]
user32.GetKeyboardLayout.restype = wintypes.HKL

user32.GetKeyboardLayoutList.argtypes = [wintypes.INT, ctypes.POINTER(wintypes.HKL)]
user32.GetKeyboardLayoutList.restype = wintypes.UINT

user32.ActivateKeyboardLayout.argtypes = [wintypes.HKL, wintypes.UINT]
user32.ActivateKeyboardLayout.restype = wintypes.HKL

kernel32.LCIDToLocaleName.argtypes = [wintypes.DWORD, wintypes.LPWSTR, wintypes.INT, wintypes.DWORD]
kernel32.LCIDToLocaleName.restype = wintypes.INT

kernel32.GetLocaleInfoEx.argtypes = [wintypes.LPWSTR, wintypes.DWORD, wintypes.LPWSTR, wintypes.INT]
kernel32.GetLocaleInfoEx.restype = wintypes.INT

imm32.ImmGetContext.argtypes = [wintypes.HWND]
imm32.ImmGetContext.restype = wintypes.HANDLE

imm32.ImmReleaseContext.argtypes = [wintypes.HWND, wintypes.HANDLE]
imm32.ImmReleaseContext.restype = wintypes.BOOL

imm32.ImmGetDefaultIMEWnd.argtypes = [wintypes.HWND]
imm32.ImmGetDefaultIMEWnd.restype = wintypes.HWND

imm32.ImmGetConversionStatus.argtypes = [wintypes.HANDLE, ctypes.POINTER(wintypes.DWORD), ctypes.POINTER(wintypes.DWORD)]
imm32.ImmGetConversionStatus.restype = wintypes.BOOL

user32.SendMessageW.argtypes = [wintypes.HWND, ctypes.c_uint, wintypes.WPARAM, wintypes.LPARAM]
user32.SendMessageW.restype = ctypes.c_int64


def _get_active_layout_hkl():
    try:
        fg_hwnd = user32.GetForegroundWindow()
        thread_id = user32.GetWindowThreadProcessId(fg_hwnd, None)
        return user32.GetKeyboardLayout(thread_id)
    except Exception:
        return 0


def _get_short_lang_name(hkl):
    if not hkl:
        return "??"
    lang_id = hkl & 0xFFFF
    return LANG_ID_MAP.get(lang_id, f"0x{lang_id:04X}")


def _get_layout_display_name(hkl):
    lang_id = hkl & 0xFFFF
    try:
        locale_name = ctypes.create_unicode_buffer(85)
        kernel32.LCIDToLocaleName(lang_id, locale_name, 85, 0)
        buf = ctypes.create_unicode_buffer(128)
        size = kernel32.GetLocaleInfoEx(locale_name, LOCALE_SLOCALIZEDDISPLAYNAME, buf, 128)
        if size > 0:
            return buf.value
    except Exception:
        pass
    return LANG_ID_MAP.get(lang_id, f"Language 0x{lang_id:04X}")


def _get_all_layouts():
    try:
        count = user32.GetKeyboardLayoutList(0, None)
        if count == 0:
            return []
        buffer = (wintypes.HKL * count)()
        user32.GetKeyboardLayoutList(count, buffer)
        layouts = []
        for hkl in buffer:
            if hkl:
                name = _get_layout_display_name(hkl)
                short = _get_short_lang_name(hkl)
                layouts.append((hkl, name, short))
        return layouts
    except Exception:
        return []


def _switch_layout(hkl):
    try:
        user32.ActivateKeyboardLayout(hkl, KLF_SETFORPROCESS | KLF_ACTIVATE)
    except Exception:
        pass


def _get_ime_indicator(lang_id):
    if lang_id not in IME_MODE_INDICATORS:
        return None
    try:
        fg_hwnd = user32.GetForegroundWindow()

        if lang_id in IME_OPENSTATUS_LANGS:
            ime_hwnd = imm32.ImmGetDefaultIMEWnd(fg_hwnd)
            if not ime_hwnd:
                return None
            is_open = user32.SendMessageW(ime_hwnd, WM_IME_CONTROL, IMC_GETOPENSTATUS, 0)
            if is_open:
                return IME_MODE_INDICATORS[lang_id][0][1]
            return None

        ime_hwnd = imm32.ImmGetDefaultIMEWnd(fg_hwnd)
        if not ime_hwnd:
            return None
        himc = imm32.ImmGetContext(ime_hwnd)
        if not himc:
            return None
        conversion = wintypes.DWORD()
        sentence = wintypes.DWORD()
        ok = imm32.ImmGetConversionStatus(himc, ctypes.byref(conversion), ctypes.byref(sentence))
        imm32.ImmReleaseContext(ime_hwnd, himc)
        if not ok:
            return None
        mode = conversion.value
        for mask, indicator in IME_MODE_INDICATORS.get(lang_id, []):
            if (mode & mask) == mask:
                return indicator
        return non_ime_default.get(lang_id)
    except Exception:
        return None


def _get_bar_display_text(hkl):
    if not hkl:
        return "??"
    short = _get_short_lang_name(hkl)
    lang_id = hkl & 0xFFFF
    indicator = _get_ime_indicator(lang_id)
    if indicator:
        return f"{short} {indicator}"
    return short


class LangPopupWidget(QWidget):
    def __init__(self, current_hkl, parent=None):
        super().__init__(parent)
        self.setFixedSize(180, 300)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(2)

        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setStyleSheet("background: transparent; border: none;")
        self.scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)

        scroll_content = QWidget()
        self.vbox = QVBoxLayout(scroll_content)
        self.vbox.setContentsMargins(0, 0, 0, 0)
        self.vbox.setSpacing(1)
        self.vbox.addStretch()

        self.scroll.setWidget(scroll_content)
        layout.addWidget(self.scroll)

        layouts = _get_all_layouts()
        if not layouts:
            lbl = QLabel("No layouts found")
            lbl.setAlignment(Qt.AlignCenter)
            lbl.setStyleSheet("color: #888; padding: 20px;")
            self.vbox.insertWidget(0, lbl)
            return

        for hkl, name, short in layouts:
            btn = QPushButton(f"  {short}  {name}")
            btn.setCursor(Qt.PointingHandCursor)
            btn.setFixedHeight(32)
            is_active = (hkl == current_hkl)

            active_bg = "#60cdff" if is_active else "transparent"
            active_color = "black" if is_active else "#ffffff"
            hover_bg = "#50b0e0" if is_active else "rgba(255,255,255,0.1)"
            hover_color = "black" if is_active else "#ffffff"

            btn.setStyleSheet(f"""
                QPushButton {{
                    text-align: left;
                    padding-left: 10px;
                    background-color: {active_bg};
                    border: none;
                    border-radius: 4px;
                    color: {active_color};
                    font-family: 'Segoe UI';
                    font-size: 12px;
                }}
                QPushButton:hover {{
                    background-color: {hover_bg};
                    color: {hover_color};
                }}
            """)

            btn.clicked.connect(lambda checked, h=hkl: self._on_select(h))
            self.vbox.insertWidget(self.vbox.count() - 1, btn)

    def _on_select(self, hkl):
        _switch_layout(hkl)
        top = self.window()
        if top and hasattr(top, 'close_animated'):
            top.close_animated()


class KeyboardLangComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("EN", parent, settings=settings)

        self._last_hkl = 0
        self._last_ime_indicator = None
        self._poll_timer = QTimer(self)
        self._poll_timer.timeout.connect(self._poll)
        self._poll_timer.start(settings.lang_poll_rate)
        self._poll()

    def wake_up(self):
        self._poll()
        self._poll_timer.start(self.settings.lang_poll_rate)

    def sleep(self):
        self._poll_timer.stop()

    def _poll(self):
        try:
            hkl = _get_active_layout_hkl()
            if not hkl:
                return

            lang_id = hkl & 0xFFFF
            ime_indicator = _get_ime_indicator(lang_id)

            if hkl != self._last_hkl or ime_indicator != self._last_ime_indicator:
                self._last_hkl = hkl
                self._last_ime_indicator = ime_indicator
                text = _get_bar_display_text(hkl)
                self.setText(text)
        except Exception:
            pass

    def get_popup_content(self):
        current = _get_active_layout_hkl()
        widget = LangPopupWidget(current)
        return "Keyboard Layout", widget
