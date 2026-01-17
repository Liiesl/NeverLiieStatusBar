import time
import io
from PySide6.QtCore import QTimer, QThread, Signal, QObject, Qt, QSize, QByteArray, QBuffer, QIODevice
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QHBoxLayout, QLabel, 
                               QPushButton, QFrame, QSizePolicy)
from PySide6.QtGui import QPixmap, QImage, QPainter, QBrush, QColor, QPainterPath

import qtawesome as qta
# Import consolidated API
from .. import winapiref as wa
from .common import ClickableLabel, ModernSlider, DeviceListItem, ACCENT_COLOR, TILE_INACTIVE, TILE_HOVER

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

# --- AUDIO SCAN WORKER ---
class AudioScanWorker(QThread):
    scan_finished = Signal(list, list) 

    def run(self):
        if not wa.COM_AVAILABLE:
            self.scan_finished.emit([], [])
            return

        try:
            wa.comtypes.CoInitialize()
        except: pass

        out_list = []
        in_list = []

        try:
            # 1. RENDER (Output)
            render_devs = wa.AudioUtilities.GetAllDevices(data_flow=0)
            for dev in render_devs:
                try:
                    state = dev.state.value if hasattr(dev.state, 'value') else dev.state
                    if state == 1:
                        out_list.append((get_device_name(dev), dev.id))
                except: continue

            # 2. CAPTURE (Input)
            capture_devs = wa.AudioUtilities.GetAllDevices(data_flow=1)
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
            wa.comtypes.CoUninitialize()
        except: pass

# --- MEDIA CONTROL WORKER (WINRT) ---
class MediaWorker(QThread):
    metadata_updated = Signal(str, str, bytes) # title, artist, thumbnail_bytes
    status_updated = Signal(bool) # is_playing

    def __init__(self):
        super().__init__()
        self.running = True
        self.manager = None
        self.current_session = None
        
        # State tracking
        self.last_title = ""
        self.current_thumbnail = b"" # Store the thumbnail bytes here

    def get_manager(self):
        if not wa.WINRT_AVAILABLE: return None
        if self.manager is None:
            try:
                # Synchronously wait for manager
                self.manager = wa.GlobalSystemMediaTransportControlsSessionManager.request_async().get()
            except Exception as e:
                print(f"WinRT Manager Request Failed: {e}")
                self.manager = None
        return self.manager

    def run(self):
        while self.running and wa.WINRT_AVAILABLE:
            try:
                mgr = self.get_manager()
                if not mgr:
                    time.sleep(2)
                    continue

                session = mgr.get_current_session()
                self.current_session = session 

                if session:
                    # 1. Status
                    try:
                        info = session.get_playback_info()
                        is_playing = (info.playback_status == wa.GlobalSystemMediaTransportControlsSessionPlaybackStatus.PLAYING)
                        self.status_updated.emit(is_playing)
                    except:
                        self.status_updated.emit(False)

                    # 2. Metadata
                    try:
                        props = session.try_get_media_properties_async().get()
                        
                        if props:
                            title = props.title if props.title else "Unknown Title"
                            artist = props.artist if props.artist else ""
                            
                            # Check if title changed. If so, fetch NEW art.
                            if title != self.last_title:
                                self.last_title = title
                                self.current_thumbnail = b"" # Reset first
                                
                                if props.thumbnail:
                                    try:
                                        stream = props.thumbnail.open_read_async().get()
                                        size = stream.size
                                        if size > 0:
                                            reader = wa.DataReader(stream.get_input_stream_at(0))
                                            reader.load_async(size).get()
                                            
                                            buffer = bytearray(size)
                                            reader.read_bytes(buffer)
                                            self.current_thumbnail = bytes(buffer)
                                    except Exception as e:
                                        print(f"Thumbnail fetch error: {e}")
                            
                            # Emit the PERSISTED thumbnail data
                            self.metadata_updated.emit(title, artist, self.current_thumbnail)
                        else:
                            self.metadata_updated.emit("Media Active", "Waiting for data...", b"")
                            
                    except Exception as e:
                        # Keep session alive in UI even if props fail
                        self.metadata_updated.emit("Media Detected", "", b"")
                else:
                    self.status_updated.emit(False)
                    self.metadata_updated.emit("No Media", "", b"")
                    self.last_title = ""
                    self.current_thumbnail = b""

            except Exception as e:
                print(f"MediaWorker Loop Error: {e}")
            
            time.sleep(1)

    def stop(self):
        self.running = False
        self.wait()

    # Public control methods
    def toggle_play(self):
        if self.current_session:
            try: self.current_session.try_toggle_play_pause_async()
            except: pass

    def next_track(self):
        if self.current_session:
            try: self.current_session.try_skip_next_async()
            except: pass

    def prev_track(self):
        if self.current_session:
            try: self.current_session.try_skip_previous_async()
            except: pass

