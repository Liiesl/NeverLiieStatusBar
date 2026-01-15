import qtawesome as qta
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QPushButton, 
                               QLabel, QGridLayout)
from PySide6.QtCore import Qt
from .common import (ClickableLabel, ModernSlider, ModernToggle, 
                     TEXT_WHITE, TEXT_SUB)

class SettingsComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("", parent, settings)
        self.settings = settings
        # Using the control center icon
        self.setIcon("fa5s.sliders-h") 

    def get_popup_content(self):
        return "Quick Settings", QuickSettingsUI(self.settings)

class QuickSettingsUI(QWidget):
    def __init__(self, settings):
        super().__init__()
        self.setFixedWidth(280) 
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(20)

        # 1. TOP SECTION: Toggles (Grid)
        grid_layout = QGridLayout()
        grid_layout.setSpacing(10)

        # Row 0: Connectivity
        self.tile_wifi = ModernToggle("Wi-Fi", "Connected", "fa5s.wifi", active=True)
        grid_layout.addWidget(self.tile_wifi, 0, 0)

        self.tile_bt = ModernToggle("Bluetooth", "On", "fa5b.bluetooth-b", active=True)
        grid_layout.addWidget(self.tile_bt, 0, 1)

        # Row 1: Utils
        self.tile_airplane = ModernToggle("Airplane", "Off", "fa5s.plane", active=False)
        grid_layout.addWidget(self.tile_airplane, 1, 0)

        self.tile_saver = ModernToggle("Battery Saver", "Off", "fa5s.leaf", active=False)
        grid_layout.addWidget(self.tile_saver, 1, 1)
        
        layout.addLayout(grid_layout)

        # 2. MIDDLE SECTION: Sliders
        sliders_layout = QVBoxLayout()
        sliders_layout.setSpacing(5) # Tight spacing between label and slider
        
        # --- Brightness Group ---
        lbl_bright = QLabel("Brightness")
        lbl_bright.setStyleSheet(f"color: {TEXT_WHITE}; font-size: 12px; font-weight: 500; border: none; background: transparent;")
        sliders_layout.addWidget(lbl_bright)
        
        sliders_layout.addWidget(ModernSlider("fa5s.sun", 75))
        
        # Spacer
        sliders_layout.addSpacing(10)

        # --- Audio Group (Speaker + Mic) ---
        lbl_audio = QLabel("Audio")
        lbl_audio.setStyleSheet(f"color: {TEXT_WHITE}; font-size: 12px; font-weight: 500; border: none; background: transparent;")
        sliders_layout.addWidget(lbl_audio)
        
        # Speaker
        sliders_layout.addWidget(ModernSlider("fa5s.volume-up", 40))
        # Mic 
        sliders_layout.addWidget(ModernSlider("fa5s.microphone", 80))

        layout.addLayout(sliders_layout)

        # 3. BOTTOM SECTION: Power Utils & Settings
        power_layout = QVBoxLayout()
        power_layout.setSpacing(5)

        # Power Label
        lbl_power = QLabel("Power")
        lbl_power.setStyleSheet(f"color: {TEXT_WHITE}; font-weight: bold; font-size: 14px; margin-bottom: 5px; border: none; background: transparent;")
        power_layout.addWidget(lbl_power)

        # Buttons Row
        footer = QHBoxLayout()
        footer.setContentsMargins(0, 0, 0, 0) 
        footer.setSpacing(5)

        # Helper to create consistent footer buttons
        def create_footer_btn(icon_name, tooltip):
            btn = QPushButton()
            btn.setIcon(qta.icon(icon_name, color=TEXT_WHITE))
            btn.setFixedSize(30, 30)
            btn.setCursor(Qt.PointingHandCursor)
            btn.setToolTip(tooltip)
            btn.setStyleSheet("""
                QPushButton { background: transparent; border-radius: 4px; border: none; }
                QPushButton:hover { background-color: rgba(255, 255, 255, 20); }
            """)
            return btn

        # Power Buttons
        self.btn_lock = create_footer_btn("fa5s.lock", "Lock")
        self.btn_sleep = create_footer_btn("fa5s.moon", "Sleep")
        self.btn_restart = create_footer_btn("fa5s.redo-alt", "Restart")
        self.btn_shutdown = create_footer_btn("fa5s.power-off", "Shutdown")
        self.btn_disable = create_footer_btn("fa5s.eye-slash", "Disable Topbar")
        
        footer.addWidget(self.btn_lock)
        footer.addWidget(self.btn_sleep)
        footer.addWidget(self.btn_restart)
        footer.addWidget(self.btn_shutdown)
        footer.addWidget(self.btn_disable)

        footer.addStretch() # Push Settings to the far right

        # Settings Cog
        self.btn_settings = create_footer_btn("fa5s.cog", "All Settings")
        footer.addWidget(self.btn_settings)

        power_layout.addLayout(footer)
        layout.addLayout(power_layout)