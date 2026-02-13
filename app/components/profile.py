# app/components/profile.py
import os
import ctypes
import asyncio
from PySide6.QtWidgets import (QWidget, QLabel, QVBoxLayout, QHBoxLayout, QPushButton, QFrame)
from PySide6.QtCore import Qt, Signal, QThread, QUrl
from PySide6.QtGui import QPixmap, QPainter, QPainterPath, QColor, QFont, QDesktopServices
import qtawesome as qta

# Import consolidated API
from .. import winapiref as wa
from .common import ClickableLabel, TILE_INACTIVE, TILE_HOVER, ACCENT_COLOR, TEXT_WHITE, TEXT_SUB
from ipclib import NeverLiieIPC # <--- Add this
# --- WORKER TO FETCH REAL WINDOWS DATA ---
class ProfileWorker(QThread):
    # Signals: display_name, principal_name (email), image_bytes
    data_ready = Signal(str, str, bytes)

    def run(self):
        if not wa.WINRT_AVAILABLE:
            self._emit_fallback()
            return

        try:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            loop.run_until_complete(self._fetch_info())
            loop.close()
        except Exception as e:
            print(f"Profile Worker Error: {e}")
            self._emit_fallback()

    def _emit_fallback(self):
        name = self._get_local_display_name()
        domain = os.environ.get('USERDOMAIN', 'Local')
        username = os.environ.get('USERNAME', 'User')
        principal = f"{domain}\\{username}" 
        self.data_ready.emit(name, principal, b"")

    def _get_local_display_name(self):
        try:
            if wa.secur32:
                NameDisplay = 3
                size = ctypes.pointer(ctypes.c_ulong(0))
                wa.secur32.GetUserNameExW(NameDisplay, None, size)
                name_buffer = ctypes.create_unicode_buffer(size.contents.value)
                wa.secur32.GetUserNameExW(NameDisplay, name_buffer, size)
                if name_buffer.value:
                    return name_buffer.value
        except: pass
        return os.getlogin()

    async def _fetch_info(self):
        # 1. Find Current User
        users = await wa.User.find_all_async()
        if not users:
            self._emit_fallback()
            return

        # Usually the first user returned is the active interactive user
        current_user = users[0]

        # 2. Request Properties
        props_to_fetch = [
            wa.KnownUserProperties.display_name,
            wa.KnownUserProperties.account_name,
            wa.KnownUserProperties.domain_name
        ]
        
        display_name = ""
        account_name = ""

        try:
            values = await current_user.get_properties_async(props_to_fetch)
            
            # Lookup returns a raw Object; we must unbox it
            raw_disp = values.lookup(wa.KnownUserProperties.display_name)
            raw_acct = values.lookup(wa.KnownUserProperties.account_name)
            
            display_name = wa.unbox_winrt_str(raw_disp)
            account_name = wa.unbox_winrt_str(raw_acct)
            
            # Fallbacks if properties are empty
            if not display_name: 
                display_name = self._get_local_display_name()
            if not account_name: 
                raw_dom = values.lookup(wa.KnownUserProperties.domain_name)
                domain = wa.unbox_winrt_str(raw_dom)
                if domain: account_name = f"{domain}\\{display_name}"
                else: account_name = "Local Account"
            
        except Exception as e:
            print(f"Prop Fetch Error: {e}")
            display_name = self._get_local_display_name()
            account_name = "Error Fetching Info"

        # 3. Request Picture
        img_bytes = b""
        try:
            stream_ref = await current_user.get_picture_async(wa.UserPictureSize.SIZE1080X1080)
            if stream_ref:
                stream = await stream_ref.open_read_async()
                if stream:
                    size = stream.size
                    reader = wa.DataReader(stream.get_input_stream_at(0))
                    await reader.load_async(size)
                    buffer = bytearray(size)
                    reader.read_bytes(buffer)
                    img_bytes = bytes(buffer)
        except Exception as e:
            print(f"Pic Fetch Error: {e}")

        # Emit standard Python strings to avoid Shiboken errors
        self.data_ready.emit(str(display_name), str(account_name), img_bytes)


# --- IMAGE PROCESSING UTILS ---
def process_avatar(image_bytes, size, name_fallback):
    pixmap = QPixmap(size, size)
    pixmap.fill(Qt.transparent)

    source = QPixmap()
    loaded = False
    
    if image_bytes:
        if source.loadFromData(image_bytes):
            loaded = True
            source = source.scaled(size, size, Qt.KeepAspectRatioByExpanding, Qt.SmoothTransformation)

    painter = QPainter(pixmap)
    painter.setRenderHint(QPainter.Antialiasing)

    path = QPainterPath()
    path.addEllipse(0, 0, size, size)
    painter.setClipPath(path)

    if loaded:
        x = (size - source.width()) // 2
        y = (size - source.height()) // 2
        painter.drawPixmap(x, y, source)
    else:
        # Initials Fallback
        painter.fillRect(0, 0, size, size, QColor(ACCENT_COLOR)) 
        painter.setPen(Qt.black)
        font = QFont("Segoe UI", int(size * 0.45))
        font.setBold(True)
        painter.setFont(font)
        
        initials = "??"
        if name_fallback:
            parts = name_fallback.strip().split()
            if len(parts) >= 2:
                initials = (parts[0][0] + parts[1][0]).upper()
            elif len(parts) == 1 and parts[0]:
                initials = parts[0][0:2].upper()
                
        painter.drawText(0, 0, size, size, Qt.AlignCenter, initials)

    painter.end()
    return pixmap


