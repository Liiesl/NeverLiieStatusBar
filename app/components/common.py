import qtawesome as qta
from PySide6.QtWidgets import (QWidget, QLabel, QPushButton, QVBoxLayout, 
                               QHBoxLayout, QSlider, QFrame, QToolTip, QGraphicsOpacityEffect,
                               QGraphicsDropShadowEffect, QComboBox, QListView, QSizePolicy, QLineEdit)
from PySide6.QtCore import Qt, Signal, QPropertyAnimation, QEasingCurve, QPoint, QSize
from PySide6.QtGui import QCursor, QColor
import sys

# --- SHARED COLORS ---
ACCENT_COLOR = "#60cdff"  # Windows 11 Light Blue
BG_DARK = "#242424"
TILE_INACTIVE = "#3e3e3e"
TILE_HOVER = "#4e4e4e"
TEXT_WHITE = "#ffffff"
TEXT_SUB = "#cccccc"

class ClickableLabel(QWidget):
    clicked = Signal()

    def __init__(self, text="", parent=None, settings=None):
        super().__init__(parent)
        self.settings = settings
        self.setCursor(Qt.PointingHandCursor)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(5, 0, 5, 0)
        layout.setSpacing(5)

        self.icon_lbl = QLabel()
        self.icon_lbl.setStyleSheet("background: transparent; border: none;")
        self.icon_lbl.setVisible(False)
        
        self.text_lbl = QLabel(text)
        self.text_lbl.setStyleSheet("background: transparent; border: none;")

        layout.addWidget(self.icon_lbl)
        layout.addWidget(self.text_lbl)
        
        if settings:
            self.text_lbl.setStyleSheet(f"""
                color: {settings.text_color};
                font-family: '{settings.font_family}';
                font-size: {settings.font_size};
                background: transparent;
            """)

    def setText(self, text):
        self.text_lbl.setText(text)

    def setIcon(self, icon_name, color=None):
        if not color and self.settings:
            color = self.settings.text_color
        icon = qta.icon(icon_name, color=color)
        pixmap = icon.pixmap(QSize(16, 16)) 
        self.icon_lbl.setPixmap(pixmap)
        self.icon_lbl.setVisible(True)

    def mouseReleaseEvent(self, event):
        if event.button() == Qt.LeftButton:
            self.clicked.emit()

class BasePopupWidget(QWidget):
    def __init__(self, title, content, settings):
        super().__init__()
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.Popup | Qt.NoDropShadowWindowHint)
        self.setAttribute(Qt.WA_TranslucentBackground)
        
        # Start invisible for animation
        self.setWindowOpacity(0.0)

        # Main layout
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(10, 10, 10, 10) # Margin for shadow

        # The visible container
        self.frame = QFrame()
        self.frame.setObjectName("PopupFrame")
        
        # Windows 11 Theme Style (Using Config Settings)
        self.frame.setStyleSheet(f"""
            QFrame#PopupFrame {{
                background-color: {settings.bg_color}; 
                border: 1px solid {settings.border_color};
                border-radius: 12px;
            }}
            QLabel {{ 
                color: {settings.text_color}; 
                border: none; 
                font-family: '{settings.font_family}'; 
            }}
        """)
        
        # Drop Shadow
        shadow = QGraphicsDropShadowEffect(self)
        shadow.setBlurRadius(20)
        shadow.setXOffset(0)
        shadow.setYOffset(4)
        shadow.setColor(QColor(0, 0, 0, 100))
        self.frame.setGraphicsEffect(shadow)
        
        frame_layout = QVBoxLayout(self.frame)
        frame_layout.setContentsMargins(16, 16, 16, 16)
        frame_layout.setSpacing(12)

        if title and title != "Quick Settings": 
            title_lbl = QLabel(f"<b>{title}</b>")
            title_lbl.setAlignment(Qt.AlignCenter)
            frame_layout.addWidget(title_lbl)

        if isinstance(content, str):
            lbl = QLabel(content)
            lbl.setWordWrap(True)
            frame_layout.addWidget(lbl)
        elif isinstance(content, QWidget):
            frame_layout.addWidget(content)

        main_layout.addWidget(self.frame)

        # --- ANIMATIONS ---
        self.anim_opacity = QPropertyAnimation(self, b"windowOpacity")
        self.anim_opacity.setDuration(150)
        self.anim_opacity.setEasingCurve(QEasingCurve.OutCubic)

    def show_animated(self):
        """Fade in and slight slide down"""
        # Ensure we layout first to get correct geometry for positioning
        self.adjustSize() 
        self.show()
        
        self.anim_opacity.stop()
        self.anim_opacity.setStartValue(0.0)
        self.anim_opacity.setEndValue(1.0)
        self.anim_opacity.start()

    def close_animated(self):
        """Fade out then close"""
        self.anim_opacity.stop()
        self.anim_opacity.setStartValue(self.windowOpacity())
        self.anim_opacity.setEndValue(0.0)
        self.anim_opacity.finished.connect(self.close)
        self.anim_opacity.start()

