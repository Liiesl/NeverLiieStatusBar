import subprocess
from PySide6.QtNetwork import QNetworkInformation
from .common import ClickableLabel

class NetworkComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("WiFi: --", parent, settings=settings)
        
        QNetworkInformation.loadBackendByFeatures(QNetworkInformation.Feature.Reachability)
        self.net_info = QNetworkInformation.instance()
        
        if self.net_info:
            self.net_info.reachabilityChanged.connect(self.on_network_change)
            self.on_network_change(self.net_info.reachability())
        else:
            self.setIcon("mdi.wifi-strength-off-outline")
            self.setText("Net Err")

    def on_network_change(self, reachability):
        if not self.net_info: return
        reach = self.net_info.reachability()
        
        if reach == QNetworkInformation.Reachability.Online: 
            self.setIcon("mdi.wifi")
            self.setText("Online")
        elif reach == QNetworkInformation.Reachability.Site: 
            self.setIcon("mdi.wifi-alert")
            self.setText("Local")
        else: 
            self.setIcon("mdi.wifi-off")
            self.setText("Offline")

    def get_popup_content(self):
        try:
            out = subprocess.check_output("netsh wlan show interfaces", shell=True).decode()
            details = "Unknown"
            for line in out.split('\n'):
                if "SSID" in line and "BSSID" not in line: 
                    details = line.strip()
                    break
            return "Network", details
        except:
            return "Network", "Ethernet/Unavailable"