# --- POPUP WIDGET ---
class ProfilePopupWidget(QWidget):
    def __init__(self, display_name, principal_name, image_bytes, ipc_client):
        super().__init__()
        self.ipc = ipc_client
        self.setFixedWidth(300)
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(15, 20, 15, 15)
        layout.setSpacing(15)

        # 1. User Info Section (Existing)
        center_layout = QVBoxLayout()
        center_layout.setSpacing(8)
        center_layout.setAlignment(Qt.AlignCenter)

        lbl_img = QLabel()
        big_pix = process_avatar(image_bytes, 100, display_name)
        lbl_img.setPixmap(big_pix)
        lbl_img.setAlignment(Qt.AlignCenter)
        
        lbl_name = QLabel(display_name)
        lbl_name.setStyleSheet("font-size: 18px; font-weight: bold; color: white;")
        lbl_name.setAlignment(Qt.AlignCenter)
        lbl_name.setWordWrap(True)
        
        # Email / Principal
        lbl_email = QLabel(principal_name)
        lbl_email.setStyleSheet(f"font-size: 13px; color: {TEXT_SUB};")
        lbl_email.setAlignment(Qt.AlignCenter)
        lbl_email.setWordWrap(True)

        center_layout.addWidget(lbl_img)
        center_layout.addWidget(lbl_name)
        center_layout.addWidget(lbl_email)
        layout.addLayout(center_layout)

        # Divider
        line = QFrame()
        line.setFrameShape(QFrame.HLine)
        line.setStyleSheet("background-color: #3e3e3e; max-height: 1px;")
        layout.addWidget(line)

        # 2. Launcher Status Section (NEW)
        launcher_online = self.ipc.ping("Launcher")
        
        status_container = QFrame()
        status_container.setStyleSheet(f"background: {TILE_INACTIVE}; border-radius: 8px;")
        status_layout = QHBoxLayout(status_container)
        
        dot_color = "#4CAF50" if launcher_online else "#F44336"
        status_text = "Launcher Online" if launcher_online else "Launcher Offline"
        
        lbl_dot = QLabel("●")
        lbl_dot.setStyleSheet(f"color: {dot_color}; font-size: 18px; margin-left: 5px;")
        lbl_status = QLabel(status_text)
        lbl_status.setStyleSheet("color: white; font-size: 13px; font-weight: bold;")
        
        status_layout.addWidget(lbl_dot)
        status_layout.addWidget(lbl_status)
        status_layout.addStretch()
        layout.addWidget(status_container)

        # 3. Action Buttons
        btn_layout = QVBoxLayout()
        btn_layout.setSpacing(5)

        def make_row(text, icon_name, func):
            btn = QPushButton(text)
            btn.setIcon(qta.icon(icon_name, color=TEXT_WHITE))
            btn.setCursor(Qt.PointingHandCursor)
            btn.setStyleSheet(f"""
                QPushButton {{
                    text-align: left;
                    padding: 10px;
                    background-color: transparent;
                    border: none;
                    border-radius: 6px;
                    color: {TEXT_WHITE};
                    font-size: 14px;
                }}
                QPushButton:hover {{ background-color: {TILE_HOVER}; }}
            """)
            btn.clicked.connect(func)
            return btn

        # Launcher Logic
        def handle_launcher_click():
            if launcher_online:
                # Call the exposed method on the Launcher
                try:
                    self.ipc.get_peer("Launcher").show()
                except: pass
            else:
                # Try to spawn the process from registry
                self.ipc.wake("Launcher")
            self.window().close() # Close popup

        launcher_btn_text = "  Open Launcher" if launcher_online else "  Wake Launcher"
        launcher_icon = "fa5s.rocket" if launcher_online else "fa5s.power-off"
        
        btn_layout.addWidget(make_row(launcher_btn_text, launcher_icon, handle_launcher_click))
        btn_layout.addWidget(make_row("  Account Settings", "fa5s.user-cog", lambda: QDesktopServices.openUrl(QUrl("ms-settings:yourinfo"))))
        btn_layout.addWidget(make_row("  Open Settings", "fa5s.cogs", lambda: QDesktopServices.openUrl(QUrl("ms-settings:"))))
        
        layout.addLayout(btn_layout)


# --- MAIN COMPONENT ---
class ProfileComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("", parent, settings)
        self.settings = settings
        
        # Initialize IPC for Status Bar
        self.ipc = NeverLiieIPC("StatusBar")
        
        self.display_name = "Loading..."
        self.principal_name = ""
        self.image_bytes = b""
        
        self.text_lbl.setText(self.display_name)
        self.text_lbl.setStyleSheet(f"""
            color: {settings.text_color};
            font-family: '{settings.font_family}';
            font-size: {settings.font_size};
            font-weight: 600; 
            padding-left: 5px;
        """)

        self.worker = ProfileWorker()
        self.worker.data_ready.connect(self.update_data)
        self.worker.start()

    def update_data(self, name, principal, img_bytes):
        self.display_name = name
        self.principal_name = principal
        self.image_bytes = img_bytes
        
        self.text_lbl.setText(name)
        small_pix = process_avatar(img_bytes, 24, name)
        self.icon_lbl.setPixmap(small_pix)
        self.icon_lbl.setVisible(True)

    def get_popup_content(self):
        # Pass the IPC instance to the popup so it can check status
        return "System", ProfilePopupWidget(
            self.display_name, 
            self.principal_name, 
            self.image_bytes,
            self.ipc
        )