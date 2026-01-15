import psutil
from PySide6.QtCore import QTimer
from .common import ClickableLabel

class BatteryComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("Bat: --", parent, settings=settings)
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_status)
        self.timer.start(settings.battery_poll_rate)
        self.update_status()

    def update_status(self):
        try:
            battery = psutil.sensors_battery()
            if battery:
                percent = int(battery.percent)
                is_plugged = battery.power_plugged
                
                # Determine Icon
                if is_plugged:
                    # MDI Charging icons
                    if percent >= 95: icon = "mdi.battery-charging-100"
                    elif percent >= 80: icon = "mdi.battery-charging-80"
                    elif percent >= 60: icon = "mdi.battery-charging-60"
                    elif percent >= 40: icon = "mdi.battery-charging-40"
                    else: icon = "mdi.battery-charging-20"
                else:
                    # MDI Discharging icons
                    if percent >= 95: icon = "mdi.battery"
                    elif percent >= 80: icon = "mdi.battery-80"
                    elif percent >= 60: icon = "mdi.battery-60"
                    elif percent >= 40: icon = "mdi.battery-40"
                    elif percent >= 20: icon = "mdi.battery-20"
                    else: icon = "mdi.battery-alert" # Critical

                self.setIcon(icon)
                self.setText(f"{percent}%")
        except:
            self.setIcon("mdi.battery-unknown")
            self.setText("Err")

    def get_popup_content(self):
        try:
            bat = psutil.sensors_battery()
            secs = bat.secsleft
            time_left = "Plugged In" if secs == psutil.POWER_TIME_UNLIMITED else f"{secs // 60} mins"
            info = f"Level: {bat.percent}%\nStatus: {time_left}"
            return "Power", info
        except:
            return "Power", "Battery info unavailable"