# app/base.py
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
from .components.profile import ProfileComponent
from .components.systray import SystemTrayComponent

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

        # Calculate Total Window Height (Visual Bar + Top Margin)
        self.total_window_height = self.cfg.bar_height + self.cfg.floating_margin_y

        # Window Flags
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool | Qt.WindowDoesNotAcceptFocus)
        self.setAttribute(Qt.WA_TranslucentBackground)
        
        # Set geometry to cover width, but height includes the top margin gap
        self.setGeometry(0, 0, self.screen_width, self.total_window_height)

        # Layout Container (The Window holds the layout, the layout holds the visual container)
        layout = QHBoxLayout(self)
        
        # --- KEY CHANGE: Add Margins to the Main Layout ---
        # This squeezes the container away from edges
        layout.setContentsMargins(
            self.cfg.floating_margin_x, # Left
            self.cfg.floating_margin_y, # Top
            self.cfg.floating_margin_x, # Right
            0                           # Bottom
        )
        
        self.container = QWidget()
        self.container.setObjectName("MainContainer")
        
        # Inner Layout for items
        inner = QHBoxLayout(self.container)
        inner.setContentsMargins(20, 0, 20, 0)

        # --- INSTANTIATE COMPONENTS ---
        self.comp_profile = ProfileComponent(self.cfg)
        self.comp_clock = ClockComponent(self.cfg, parent=self.container)
        self.comp_tray = SystemTrayComponent(self.cfg)
        self.comp_audio = AudioComponent(self.cfg)
        self.comp_net = NetworkComponent(self.cfg)
        self.comp_bat = BatteryComponent(self.cfg)
        self.comp_settings = SettingsComponent(self.cfg)
        
        # Connect Clickables
        self.comp_profile.clicked.connect(lambda: self.handle_popup(self.comp_profile))
        self.comp_tray.clicked.connect(lambda: self.handle_popup(self.comp_tray))
        self.comp_audio.clicked.connect(lambda: self.handle_popup(self.comp_audio))
        self.comp_net.clicked.connect(lambda: self.handle_popup(self.comp_net))
        self.comp_bat.clicked.connect(lambda: self.handle_popup(self.comp_bat))
        self.comp_settings.clicked.connect(lambda: self.handle_popup(self.comp_settings))

        # --- LEFT SIDE ---
        inner.addWidget(self.comp_profile)
        
        # --- MIDDLE SPACER ---
        inner.addStretch()

        # --- RIGHT SIDE ---
        inner.addWidget(self.comp_tray)
        inner.addWidget(self.comp_audio)
        inner.addWidget(self.comp_net)
        inner.addWidget(self.comp_bat)
        inner.addWidget(self.comp_settings)

        # --- ABSOLUTE CLOCK POSITIONING (CENTERED) ---
        clock_width = 300 
        self.comp_clock.setFixedSize(clock_width, self.cfg.bar_height)
        
        # Calculate visual container width based on margins
        container_width = self.screen_width - (2 * self.cfg.floating_margin_x)
        
        # Center clock relative to the CONTAINER, not screen
        clock_x = (container_width - clock_width) // 2
        self.comp_clock.move(clock_x, 0)
        
        self.comp_clock.raise_()

        layout.addWidget(self.container)
        self.apply_style()
        
        # Animation Setup
        self.anim = QPropertyAnimation(self, b"geometry")
        self.anim.setDuration(self.cfg.anim_duration)
        self.anim.setEasingCurve(QEasingCurve.OutCubic)
        self.show()

    def apply_style(self):
        # Updated CSS for rounded corners on ALL sides
        self.container.setStyleSheet(f"""
            QWidget#MainContainer {{
                background-color: {self.cfg.bg_color};
                border-radius: {self.cfg.border_radius}px;
                border: 1px solid {self.cfg.border_color};
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
        if self.active_popup: 
            self.active_popup.close()
            
        title, content = component.get_popup_content()
        self.active_popup = BasePopupWidget(title, content, self.cfg)
        
        self.active_popup.adjustSize()
        popup_w = self.active_popup.width()
        
        comp_global_pos = component.mapToGlobal(QPoint(0, 0))
        comp_w = component.width()
        
        target_x = comp_global_pos.x() + (comp_w // 2) - (popup_w // 2)
        
        # --- FIX: Remove gap between bar and popup ---
        # The BasePopupWidget has a 10px transparent margin (setContentsMargins)
        # to hold the drop shadow. We subtract that margin so the visual frames touch.
        # We assume common.py uses setContentsMargins(10, 10, 10, 10).
        popup_shadow_margin = 10 
        
        # Visual Bottom of Bar = self.total_window_height
        # Visual Top of Popup = target_y + popup_shadow_margin
        target_y = self.total_window_height - popup_shadow_margin 

        if target_x + popup_w > self.screen_width - 10:
            target_x = self.screen_width - popup_w - 10
        if target_x < 10:
            target_x = 10
            
        self.active_popup.move(target_x, target_y)
        
        self.active_popup.show()
        hwnd_popup = int(self.active_popup.winId())
        win32gui.SetWindowPos(hwnd_popup, win32con.HWND_TOPMOST, 0, 0, 0, 0,
                              win32con.SWP_NOMOVE | win32con.SWP_NOSIZE | win32con.SWP_NOACTIVATE)
        
        self.active_popup.show_animated()

    def force_z_order(self):
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

    def monitor_logic(self):
        if self.is_visible:
            self.force_z_order()

        cursor = QCursor.pos()
        
        # Check if hovering over window (includes the margin gap, which makes interaction easier)
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
        self.force_z_order()
        self.anim.stop()
        self.anim.setStartValue(self.geometry())
        # Slide down to 0 (top of screen)
        self.anim.setEndValue(QRect(0, 0, self.screen_width, self.total_window_height))
        self.anim.start()

    def slide_out(self):
        self.is_visible = False
        self.anim.stop()
        self.anim.setStartValue(self.geometry())
        # Slide up completely off screen (Height of bar + Height of margin)
        self.anim.setEndValue(QRect(0, -self.total_window_height, self.screen_width, self.total_window_height))
        self.anim.start()