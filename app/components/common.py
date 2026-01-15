import qtawesome as qta
from PySide6.QtWidgets import (QWidget, QLabel, QPushButton, QVBoxLayout, 
                               QHBoxLayout, QSlider, QFrame, QToolTip, QGraphicsOpacityEffect,
                               QGraphicsDropShadowEffect)
from PySide6.QtCore import Qt, Signal, QPropertyAnimation, QEasingCurve, QPoint, QSize
from PySide6.QtGui import QCursor, QColor

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
        
        # Windows 11 Dark Theme Style
        self.frame.setStyleSheet(f"""
            QFrame#PopupFrame {{
                background-color: #242424; 
                border: 1px solid #454545;
                border-radius: 12px;
            }}
            QLabel {{ color: #ffffff; border: none; font-family: 'Segoe UI'; }}
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