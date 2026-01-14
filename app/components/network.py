import subprocess
from PySide6.QtNetwork import QNetworkInformation
from .common import ClickableLabel

class NetworkComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("WiFi: --", parent)
        
        QNetworkInformation.loadBackendByFeatures(QNetworkInformation.Feature.Reachability)
        self.net_info = QNetworkInformation.instance()
        
        if self.net_info:
            self.net_info.reachabilityChanged.connect(self.on_network_change)
            self.on_network_change(self.net_info.reachability())
        else:
            self.setText("Net Err")

    def on_network_change(self, reachability):
        if not self.net_info: return
        reach = self.net_info.reachability()
        if reach == QNetworkInformation.Reachability.Online: self.setText("📶 Online")
        elif reach == QNetworkInformation.Reachability.Site: self.setText("⚠️ Local")
        else: self.setText("🚫 Offline")

    def get_popup_content(self):
        try:
            # Simple parsing of netsh command
            out = subprocess.check_output("netsh wlan show interfaces", shell=True).decode()
            details = "Unknown"
            for line in out.split('\n'):
                if "SSID" in line and "BSSID" not in line: 
                    details = line.strip()
                    break
            return "Network", details
        except:
            return "Network", "Ethernet/Unavailable"