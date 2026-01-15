import win32gui
import win32con
import time
from PySide6.QtWidgets import (QApplication, QWidget, QLabel, QHBoxLayout, 
                               QSpacerItem, QSizePolicy)
from PySide6.QtCore import Qt, QTimer, QRect, QEasingCurve, QPropertyAnimation, QPoint
from PySide6.QtGui import QCursor

from .components.common import BasePopupWidget
from .components.clock import ClockComponent
from .components.audio import AudioComponent
from .components.network import NetworkComponent
from .components.battery import BatteryComponent
from .components.settings_menu import SettingsComponent

class SystemStatusBar(QWidget):
    def __init__(self, settings):
        super().__init__()
        self.cfg = settings
        
        # State
        self.is_visible = True
        self.active_popup = None
        self.edge_dwell_start = None  
        
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

        # Added Tool and WindowDoesNotAcceptFocus to help with overlay behavior
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
        self.comp_settings = SettingsComponent(self.cfg)
        
        # Menu Label (Static)
        lbl_menu = QLabel("⚡ System")
        lbl_menu.setStyleSheet(f"color: {self.cfg.text_color}; font-family: {self.cfg.font_family};")

        # Connect Clickables to Popup Manager
        self.comp_audio.clicked.connect(lambda: self.handle_popup(self.comp_audio))
        self.comp_net.clicked.connect(lambda: self.handle_popup(self.comp_net))
        self.comp_bat.clicked.connect(lambda: self.handle_popup(self.comp_bat))
        self.comp_settings.clicked.connect(lambda: self.handle_popup(self.comp_settings))

        # Add to Layout
        inner.addWidget(lbl_menu)
        inner.addSpacerItem(QSpacerItem(40, 20, QSizePolicy.Expanding, QSizePolicy.Minimum))
        inner.addWidget(self.comp_clock)
        inner.addSpacerItem(QSpacerItem(40, 20, QSizePolicy.Expanding, QSizePolicy.Minimum))
        inner.addWidget(self.comp_audio)
        inner.addWidget(self.comp_net)
        inner.addWidget(self.comp_bat)
        inner.addWidget(self.comp_settings)

        layout.addWidget(self.container)
        self.apply_style()
        
        # Animation Setup
        self.anim = QPropertyAnimation(self, b"geometry")
        self.anim.setDuration(self.cfg.anim_duration)
        self.anim.setEasingCurve(QEasingCurve.OutCubic)
        self.show()

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
        """Calculates precise position relative to component and animates."""
        if self.active_popup: 
            self.active_popup.close()
            
        title, content = component.get_popup_content()
        self.active_popup = BasePopupWidget(title, content, self.cfg)
        
        # 1. Force layout to calculate size
        self.active_popup.adjustSize()
        popup_w = self.active_popup.width()
        
        # 2. Get Component Global Position
        comp_global_pos = component.mapToGlobal(QPoint(0, 0))
        comp_w = component.width()
        
        # 3. Calculate Center X
        target_x = comp_global_pos.x() + (comp_w // 2) - (popup_w // 2)
        
        # 4. Y Position
        target_y = self.cfg.bar_height + 5 

        # 5. Screen Bounds Check
        if target_x + popup_w > self.screen_width - 10:
            target_x = self.screen_width - popup_w - 10
        if target_x < 10:
            target_x = 10
            
        self.active_popup.move(target_x, target_y)
        
        # Force popup on top as well
        self.active_popup.show()
        # Apply the win32 Z-order fix to the popup too
        hwnd_popup = int(self.active_popup.winId())
        win32gui.SetWindowPos(hwnd_popup, win32con.HWND_TOPMOST, 0, 0, 0, 0,
                              win32con.SWP_NOMOVE | win32con.SWP_NOSIZE | win32con.SWP_NOACTIVATE)
        
        self.active_popup.show_animated()

    def force_z_order(self):
        """
        Forcefully reapplies the HWND_TOPMOST flag. 
        Windows likes to remove this flag when returning from sleep/lock.
        """
        try:
            hwnd = int(self.winId())
            win32gui.SetWindowPos(
                hwnd, 
                win32con.HWND_TOPMOST, 
                0, 0, 0, 0, 
                win32con.SWP_NOMOVE | win32con.SWP_NOSIZE | win32con.SWP_NOACTIVATE
            )
        except Exception:
            pass

    # --- AUTO HIDE LOGIC ---
    def monitor_logic(self):
        # 1. FIX: Enforce Always on Top if visible
        if self.is_visible:
            self.force_z_order()

        cursor = QCursor.pos()
        
        is_hovering = self.geometry().contains(cursor)
        is_at_top_edge = cursor.y() < self.cfg.mouse_trigger_height
        is_popup_open = (self.active_popup and self.active_popup.isVisible())
        
        trigger_activated = False
        
        if is_at_top_edge:
            if self.is_visible:
                trigger_activated = True
                self.edge_dwell_start = None
            else:
                if self.edge_dwell_start is None:
                    self.edge_dwell_start = time.time()
                
                elapsed_ms = (time.time() - self.edge_dwell_start) * 1000
                if elapsed_ms >= self.cfg.trigger_dwell_time:
                    trigger_activated = True
        else:
            self.edge_dwell_start = None

        user_interacting = is_hovering or trigger_activated or is_popup_open
        
        # Check if fullscreen app is blocking
        blocked = False
        fg = win32gui.GetForegroundWindow()
        if fg and fg != int(self.winId()):
            if win32gui.GetClassName(fg) not in ["Progman", "WorkerW", "Shell_TrayWnd"]:
                rect = win32gui.GetWindowRect(fg)
                if (rect[3] - rect[1]) >= self.screen.geometry().height():
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
        if self.active_popup and self.active_popup.isVisible():
            self.active_popup.close_animated()

    def slide_in(self):
        self.is_visible = True
        self.force_z_order() # Ensure it's on top immediately upon sliding in
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