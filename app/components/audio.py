import comtypes
from comtypes import CLSCTX_ALL, GUID
from pycaw.pycaw import IAudioEndpointVolume, IMMDeviceEnumerator
from PySide6.QtCore import QTimer
from .common import ClickableLabel

class AudioComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("Vol: --", parent)
        self.settings = settings
        self.volume_interface = None
        
        self.setup_audio_interface()
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_status)
        self.timer.start(settings.audio_poll_rate)
        self.update_status()

    def setup_audio_interface(self):
        try:
            comtypes.CoInitialize()
            device_enumerator = comtypes.CoCreateInstance(
                GUID("{BCDE0395-E52F-467C-8E3D-C4579291692E}"),
                IMMDeviceEnumerator, comtypes.CLSCTX_INPROC_SERVER
            )
            audio_device = device_enumerator.GetDefaultAudioEndpoint(0, 1)
            self.volume_interface = audio_device.Activate(
                IAudioEndpointVolume._iid_, CLSCTX_ALL, None
            ).QueryInterface(IAudioEndpointVolume)
        except Exception as e:
            print(f"Audio component error: {e}")

    def update_status(self):
        if self.volume_interface:
            try:
                vol = int(self.volume_interface.GetMasterVolumeLevelScalar() * 100)
                muted = self.volume_interface.GetMute()
                icon = "🔇" if muted else "♪"
                self.setText(f"{icon} {vol}%")
            except:
                pass

    def get_popup_content(self):
        # Return title, body text for the popup
        return "Audio", "Master Volume Control (System)"