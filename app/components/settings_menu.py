import qtawesome as qta
import os
import sys
import ctypes
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QPushButton, 
                               QLabel, QGridLayout, QApplication)
from PySide6.QtCore import Qt
from .common import (ClickableLabel, ModernSlider, ActionTile, CompactToggleBtn,
                     ConfirmationDialog, TEXT_WHITE, TEXT_SUB)

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

        # 1. TOP SECTION: Action Tiles (Grid)
        grid_layout = QGridLayout()
        grid_layout.setSpacing(10)

        # Row 0: Connectivity
        self.tile_wifi = ActionTile("Wi-Fi", "Connected", "fa5s.wifi", active=True)
        grid_layout.addWidget(self.tile_wifi, 0, 0)

        self.tile_bt = ActionTile("Bluetooth", "On", "fa5b.bluetooth-b", active=True)
        grid_layout.addWidget(self.tile_bt, 0, 1)

        # Row 1: Utils
        self.tile_airplane = ActionTile("Airplane", "Off", "fa5s.plane", active=False)
        grid_layout.addWidget(self.tile_airplane, 1, 0)

        self.tile_saver = ActionTile("Battery Saver", "Off", "fa5s.leaf", active=False)
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

        # Power Buttons (Triggers)
        self.btn_lock = create_footer_btn("fa5s.lock", "Lock")
        self.btn_sleep = create_footer_btn("fa5s.moon", "Sleep")
        self.btn_restart = create_footer_btn("fa5s.redo-alt", "Restart")
        self.btn_shutdown = create_footer_btn("fa5s.power-off", "Shutdown")
        
        # Changed from CompactToggleBtn to normal button using helper
        self.btn_disable = create_footer_btn("fa5s.eye-slash", "Disable Topbar (Quit)")
        
        # Connections with confirmations
        self.btn_lock.clicked.connect(lambda: self.confirm_action("Lock"))
        self.btn_sleep.clicked.connect(lambda: self.confirm_action("Sleep"))
        self.btn_restart.clicked.connect(lambda: self.confirm_action("Restart"))
        self.btn_shutdown.clicked.connect(lambda: self.confirm_action("Shutdown"))
        self.btn_disable.clicked.connect(lambda: self.confirm_action("Disable"))

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

    def confirm_action(self, action_type):
        """Shows a confirmation dialog before executing a power action."""
        msg_map = {
            "Lock": "Are you sure you want to lock the computer?",
            "Sleep": "Are you sure you want to put the computer to sleep?",
            "Restart": "Are you sure you want to restart the computer?",
            "Shutdown": "Are you sure you want to shut down the computer?",
            "Disable": "Are you sure you want to quit the Topbar application?"
        }
        
        dialog = ConfirmationDialog(action_type, msg_map.get(action_type, "Confirm action?"), self)
        if dialog.exec():
            self.execute_power_action(action_type)

    def execute_power_action(self, action_type):
        """Executes the actual system command."""
        try:
            if action_type == "Lock":
                ctypes.windll.user32.LockWorkStation()
            
            elif action_type == "Sleep":
                # Standard command to trigger sleep
                os.system("rundll32.exe powrprof.dll,SetSuspendState 0,1,0")
            
            elif action_type == "Restart":
                # /r = restart, /t 0 = immediately
                os.system("shutdown /r /t 0")
            
            elif action_type == "Shutdown":
                # /s = shutdown, /t 0 = immediately
                os.system("shutdown /s /t 0")
            
            elif action_type == "Disable":
                # Quit the application
                QApplication.instance().quit()
                
        except Exception as e:
            print(f"Error executing {action_type}: {e}")