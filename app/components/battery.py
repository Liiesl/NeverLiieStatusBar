# app/components/battery.py

import psutil
import subprocess
import re
import os
from PySide6.QtCore import QTimer, QThread, Signal, Qt, QSize, QUrl
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                               QPushButton, QProgressBar, QFrame, QSizePolicy)
from PySide6.QtGui import QDesktopServices
import qtawesome as qta
from .common import ClickableLabel, DeviceListItem, ACCENT_COLOR, TILE_INACTIVE, TILE_HOVER

# --- POWER PLAN WORKER ---
class PowerPlanWorker(QThread):
    plans_fetched = Signal(list, str) # [(name, guid)], active_guid

    def run(self):
        try:
            # 1. Get List
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            
            output = subprocess.check_output("powercfg /list", startupinfo=startupinfo).decode("utf-8", errors="ignore")
            
            plans = []
            active_guid = None
            
            # Parse output like: "Power Scheme GUID: 381b4222...  (Balanced) *"
            lines = output.split('\n')
            for line in lines:
                if "GUID" in line:
                    parts = line.split()
                    guid = parts[3]
                    # Extract name inside parentheses
                    match = re.search(r'\((.*?)\)', line)
                    name = match.group(1) if match else "Unknown"
                    
                    plans.append((name, guid))
                    
                    if "*" in line:
                        active_guid = guid
            
            self.plans_fetched.emit(plans, active_guid)

        except Exception as e:
            print(f"Power Plan Scan Error: {e}")

    def set_plan(self, guid):
        try:
            startupinfo = subprocess.STARTUPINFO()
            startupinfo.dwFlags |= subprocess.STARTF_USESHOWWINDOW
            subprocess.run(f"powercfg /setactive {guid}", startupinfo=startupinfo)
        except: pass

# --- POPUP WIDGET ---
class BatteryPopupWidget(QWidget):
    def __init__(self, plan_worker):
        super().__init__()
        self.plan_worker = plan_worker
        self.setFixedWidth(300)
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(5, 5, 5, 5)
        layout.setSpacing(10)

        # 1. Header Info (Icon + Text)
        self.info_layout = QHBoxLayout()
        self.icon_lbl = QLabel()
        self.icon_lbl.setFixedSize(48, 48)
        self.icon_lbl.setAlignment(Qt.AlignCenter)
        
        text_layout = QVBoxLayout()
        text_layout.setSpacing(2)
        self.lbl_percent = QLabel("--%")
        self.lbl_percent.setStyleSheet("font-size: 20px; font-weight: bold; color: white;")
        self.lbl_status = QLabel("Calculating...")
        self.lbl_status.setStyleSheet("font-size: 12px; color: #aaaaaa;")
        
        text_layout.addWidget(self.lbl_percent)
        text_layout.addWidget(self.lbl_status)
        
        self.info_layout.addWidget(self.icon_lbl)
        self.info_layout.addLayout(text_layout)
        self.info_layout.addStretch()
        layout.addLayout(self.info_layout)

        # 2. Visual Progress Bar
        self.progress = QProgressBar()
        self.progress.setFixedHeight(6)
        self.progress.setTextVisible(False)
        self.progress.setStyleSheet(f"""
            QProgressBar {{
                background-color: {TILE_INACTIVE};
                border-radius: 3px;
                border: none;
            }}
            QProgressBar::chunk {{
                background-color: {ACCENT_COLOR};
                border-radius: 3px;
            }}
        """)
        layout.addWidget(self.progress)

        # Divider
        line = QFrame()
        line.setFrameShape(QFrame.HLine)
        line.setStyleSheet("background-color: #3e3e3e; max-height: 1px;")
        layout.addWidget(line)

        # 3. Power Plans List
        lbl_plans = QLabel("Power Mode")
        lbl_plans.setStyleSheet("color: #cccccc; font-size: 11px; font-weight: bold;")
        layout.addWidget(lbl_plans)

        self.plans_layout = QVBoxLayout()
        self.plans_layout.setSpacing(2)
        layout.addLayout(self.plans_layout)

        # 4. Footer (Settings Button)
        self.btn_settings = QPushButton("Battery Settings")
        self.btn_settings.setCursor(Qt.PointingHandCursor)
        self.btn_settings.clicked.connect(lambda: QDesktopServices.openUrl(QUrl("ms-settings:batterysaver")))
        self.btn_settings.setIcon(qta.icon("mdi.cog", color="white"))
        self.btn_settings.setStyleSheet(f"""
            QPushButton {{
                background-color: {TILE_INACTIVE};
                color: white;
                border: none;
                border-radius: 4px;
                padding: 8px;
                text-align: left;
            }}
            QPushButton:hover {{ background-color: {TILE_HOVER}; }}
        """)
        layout.addWidget(self.btn_settings)

        # Initialize
        self.update_battery_info()
        
        # Connect worker
        self.plan_worker.plans_fetched.connect(self.populate_plans)
        self.plan_worker.start()

    def update_battery_info(self):
        try:
            bat = psutil.sensors_battery()
            if not bat: return

            percent = int(bat.percent)
            plugged = bat.power_plugged
            
            # Text
            self.lbl_percent.setText(f"{percent}%")
            self.progress.setValue(percent)

            # Status Text & Icon
            if plugged:
                status_txt = "Charging"
                if percent >= 99: status_txt = "Fully Charged"
                
                # Dynamic Charging Icon
                if percent > 90: icon = "mdi.battery-charging-100"
                elif percent > 60: icon = "mdi.battery-charging-60"
                elif percent > 30: icon = "mdi.battery-charging-40"
                else: icon = "mdi.battery-charging-20"
                
            else:
                secs = bat.secsleft
                if secs == psutil.POWER_TIME_UNLIMITED:
                    status_txt = "On Battery"
                elif secs == psutil.POWER_TIME_UNKNOWN:
                    status_txt = "Estimating..."
                else:
                    hrs = secs // 3600
                    mins = (secs % 3600) // 60
                    status_txt = f"{hrs} hr {mins} min remaining"

                # Dynamic Discharging Icon
                if percent > 90: icon = "mdi.battery"
                elif percent > 60: icon = "mdi.battery-60"
                elif percent > 30: icon = "mdi.battery-40"
                elif percent > 15: icon = "mdi.battery-20"
                else: icon = "mdi.battery-alert"
            
            # Change color if low
            if not plugged and percent < 20:
                self.progress.setStyleSheet(self.progress.styleSheet().replace(ACCENT_COLOR, "#ff4444"))
                icon_color = "#ff4444"
            else:
                self.progress.setStyleSheet(self.progress.styleSheet().replace("#ff4444", ACCENT_COLOR))
                icon_color = ACCENT_COLOR if plugged else "white"

            self.lbl_status.setText(status_txt)
            
            # Set Icon
            pix = qta.icon(icon, color=icon_color).pixmap(QSize(40, 40))
            self.icon_lbl.setPixmap(pix)

        except Exception as e:
            print(e)

    def populate_plans(self, plans, active_guid):
        # Clear existing
        while self.plans_layout.count():
            child = self.plans_layout.takeAt(0)
            if child.widget(): child.widget().deleteLater()

        if not plans:
            lbl = QLabel("No power plans available")
            lbl.setStyleSheet("color: #666; padding: 5px;")
            self.plans_layout.addWidget(lbl)
            return

        for name, guid in plans:
            is_active = (guid == active_guid)
            
            # Determine icon based on name
            icon = "mdi.lightning-bolt" # Default (High perf)
            if "balanced" in name.lower(): icon = "mdi.scale-balance"
            elif "save" in name.lower() or "eco" in name.lower(): icon = "mdi.leaf"

            # Use common.py DeviceListItem for the radio-button look
            btn = DeviceListItem(name, guid, is_active, icon_name=icon)
            btn.clicked.connect(lambda checked=False, g=guid: self._switch_plan(g))
            
            self.plans_layout.addWidget(btn)

    def _switch_plan(self, guid):
        self.plan_worker.set_plan(guid)
        
        # Visually update immediately
        for i in range(self.plans_layout.count()):
            w = self.plans_layout.itemAt(i).widget()
            if isinstance(w, DeviceListItem):
                w.setChecked(w.device_id == guid)

