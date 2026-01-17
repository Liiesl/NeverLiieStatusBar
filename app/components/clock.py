from datetime import datetime
from PySide6.QtWidgets import QLabel
from PySide6.QtCore import QTimer, Qt

class ClockComponent(QLabel):
    def __init__(self, settings, parent=None):
        super().__init__(parent)
        self.setObjectName("ClockLabel")
        
        # Center the text inside the label (important because label has fixed width now)
        self.setAlignment(Qt.AlignCenter)
        
        self.setStyleSheet(f"font-weight: bold; font-size: 14px; padding: 0 10px; color: {settings.text_color};")
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_time)
        self.timer.start(settings.clock_refresh_rate)
        self.update_time()

    def update_time(self):
        self.setText(datetime.now().strftime("%a %b %d   %I:%M %p"))