# --- MEDIA CONTROL WIDGET ---
class MediaControlWidget(QWidget):
    def __init__(self, worker):
        super().__init__()
        self.worker = worker
        self.setup_ui()
        
        # Connect worker signals
        self.worker.metadata_updated.connect(self.update_metadata)
        self.worker.status_updated.connect(self.update_status)

    def setup_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(10, 10, 10, 10)
        layout.setSpacing(8)

        # Background style
        self.setStyleSheet(f"""
            QWidget {{
                background-color: {TILE_INACTIVE};
                border-radius: 8px;
            }}
        """)

        # 1. Info Row (Art + Text)
        info_layout = QHBoxLayout()
        
        # Album Art
        self.art_lbl = QLabel()
        self.art_lbl.setFixedSize(48, 48)
        self.art_lbl.setStyleSheet("background-color: #333; border-radius: 4px; border: 1px solid #555;")
        self.art_lbl.setAlignment(Qt.AlignCenter)
        
        # Text Info
        text_layout = QVBoxLayout()
        text_layout.setSpacing(2)
        text_layout.setContentsMargins(0, 4, 0, 4)
        
        self.title_lbl = QLabel("No Media")
        self.title_lbl.setStyleSheet("font-weight: bold; font-size: 13px; color: white; background: transparent;")
        
        self.artist_lbl = QLabel("")
        self.artist_lbl.setStyleSheet("font-size: 11px; color: #aaaaaa; background: transparent;")
        
        text_layout.addWidget(self.title_lbl)
        text_layout.addWidget(self.artist_lbl)
        text_layout.addStretch()

        info_layout.addWidget(self.art_lbl)
        info_layout.addLayout(text_layout)
        info_layout.addStretch()

        layout.addLayout(info_layout)

        # 2. Controls Row
        ctrl_layout = QHBoxLayout()
        ctrl_layout.setContentsMargins(0, 0, 0, 0)
        ctrl_layout.setSpacing(15)
        ctrl_layout.setAlignment(Qt.AlignCenter)

        self.btn_prev = self._make_btn("mdi.skip-previous", self.worker.prev_track)
        self.btn_play = self._make_btn("mdi.play", self.worker.toggle_play, size=32)
        self.btn_next = self._make_btn("mdi.skip-next", self.worker.next_track)

        ctrl_layout.addWidget(self.btn_prev)
        ctrl_layout.addWidget(self.btn_play)
        ctrl_layout.addWidget(self.btn_next)

        layout.addLayout(ctrl_layout)

    def _make_btn(self, icon, func, size=28):
        btn = QPushButton()
        btn.setFixedSize(size, size)
        btn.setCursor(Qt.PointingHandCursor)
        btn.clicked.connect(func)
        btn.setIcon(qta.icon(icon, color="white"))
        btn.setIconSize(QSize(size-10, size-10))
        btn.setStyleSheet(f"""
            QPushButton {{
                background-color: transparent;
                border-radius: {size//2}px;
                border: none;
            }}
            QPushButton:hover {{
                background-color: rgba(255, 255, 255, 0.1);
            }}
            QPushButton:pressed {{
                background-color: rgba(255, 255, 255, 0.2);
            }}
        """)
        return btn

    def update_metadata(self, title, artist, thumb_bytes):
        self.title_lbl.setText(title[:30] + "..." if len(title) > 30 else title)
        self.artist_lbl.setText(artist[:30] + "..." if len(artist) > 30 else artist)
        
        if thumb_bytes:
            pixmap = QPixmap()
            pixmap.loadFromData(thumb_bytes)
            if not pixmap.isNull():
                # Scale and round corners
                scaled = pixmap.scaled(48, 48, Qt.KeepAspectRatioByExpanding, Qt.SmoothTransformation)
                
                # Apply rounded clipping
                target = QPixmap(48, 48)
                target.fill(Qt.transparent)
                painter = QPainter(target)
                painter.setRenderHint(QPainter.Antialiasing)
                path = QPainterPath()
                path.addRoundedRect(0, 0, 48, 48, 4, 4)
                painter.setClipPath(path)
                painter.drawPixmap(0, 0, scaled)
                painter.end()
                
                self.art_lbl.setPixmap(target)
                self.art_lbl.setText("") # <--- Ensure text is cleared
                return
        
        # Default icon if no art
        self.art_lbl.setPixmap(QPixmap())
        self.art_lbl.setText("🎵")

    def update_status(self, is_playing):
        icon = "mdi.pause" if is_playing else "mdi.play"
        self.btn_play.setIcon(qta.icon(icon, color="white"))

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
        
        # Media Worker (WinRT)
        self.media_worker = MediaWorker()
        
        self.refresh_interfaces()
        self.scanner.start()
        
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.update_status)
        self.timer.start(settings.audio_poll_rate)
        self.update_status()

    def refresh_interfaces(self):
        if not wa.COM_AVAILABLE: return
        try:
            def_spk = wa.AudioUtilities.GetSpeakers()
            if def_spk:
                self.current_output_id = def_spk.id
                self.spk_interface = self._activate_interface(def_spk)
            
            def_mic = wa.AudioUtilities.GetMicrophone()
            if def_mic:
                self.current_input_id = def_mic.id
                self.mic_interface = self._activate_interface(def_mic)
        except: pass

    def _activate_interface(self, dev):
        try:
            raw_dev = dev._dev if hasattr(dev, '_dev') else dev
            if hasattr(raw_dev, 'Activate'):
                interface = raw_dev.Activate(wa.IAudioEndpointVolume._iid_, wa.CLSCTX_ALL, None)
            else:
                interface = raw_dev.activate(wa.IAudioEndpointVolume._iid_, wa.CLSCTX_ALL, None)
            return interface.QueryInterface(wa.IAudioEndpointVolume)
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
            btn.clicked.connect(lambda checked=False, d_id=dev_id, inp=is_input: self._on_device_selected(d_id, inp))
            
            layout.addWidget(btn)

    def _on_device_selected(self, dev_id, is_input):
        if not wa.COM_AVAILABLE: return

        # 1. Set Windows Default
        try:
            roles = [wa.ERole.eConsole, wa.ERole.eMultimedia, wa.ERole.eCommunications]
            wa.AudioUtilities.SetDefaultDevice(dev_id, roles=roles)
        except Exception as e:
            print(f"Failed to switch device: {e}")
            return

        # 2. Update Internal State
        if is_input: self.current_input_id = dev_id
        else: self.current_output_id = dev_id
        
        self.refresh_interfaces()
        
        # 3. Visually update the list
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

        # Start Media Worker if not running
        if not self.media_worker.isRunning():
            self.media_worker.start()

        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(5, 5, 5, 5)
        layout.setSpacing(10)

        # Helper to add section dividers
        def add_divider():
            line = QFrame()
            line.setFrameShape(QFrame.HLine)
            line.setFrameShadow(QFrame.Sunken)
            line.setStyleSheet(f"background-color: #3e3e3e; max-height: 1px; margin-top: 5px; margin-bottom: 5px;")
            layout.addWidget(line)

        # --- SECTION 1: MASTER VOLUME (Both Sliders) ---
        lbl_vol = QLabel("Master Volume")
        lbl_vol.setStyleSheet("color: #cccccc; font-size: 11px; font-weight: bold; margin-bottom: 2px;")
        layout.addWidget(lbl_vol)

        # 1A. Speaker Logic & Slider
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
        self.slider_out_ref.setToolTip("Speaker Volume")
        layout.addWidget(self.slider_out_ref)

        # 1B. Microphone Logic & Slider
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
        self.slider_in_ref.setToolTip("Microphone Volume")
        layout.addWidget(self.slider_in_ref)

        add_divider()

        # --- SECTION 2: OUTPUT DEVICE ---
        lbl_out = QLabel("Output Device")
        lbl_out.setStyleSheet("color: #cccccc; font-size: 11px; font-weight: bold; margin-bottom: 2px;")
        layout.addWidget(lbl_out)

        self.out_list_layout = QVBoxLayout()
        self.out_list_layout.setSpacing(2)
        layout.addLayout(self.out_list_layout)

        # Initial Populate Output
        self._repopulate_list(self.out_list_layout, self.output_devices_list, self.current_output_id, is_input=False)

        add_divider()

        # --- SECTION 3: INPUT DEVICE ---
        lbl_in = QLabel("Input Device")
        lbl_in.setStyleSheet("color: #cccccc; font-size: 11px; font-weight: bold; margin-bottom: 2px;")
        layout.addWidget(lbl_in)

        self.in_list_layout = QVBoxLayout()
        self.in_list_layout.setSpacing(2)
        layout.addLayout(self.in_list_layout)
        
        # Initial Populate Input
        self._repopulate_list(self.in_list_layout, self.input_devices_list, self.current_input_id, is_input=True)

        # --- SECTION 4: MEDIA CONTROL (IF AVAILABLE) ---
        if wa.WINRT_AVAILABLE:
            add_divider()

            lbl_media = QLabel("Media Player")
            lbl_media.setStyleSheet("color: #cccccc; font-size: 11px; font-weight: bold; margin-bottom: 2px;")
            layout.addWidget(lbl_media)

            media_widget = MediaControlWidget(self.media_worker)
            layout.addWidget(media_widget)
        
        return "Audio Mixer", container