from PySide6.QtWidgets import QLabel, QWidget, QVBoxLayout, QPushButton
from PySide6.QtCore import Qt, Signal

class ClickableLabel(QLabel):
    clicked = Signal()
    def __init__(self, text="", parent=None):
        super().__init__(text, parent)
        self.setCursor(Qt.PointingHandCursor)

    def mouseReleaseEvent(self, event):
        if event.button() == Qt.LeftButton:
            self.clicked.emit()

class BasePopupWidget(QWidget):
    def __init__(self, title, info_text, settings):
        super().__init__()
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.Popup)
        self.setAttribute(Qt.WA_TranslucentBackground)
        
        self.setStyleSheet(f"""
            QWidget {{
                background-color: {settings.bg_color.replace('230', '250')}; 
                border: 1px solid {settings.border_color};
                border-radius: 8px;
            }}
            QLabel {{ color: {settings.text_color}; border: none; padding: 5px; }}
            QPushButton {{
                background-color: #0078d4; color: white; border: none;
                padding: 5px; border-radius: 4px;
            }}
            QPushButton:hover {{ background-color: #0099ff; }}
        """)
        
        layout = QVBoxLayout(self)
        layout.addWidget(QLabel(f"<b>{title}</b>"))
        layout.addWidget(QLabel(info_text))
        
        btn = QPushButton("Close")
        btn.clicked.connect(self.close)
        layout.addWidget(btn)