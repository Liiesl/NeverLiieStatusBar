import comtypes
from comtypes import CLSCTX_ALL
from pycaw.pycaw import AudioUtilities, IAudioEndpointVolume
try:
    from pycaw.constants import ERole
except ImportError:
    class ERole:
        eConsole = 0
        eMultimedia = 1
        eCommunications = 2

import time
from PySide6.QtCore import QTimer, QThread, Signal, QObject, Qt
from PySide6.QtWidgets import QWidget, QVBoxLayout, QLabel
from .common import ClickableLabel, ModernSlider, DeviceListItem

# --- HELPER FUNCTION FOR DEVICE NAMES ---
def get_device_name(dev):
    """Extracts Friendly Name from device properties."""
    try:
        val_friendly = None
        val_desc = None
        if hasattr(dev, 'properties') and dev.properties:
            for k, v in dev.properties.items():
                k_str = str(k).upper()
                if "67D146A850E0} 14" in k_str: val_friendly = v
                elif "67D146A850E0} 2" in k_str: val_desc = v
        
        if val_friendly: return val_friendly
        if val_desc: return val_desc
        if hasattr(dev, 'FriendlyName'): return dev.FriendlyName 
        if hasattr(dev, 'friendly_name'): return dev.friendly_name
        return "Generic Audio Device"
    except:
        return "Unknown Device"

def get_icon_for_dev(name, is_input):
    """Returns a suitable material icon based on device name."""
    name = name.lower()
    if is_input:
        return "mdi.microphone"
    if "headphone" in name or "headset" in name or "buds" in name or "airpods" in name:
        return "mdi.headphones"
    if "monitor" in name or "tv" in name:
        return "mdi.monitor"
    return "mdi.speaker"

# --- BACKGROUND WORKER ---
class AudioScanWorker(QThread):
    scan_finished = Signal(list, list) 

    def run(self):
        try:
            comtypes.CoInitialize()
        except: pass

        out_list = []
        in_list = []

        try:
            # 1. RENDER (Output)
            render_devs = AudioUtilities.GetAllDevices(data_flow=0)
            for dev in render_devs:
                try:
                    state = dev.state.value if hasattr(dev.state, 'value') else dev.state
                    if state == 1:
                        out_list.append((get_device_name(dev), dev.id))
                except: continue

            # 2. CAPTURE (Input)
            capture_devs = AudioUtilities.GetAllDevices(data_flow=1)
            for dev in capture_devs:
                try:
                    state = dev.state.value if hasattr(dev.state, 'value') else dev.state
                    if state == 1:
                        in_list.append((get_device_name(dev), dev.id))
                except: continue
        except Exception as e:
            print(f"Background Audio Scan Error: {e}")
        
        out_list.sort(key=lambda x: x[0])
        in_list.sort(key=lambda x: x[0])

        self.scan_finished.emit(out_list, in_list)
        try:
            comtypes.CoUninitialize()
        except: pass

