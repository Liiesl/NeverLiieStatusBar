import win32gui
import win32con
from PySide6.QtWidgets import (QApplication, QWidget, QLabel, QHBoxLayout, 
                               QSpacerItem, QSizePolicy)
from PySide6.QtCore import Qt, QTimer, QRect, QEasingCurve, Slot, QPropertyAnimation
from PySide6.QtGui import QCursor

from .components.common import BasePopupWidget
from .components.clock import ClockComponent
from .components.audio import AudioComponent
from .components.network import NetworkComponent
from .components.battery import BatteryComponent

class SystemStatusBar(QWidget):
    def __init__(self, settings):
        super().__init__()
        self.cfg = settings
        
        # State
        self.is_visible = True
        self.active_popup = None
        self.hide_timer = QTimer(self)
        self.hide_timer.setSingleShot(True)
        self.hide_timer.timeout.connect(self.execute_hide)

        self.setup_ui()
        
        # Logic Monitor (Mouse/Focus)
        self.monitor_timer = QTimer(self)
        self.monitor_timer.timeout.connect(self.monitor_logic)
        self.monitor_timer.start(self.cfg.monitor_interval)

    def setup_ui(self):
        self.screen = QApplication.primaryScreen()
        self.screen_width = self.screen.geometry().width()

        self.setWindowFlags(Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool | Qt.WindowDoesNotAcceptFocus)
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setGeometry(0, 0, self.screen_width, self.cfg.bar_height)

        # Layout Container
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        self.container = QWidget()
        self.container.setObjectName("MainContainer")
        inner = QHBoxLayout(self.container)
        inner.setContentsMargins(20, 0, 20, 0)

        # --- INSTANTIATE COMPONENTS ---
        self.comp_clock = ClockComponent(self.cfg)
        self.comp_audio = AudioComponent(self.cfg)
        self.comp_net = NetworkComponent(self.cfg)
        self.comp_bat = BatteryComponent(self.cfg)
        
        # Menu Label (Static)
        lbl_menu = QLabel("⚡ System")
        lbl_menu.setStyleSheet(f"color: {self.cfg.text_color}; font-family: {self.cfg.font_family};")

        # Connect Clickables to Popup Manager
        self.comp_audio.clicked.connect(lambda: self.handle_popup(self.comp_audio))
        self.comp_net.clicked.connect(lambda: self.handle_popup(self.comp_net))
        self.comp_bat.clicked.connect(lambda: self.handle_popup(self.comp_bat))

        # Add to Layout
        inner.addWidget(lbl_menu)
        inner.addSpacerItem(QSpacerItem(40, 20, QSizePolicy.Expanding, QSizePolicy.Minimum))
        inner.addWidget(self.comp_clock)
        inner.addSpacerItem(QSpacerItem(40, 20, QSizePolicy.Expanding, QSizePolicy.Minimum))
        inner.addWidget(self.comp_audio)
        inner.addWidget(self.create_separator())
        inner.addWidget(self.comp_net)
        inner.addWidget(self.create_separator())
        inner.addWidget(self.comp_bat)

        layout.addWidget(self.container)
        self.apply_style()
        
        # Animation Setup
        self.anim = QPropertyAnimation(self, b"geometry")
        self.anim.setDuration(self.cfg.anim_duration)
        self.anim.setEasingCurve(QEasingCurve.OutCubic)
        self.show()

    def create_separator(self):
        sep = QLabel("|")
        sep.setStyleSheet(f"color: {self.cfg.text_color}; padding: 0 5px;")
        return sep

    def apply_style(self):
        self.container.setStyleSheet(f"""
            QWidget#MainContainer {{
                background-color: {self.cfg.bg_color};
                border-bottom-left-radius: {self.cfg.border_radius}px;
                border-bottom-right-radius: {self.cfg.border_radius}px;
                border: 1px solid {self.cfg.border_color};
                border-top: none;
            }}
            ClickableLabel {{
                color: {self.cfg.text_color};
                font-family: '{self.cfg.font_family}';
                font-size: {self.cfg.font_size};
                background: transparent;
                padding: 0 5px;
            }}
            ClickableLabel:hover {{
                color: #ffffff;
                background-color: {self.cfg.hover_bg};
                border-radius: 4px;
            }}
        """)

    def handle_popup(self, component):
        """Standardized handler for component popups"""
        title, content = component.get_popup_content()
        
        if self.active_popup: 
            self.active_popup.close()
            
        self.active_popup = BasePopupWidget(title, content, self.cfg)
        pos = QCursor.pos()
        self.active_popup.move(pos.x() - 100, pos.y() + 20)
        self.active_popup.show()

    # --- AUTO HIDE LOGIC ---
    def monitor_logic(self):
        cursor = QCursor.pos()
        is_hovering = self.geometry().contains(cursor)
        is_at_top_edge = cursor.y() < self.cfg.mouse_trigger_height
        is_popup_open = (self.active_popup and self.active_popup.isVisible())
        
        user_interacting = is_hovering or is_at_top_edge or is_popup_open
        
        # Check if fullscreen app is blocking
        blocked = False
        fg = win32gui.GetForegroundWindow()
        if fg and fg != int(self.winId()):
            if win32gui.GetClassName(fg) not in ["Progman", "WorkerW", "Shell_TrayWnd"]:
                rect = win32gui.GetWindowRect(fg)
                if win32gui.GetWindowPlacement(fg)[1] == win32con.SW_SHOWMAXIMIZED or (rect[1] < self.cfg.bar_height and rect[1] > -32000):
                    blocked = True
        
        should_show = user_interacting or not blocked

        if should_show:
            self.hide_timer.stop()
            if not self.is_visible:
                self.slide_in()
        else:
            if self.is_visible and not self.hide_timer.isActive():
                self.hide_timer.start(self.cfg.auto_hide_delay)

    def execute_hide(self):
        self.slide_out()

    def slide_in(self):
        self.is_visible = True
        self.anim.stop()
        self.anim.setStartValue(self.geometry())
        self.anim.setEndValue(QRect(0, 0, self.screen_width, self.cfg.bar_height))
        self.anim.start()

    def slide_out(self):
        self.is_visible = False
        self.anim.stop()
        self.anim.setStartValue(self.geometry())
        self.anim.setEndValue(QRect(0, -self.cfg.bar_height, self.screen_width, self.cfg.bar_height))
        self.anim.start()