# --- INPUT CONTROLS ---

class ModernInput(QLineEdit):
    def __init__(self, placeholder="Password", parent=None):
        super().__init__(parent)
        self.setPlaceholderText(placeholder)
        self.setEchoMode(QLineEdit.Password)
        self.setStyleSheet(f"""
            QLineEdit {{
                background-color: {TILE_INACTIVE};
                border: 1px solid #555;
                border-radius: 4px;
                padding: 6px;
                color: {TEXT_WHITE};
                selection-background-color: {ACCENT_COLOR};
            }}
            QLineEdit:focus {{
                border: 1px solid {ACCENT_COLOR};
                background-color: {BG_DARK};
            }}
        """)

class WifiListItem(QWidget):
    connect_requested = Signal(object, str) 
    disconnect_requested = Signal(object) 

    def __init__(self, ssid, signal_bars, is_secure, is_connected, network_obj, parent=None):
        super().__init__(parent)
        self.network_obj = network_obj
        self.is_connected = is_connected
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(5)

        # --- HEADER (Always Visible) ---
        self.header = QPushButton()
        self.header.setCheckable(True)
        self.header.setChecked(is_connected)
        self.header.setFixedHeight(45)
        self.header.setCursor(Qt.PointingHandCursor)
        
        icon_name = "mdi.wifi"
        if signal_bars < 2: icon_name = "mdi.wifi-strength-1"
        elif signal_bars < 4: icon_name = "mdi.wifi-strength-2"
        
        if is_secure:
            icon_name += "-lock"
            
        icon_color = ACCENT_COLOR if is_connected else TEXT_WHITE
        self.header.setIcon(qta.icon(icon_name, color=icon_color))
        self.header.setText(f"  {ssid}")
        
        self.header.setStyleSheet(f"""
            QPushButton {{
                text-align: left;
                padding-left: 10px;
                background-color: transparent;
                border: none;
                color: {TEXT_WHITE};
                font-weight: {'bold' if is_connected else 'normal'};
            }}
            QPushButton:hover {{ background-color: {TILE_HOVER}; }}
            QPushButton:checked {{ background-color: {TILE_INACTIVE}; }}
        """)
        self.header.clicked.connect(self.toggle_expanded)
        layout.addWidget(self.header)

        # --- DETAILS ---
        self.details = QFrame()
        self.details.setVisible(False)
        det_layout = QVBoxLayout(self.details)
        det_layout.setContentsMargins(10, 0, 10, 10)
        
        if is_connected:
            btn_disc = QPushButton("Disconnect")
            btn_disc.setCursor(Qt.PointingHandCursor)
            btn_disc.setStyleSheet(f"background-color: {TILE_INACTIVE}; color: white; border: 1px solid #555; border-radius: 4px; padding: 6px;")
            btn_disc.clicked.connect(lambda: self.disconnect_requested.emit(self.network_obj))
            det_layout.addWidget(btn_disc)
        else:            
            # Passing 'parent=self.details' prevents it from becoming a top-level window (popup)
            self.txt_pass = ModernInput(parent=self.details) 
            
            self.txt_pass.setVisible(is_secure)
            
            btn_conn = QPushButton("Connect")
            btn_conn.setCursor(Qt.PointingHandCursor)
            btn_conn.setStyleSheet(f"background-color: {ACCENT_COLOR}; color: black; border-radius: 4px; padding: 6px; font-weight: bold;")
            btn_conn.clicked.connect(self._on_connect)

            det_layout.addWidget(self.txt_pass)
            det_layout.addWidget(btn_conn)

        layout.addWidget(self.details)

    def toggle_expanded(self):
        self.details.setVisible(not self.details.isVisible())
        self.header.setChecked(self.details.isVisible() or self.is_connected)

    def _on_connect(self):
        pwd = self.txt_pass.text()
        self.connect_requested.emit(self.network_obj, pwd)

