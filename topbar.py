import sys
from PySide6.QtWidgets import QApplication
from app.config import Settings
from app.base import SystemStatusBar

if __name__ == "__main__":
    app = QApplication(sys.argv)
    
    # 1. Initialize Configuration
    current_settings = Settings()
    
    # 2. Initialize Main Window with Config
    window = SystemStatusBar(current_settings)
    
    sys.exit(app.exec())