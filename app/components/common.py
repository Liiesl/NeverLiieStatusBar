import qtawesome as qta
from PySide6.QtWidgets import (QWidget, QLabel, QPushButton, QVBoxLayout, 
                               QHBoxLayout, QSlider, QFrame, QToolTip, QGraphicsOpacityEffect)
from PySide6.QtCore import Qt, Signal, QPropertyAnimation, QEasingCurve, QPoint, QSize
from PySide6.QtGui import QCursor

# --- SHARED COLORS ---
ACCENT_COLOR = "#60cdff"  # Windows 11 Light Blue
BG_DARK = "#242424"
TILE_INACTIVE = "#3e3e3e"
TILE_HOVER = "#4e4e4e"
TEXT_WHITE = "#ffffff"
TEXT_SUB = "#cccccc"

class ClickableLabel(QLabel):
    clicked = Signal()

    def __init__(self, text, parent=None, settings=None):
        super().__init__(text, parent)
        self.settings = settings
        self.setCursor(Qt.PointingHandCursor)
        self.setAlignment(Qt.AlignCenter)

    def mousePressEvent(self, event):
        if event.button() == Qt.LeftButton:
            self.clicked.emit()

    def setIcon(self, icon_name, color=None):
        if not color and self.settings:
            color = self.settings.text_color
        elif not color:
            color = TEXT_WHITE
            
        icon = qta.icon(icon_name, color=color)
        self.setPixmap(icon.pixmap(QSize(20, 20)))

    def get_popup_content(self):
        """Override this in subclasses"""
        return "Info", QLabel("Empty")

class BasePopupWidget(QWidget):
    def __init__(self, title, content_widget, settings):
        super().__init__()
        self.settings = settings
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.Popup | Qt.NoDropShadowWindowHint)
        self.setAttribute(Qt.WA_TranslucentBackground)
        
        # Main Layout
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        # Container (for styling border/bg)
        self.container = QFrame()
        self.container.setStyleSheet(f"""
            QFrame {{
                background-color: {BG_DARK};
                border: 1px solid {settings.border_color};
                border-radius: {settings.border_radius}px;
            }}
        """)
        
        container_layout = QVBoxLayout(self.container)
        container_layout.setContentsMargins(15, 15, 15, 15)
        container_layout.setSpacing(10)
        
        # Header
        header_lbl = QLabel(title)
        header_lbl.setStyleSheet(f"color: {TEXT_WHITE}; font-weight: bold; font-size: 14px; border: none;")
        container_layout.addWidget(header_lbl)
        
        # Content
        container_layout.addWidget(content_widget)
        
        layout.addWidget(self.container)
        
        # Animation Setup
        self.opacity_effect = QGraphicsOpacityEffect(self)
        self.setGraphicsEffect(self.opacity_effect)
        self.anim = QPropertyAnimation(self.opacity_effect, b"opacity")
        self.anim.setDuration(200)
        
    def show_animated(self):
        self.setWindowOpacity(0)
        self.show()
        self.anim.setStartValue(0)
        self.anim.setEndValue(1)
        self.anim.setEasingCurve(QEasingCurve.OutCubic)
        self.anim.start()

    def close_animated(self):
        self.anim.setStartValue(1)
        self.anim.setEndValue(0)
        self.anim.finished.connect(self.close)
        self.anim.start()

# --- SHARED UI CONTROLS ---

class ModernToggle(QWidget):
    """
    Looks like Windows 11 Action Tile:
    [ Icon ] 
    Label
    """
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

class ModernSlider(QWidget):
    """
    Layout: [Icon] [Slider]
    Tooltips appear ABOVE the cursor when sliding.
    """
    def __init__(self, icon_name, value):
        super().__init__()
        
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(15)

        # Icon
        lbl_icon = QLabel()
        if icon_name:
            lbl_icon.setPixmap(qta.icon(icon_name, color=TEXT_SUB).pixmap(QSize(18, 18)))
            lbl_icon.setStyleSheet("border: none; background: transparent;")
            layout.addWidget(lbl_icon)

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
        
        # Connect events for tooltip
        self.slider.valueChanged.connect(self.on_value_change)
        
        layout.addWidget(self.slider)

    def on_value_change(self, val):
        # Calculate position: Current mouse position - offset Y
        cursor_pos = QCursor.pos()
        # Move tooltip 40px up so it sits above the handle/mouse
        tooltip_pos = cursor_pos - QPoint(0, 40)
        
        # Show tooltip
        QToolTip.showText(tooltip_pos, f"{val}%", self.slider)