# --- SHARED UI CONTROLS ---

class ActionTile(QWidget):
    def __init__(self, label_text, sub_text, icon_name, active=False):
        super().__init__()
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(5)

        # The Button Tile
        self.btn = QPushButton()
        self.btn.setCheckable(True)
        self.btn.setChecked(active)
        self.btn.setFixedHeight(50) 
        self.btn.setCursor(Qt.PointingHandCursor)
        
        self.btn.setIcon(qta.icon(icon_name, color="black" if active else "white"))
        self.btn.setIconSize(QSize(20, 20))

        self.btn.setStyleSheet(f"""
            QPushButton {{
                background-color: {TILE_INACTIVE};
                border: 1px solid #555;
                border-radius: 4px;
                text-align: left;
                padding-left: 15px;
            }}
            QPushButton:hover {{
                background-color: {TILE_HOVER};
            }}
            QPushButton:checked {{
                background-color: {ACCENT_COLOR};
                border: 1px solid {ACCENT_COLOR};
            }}
            QPushButton:checked:hover {{
                background-color: #50b0e0;
            }}
        """)

        layout.addWidget(self.btn)

        # The Label Underneath
        lbl = QLabel(label_text)
        lbl.setAlignment(Qt.AlignCenter)
        lbl.setStyleSheet(f"color: {TEXT_WHITE}; font-size: 12px; border: none; background: transparent;")
        layout.addWidget(lbl)

class CompactToggleBtn(QPushButton):
    def __init__(self, icon_name, tooltip_text="", size=36, active=False, parent=None):
        super().__init__(parent)
        self.setCheckable(True)
        self.setChecked(active)
        self.setFixedSize(size, size)
        self.setCursor(Qt.PointingHandCursor)
        self.setToolTip(tooltip_text)
        
        # Set Icon
        self.icon_active = qta.icon(icon_name, color="black")
        self.icon_inactive = qta.icon(icon_name, color="white")
        self.setIcon(self.icon_active if active else self.icon_inactive)
        self.setIconSize(QSize(18, 18))

        # Handle icon color switching manually or via stylesheet logic
        self.toggled.connect(self._update_icon)

        self.setStyleSheet(f"""
            QPushButton {{
                background-color: {TILE_INACTIVE};
                border: 1px solid #555;
                border-radius: {size//2 - 4}px; 
            }}
            QPushButton:hover {{
                background-color: {TILE_HOVER};
            }}
            QPushButton:checked {{
                background-color: {ACCENT_COLOR};
                border: 1px solid {ACCENT_COLOR};
            }}
            QPushButton:checked:hover {{
                background-color: #50b0e0;
            }}
        """)

    def _update_icon(self, checked):
        if checked:
            self.setIcon(self.icon_active)
        else:
            self.setIcon(self.icon_inactive)