# --- MAIN COMPONENT ---
class BatteryComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("Bat: --", parent, settings=settings)
        self.settings = settings
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_status)
        
        # Keep a worker instance ready for popup interactions
        self.plan_worker = PowerPlanWorker()
        
        # Initial poll
        self.update_status()
        self.timer.start(settings.battery_poll_rate)

    # --- OPTIMIZATION HOOKS ---
    def wake_up(self):
        """Called by base.py when bar is visible."""
        self.update_status() # Refresh immediately
        if not self.timer.isActive():
            self.timer.start(self.settings.battery_poll_rate)

    def sleep(self):
        """Called by base.py when bar hides."""
        self.timer.stop()
    # --------------------------

    def update_status(self):
        try:
            battery = psutil.sensors_battery()
            if battery:
                percent = int(battery.percent)
                is_plugged = battery.power_plugged
                
                # Determine Icon for Taskbar
                if is_plugged:
                    if percent >= 95: icon = "mdi.battery-charging-100"
                    elif percent >= 80: icon = "mdi.battery-charging-80"
                    elif percent >= 60: icon = "mdi.battery-charging-60"
                    elif percent >= 40: icon = "mdi.battery-charging-40"
                    else: icon = "mdi.battery-charging-20"
                else:
                    if percent >= 95: icon = "mdi.battery"
                    elif percent >= 80: icon = "mdi.battery-80"
                    elif percent >= 60: icon = "mdi.battery-60"
                    elif percent >= 40: icon = "mdi.battery-40"
                    elif percent >= 20: icon = "mdi.battery-20"
                    else: icon = "mdi.battery-alert"

                self.setIcon(icon)
                self.setText(f"{percent}%")
        except:
            self.setIcon("mdi.battery-unknown")
            self.setText("Err")

    def get_popup_content(self):
        # We probably want to ensure the worker is ready, but it doesn't run continuously
        widget = BatteryPopupWidget(self.plan_worker)
        return "Power", widget