# --- MAIN COMPONENT ---
class AudioComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("Vol: --", parent, settings=settings)
        self.settings = settings
        
        self.spk_interface = None
        self.mic_interface = None
        
        self.output_devices_list = []
        self.input_devices_list = []

        self.current_output_id = None
        self.current_input_id = None
        
        # UI Layout References for live updates
        self.out_list_layout = None
        self.in_list_layout = None
        
        # References to update sliders if device changes
        self.slider_out_ref = None
        self.slider_in_ref = None

        self.scanner = AudioScanWorker()
        self.scanner.scan_finished.connect(self.on_scan_finished)
        
        self.refresh_interfaces()
        self.scanner.start()
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_status)
        self.timer.start(settings.audio_poll_rate)
        self.update_status()

    def refresh_interfaces(self):
        try:
            def_spk = AudioUtilities.GetSpeakers()
            if def_spk:
                self.current_output_id = def_spk.id
                self.spk_interface = self._activate_interface(def_spk)
            
            def_mic = AudioUtilities.GetMicrophone()
            if def_mic:
                self.current_input_id = def_mic.id
                self.mic_interface = self._activate_interface(def_mic)
        except: pass

    def _activate_interface(self, dev):
        try:
            raw_dev = dev._dev if hasattr(dev, '_dev') else dev
            if hasattr(raw_dev, 'Activate'):
                interface = raw_dev.Activate(IAudioEndpointVolume._iid_, CLSCTX_ALL, None)
            else:
                interface = raw_dev.activate(IAudioEndpointVolume._iid_, CLSCTX_ALL, None)
            return interface.QueryInterface(IAudioEndpointVolume)
        except: return None

    def on_scan_finished(self, out_list, in_list):
        self.output_devices_list = out_list
        self.input_devices_list = in_list
        
        # Update UI if open
        if self.out_list_layout:
            self._repopulate_list(self.out_list_layout, self.output_devices_list, self.current_output_id, is_input=False)
        if self.in_list_layout:
            self._repopulate_list(self.in_list_layout, self.input_devices_list, self.current_input_id, is_input=True)

    def _repopulate_list(self, layout, data_list, current_id, is_input):
        # Clear existing items
        while layout.count():
            item = layout.takeAt(0)
            widget = item.widget()
            if widget: widget.deleteLater()
        
        # Add new items
        for name, dev_id in data_list:
            is_active = (dev_id == current_id)
            icon = get_icon_for_dev(name, is_input)
            
            btn = DeviceListItem(name, dev_id, is_active, icon_name=icon)
            
            # Use closure to capture device ID
            # When clicked, update default device
            btn.clicked.connect(lambda checked=False, d_id=dev_id, inp=is_input: self._on_device_selected(d_id, inp))
            
            layout.addWidget(btn)

    def _on_device_selected(self, dev_id, is_input):
        # 1. Set Windows Default
        try:
            roles = [ERole.eConsole, ERole.eMultimedia, ERole.eCommunications]
            AudioUtilities.SetDefaultDevice(dev_id, roles=roles)
        except Exception as e:
            print(f"Failed to switch device: {e}")
            return

        # 2. Update Internal State
        if is_input: self.current_input_id = dev_id
        else: self.current_output_id = dev_id
        
        self.refresh_interfaces()
        
        # 3. Visually update the list (make the clicked one active, others inactive)
        layout = self.in_list_layout if is_input else self.out_list_layout
        if layout:
            for i in range(layout.count()):
                w = layout.itemAt(i).widget()
                if isinstance(w, DeviceListItem):
                    w.setChecked(w.device_id == dev_id)

        # 4. Update Sliders
        target_slider = self.slider_in_ref if is_input else self.slider_out_ref
        interface = self.mic_interface if is_input else self.spk_interface
        
        if target_slider and interface:
            try:
                vol = int(interface.GetMasterVolumeLevelScalar() * 100)
                target_slider.slider.blockSignals(True)
                target_slider.slider.setValue(vol)
                target_slider.slider.blockSignals(False)
                
                muted = interface.GetMute()
                if is_input:
                    icon = "mdi.microphone-off" if muted else "mdi.microphone"
                else:
                    icon = "mdi.volume-off" if muted else "mdi.volume-high"
                target_slider.set_icon(icon)
                
                if not is_input: self.update_status()
            except: pass

    def update_status(self):
        if self.spk_interface:
            try:
                vol = int(self.spk_interface.GetMasterVolumeLevelScalar() * 100)
                muted = self.spk_interface.GetMute()
                
                icon_name = "mdi.volume-high"
                if muted or vol == 0: icon_name = "mdi.volume-off"
                elif vol < 30: icon_name = "mdi.volume-low"
                elif vol < 70: icon_name = "mdi.volume-medium"
                
                self.setIcon(icon_name)
                self.setText(f"{vol}%")
                return
            except: pass
        self.setText("--")

    def set_speaker_volume(self, value):
        if self.spk_interface:
            try:
                scalar = value / 100.0
                self.spk_interface.SetMasterVolumeLevelScalar(scalar, None)
                if self.spk_interface.GetMute() and value > 0:
                    self.spk_interface.SetMute(0, None)
                self.update_status()
            except: pass

    def set_mic_volume(self, value):
        if self.mic_interface:
            try:
                scalar = value / 100.0
                self.mic_interface.SetMasterVolumeLevelScalar(scalar, None)
            except: pass

    def toggle_mute(self, slider_widget, is_input=False):
        interface = self.mic_interface if is_input else self.spk_interface
        if not interface: return
        try:
            current_mute = interface.GetMute()
            new_mute = 0 if current_mute else 1
            interface.SetMute(new_mute, None)
            
            if is_input:
                icon = "mdi.microphone-off" if new_mute else "mdi.microphone"
            else:
                if new_mute: icon = "mdi.volume-off"
                else:
                    vol = int(interface.GetMasterVolumeLevelScalar() * 100)
                    if vol < 30: icon = "mdi.volume-low"
                    elif vol < 70: icon = "mdi.volume-medium"
                    else: icon = "mdi.volume-high"
            slider_widget.set_icon(icon)
            if not is_input: self.update_status()
        except: pass

    def get_popup_content(self):
        if not self.scanner.isRunning():
            self.scanner.start()
        self.refresh_interfaces()

        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(5, 5, 5, 5)
        layout.setSpacing(10)

        # --- OUTPUT ---
        lbl_out = QLabel("Output Device")
        lbl_out.setStyleSheet("color: #cccccc; font-size: 11px; font-weight: bold; margin-bottom: 4px;")
        layout.addWidget(lbl_out)

        # Output List Layout
        self.out_list_layout = QVBoxLayout()
        self.out_list_layout.setSpacing(2)
        layout.addLayout(self.out_list_layout)

        # Initial Populate
        self._repopulate_list(self.out_list_layout, self.output_devices_list, self.current_output_id, is_input=False)

        # Output Slider
        spk_vol = 0
        spk_muted = False
        if self.spk_interface:
            try:
                spk_vol = int(self.spk_interface.GetMasterVolumeLevelScalar() * 100)
                spk_muted = self.spk_interface.GetMute()
            except: pass

        out_icon = "mdi.volume-high"
        if spk_muted or spk_vol == 0: out_icon = "mdi.volume-off"
        elif spk_vol < 30: out_icon = "mdi.volume-low"
        elif spk_vol < 70: out_icon = "mdi.volume-medium"

        self.slider_out_ref = ModernSlider(out_icon, spk_vol, clickable_icon=True)
        self.slider_out_ref.slider.valueChanged.connect(self.set_speaker_volume)
        self.slider_out_ref.icon_clicked.connect(lambda: self.toggle_mute(self.slider_out_ref, is_input=False))
        layout.addWidget(self.slider_out_ref)

        layout.addSpacing(15)

        # --- INPUT ---
        lbl_in = QLabel("Input Device")
        lbl_in.setStyleSheet("color: #cccccc; font-size: 11px; font-weight: bold; margin-bottom: 4px;")
        layout.addWidget(lbl_in)

        self.in_list_layout = QVBoxLayout()
        self.in_list_layout.setSpacing(2)
        layout.addLayout(self.in_list_layout)
        
        # Initial Populate
        self._repopulate_list(self.in_list_layout, self.input_devices_list, self.current_input_id, is_input=True)

        mic_vol = 0
        mic_muted = False
        if self.mic_interface:
            try:
                mic_vol = int(self.mic_interface.GetMasterVolumeLevelScalar() * 100)
                mic_muted = self.mic_interface.GetMute()
            except: pass

        in_icon = "mdi.microphone-off" if mic_muted else "mdi.microphone"
        self.slider_in_ref = ModernSlider(in_icon, mic_vol, clickable_icon=True)
        self.slider_in_ref.slider.valueChanged.connect(self.set_mic_volume)
        self.slider_in_ref.icon_clicked.connect(lambda: self.toggle_mute(self.slider_in_ref, is_input=True))
        layout.addWidget(self.slider_in_ref)

        return "Audio Mixer", container