class ModernSlider(QWidget):
    icon_clicked = Signal()

    def __init__(self, icon_name, value, clickable_icon=False):
        super().__init__()
        
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(10)

        # Icon Button (Toggle)
        self.icon_btn = QPushButton()
        self.icon_btn.setFixedSize(32, 32)
        
        if clickable_icon:
            self.icon_btn.setCursor(Qt.PointingHandCursor)
            self.icon_btn.clicked.connect(self.icon_clicked.emit)
            self.icon_btn.setStyleSheet("""
                QPushButton {
                    background: transparent;
                    border: none;
                    border-radius: 4px;
                }
                QPushButton:hover {
                    background-color: rgba(255, 255, 255, 0.1);
                }
            """)
        else:
            self.icon_btn.setCursor(Qt.ArrowCursor)
            self.icon_btn.setAttribute(Qt.WA_TransparentForMouseEvents)
            self.icon_btn.setFocusPolicy(Qt.NoFocus)
            self.icon_btn.setStyleSheet("background: transparent; border: none;")
        
        if icon_name:
            self.set_icon(icon_name)
            
        layout.addWidget(self.icon_btn)

        # Slider
        self.slider = QSlider(Qt.Horizontal)
        self.slider.setValue(value)
        self.slider.setCursor(Qt.PointingHandCursor)
        self.slider.setRange(0, 100)
        
        # Thick Windows 11 Style Slider
        self.slider.setStyleSheet(f"""
            QSlider::groove:horizontal {{
                border-radius: 2px;
                height: 4px;
                margin: 0px;
                background-color: #868686;
            }}
            QSlider::groove:horizontal:hover {{
                background-color: #999999;
            }}
            QSlider::handle:horizontal {{
                background-color: #ffffff;
                border: none;
                height: 16px;
                width: 16px;
                margin: -6px 0;
                border-radius: 8px;
            }}
            QSlider::handle:horizontal:hover {{
                height: 18px;
                width: 18px;
                margin: -7px 0;
                border-radius: 9px;
            }}
            QSlider::sub-page:horizontal {{
                background: {ACCENT_COLOR};
                border-radius: 2px;
            }}
        """)
        
        self.slider.valueChanged.connect(self.on_value_change)
        layout.addWidget(self.slider)

    def set_icon(self, icon_name, color=TEXT_SUB):
        self.icon_btn.setIcon(qta.icon(icon_name, color=color))
        self.icon_btn.setIconSize(QSize(20, 20))

    def on_value_change(self, val):
        cursor_pos = QCursor.pos()
        tooltip_pos = cursor_pos - QPoint(0, 40)
        QToolTip.showText(tooltip_pos, f"{val}%", self.slider)

class ModernComboBox(QComboBox):
    """
    Styled QComboBox for dark theme. (Kept for compatibility, though used less now)
    """
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setCursor(Qt.PointingHandCursor)
        self.setView(QListView(self)) 
        
        self.setStyleSheet(f"""
            QComboBox {{
                background-color: {TILE_INACTIVE};
                border: 1px solid #555;
                border-radius: 4px;
                padding: 4px 10px;
                color: {TEXT_WHITE};
                font-family: 'Segoe UI';
                font-size: 12px;
                min-height: 20px;
            }}
            QComboBox:hover {{
                background-color: {TILE_HOVER};
                border-color: #666;
            }}
            QComboBox::drop-down {{
                subcontrol-origin: padding;
                subcontrol-position: top right;
                width: 25px;
                border-left-width: 0px;
                border-top-right-radius: 4px;
                border-bottom-right-radius: 4px;
            }}
            QComboBox::down-arrow {{
                border: none;
                background: transparent;
                width: 0px; height: 0px;
            }}
            QComboBox QAbstractItemView {{
                background-color: {BG_DARK};
                border: 1px solid #454545;
                selection-background-color: {ACCENT_COLOR};
                selection-color: black;
                color: {TEXT_WHITE};
                outline: none;
                border-radius: 4px;
            }}
        """)

class DeviceListItem(QPushButton):
    """
    A list item that functions like a Radio Button.
    Displays an icon, text, and visual highlights when selected.
    """
    def __init__(self, name, device_id, is_active, icon_name="mdi.speaker", parent=None):
        super().__init__(parent)
        self.device_id = device_id
        self.setText(name)
        self.setCheckable(True)
        self.setChecked(is_active)
        self.setCursor(Qt.PointingHandCursor)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Fixed)
        
        # Set icon
        # Active items usually have white icons in this theme, or accent color. 
        # We stick to white/grey for consistency.
        self.setIcon(qta.icon(icon_name, color=TEXT_WHITE))
        self.setIconSize(QSize(20, 20))

        # Windows 11-style Radio List Item styling
        # "Left Border" simulates the accent color pill.
        self.setStyleSheet(f"""
            QPushButton {{
                text-align: left;
                padding: 10px 12px;
                background-color: transparent;
                border: none;
                border-radius: 4px;
                color: {TEXT_WHITE};
                font-family: 'Segoe UI';
                font-size: 12px;
                border-left: 3px solid transparent; /* Reserve space for accent */
            }}
            QPushButton:hover {{
                background-color: {TILE_HOVER};
            }}
            QPushButton:checked {{
                background-color: {TILE_INACTIVE};
                border-left: 3px solid {ACCENT_COLOR}; /* The Accent Pill */
                font-weight: bold;
            }}
            QPushButton:checked:hover {{
                background-color: #454545;
            }}
        """)