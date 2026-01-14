import psutil
from PySide6.QtCore import QTimer
from .common import ClickableLabel

class BatteryComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("Bat: --", parent)
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_status)
        self.timer.start(settings.battery_poll_rate)
        self.update_status()

    def update_status(self):
        try:
            battery = psutil.sensors_battery()
            if battery:
                plugged = "⚡" if battery.power_plugged else ""
                self.setText(f"{plugged}{int(battery.percent)}%")
        except:
            self.setText("Bat: Err")

    def get_popup_content(self):
        try:
            bat = psutil.sensors_battery()
            secs = bat.secsleft
            time_left = "Plugged In" if secs == psutil.POWER_TIME_UNLIMITED else f"{secs // 60} mins"
            info = f"Level: {bat.percent}%\nStatus: {time_left}"
            return "Power", info
        except:
            return "Power", "Battery info unavailable"