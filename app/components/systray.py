# app/components/systray.py
import sys
import threading
import ctypes
import ctypes.wintypes
from typing import Dict, Tuple

from PySide6.QtWidgets import (QWidget, QVBoxLayout, QPushButton, QLabel, 
                               QHBoxLayout, QFrame, QScrollArea, QSizePolicy)
from PySide6.QtCore import Signal, QObject, Qt, QSize, QPoint
from PySide6.QtGui import QIcon, QPixmap, QImage, QCursor, QColor
import qtawesome as qta

# Import consolidated API
from .. import winapiref as wa
from .common import ClickableLabel, ACCENT_COLOR, TILE_HOVER, TILE_INACTIVE, TEXT_WHITE

# --- Helpers ---
def to_signed_64(val):
    val = val & 0xFFFFFFFFFFFFFFFF
    if val > 0x7FFFFFFFFFFFFFFF:
        val -= 0x10000000000000000
    return val

def pack_wparam_coords(x, y):
    x = int(x) & 0xFFFF
    y = int(y) & 0xFFFF
    return (y << 16) | x

def pack_lparam_v4(msg, uid):
    msg = int(msg) & 0xFFFF
    uid = int(uid) & 0xFFFF
    return (uid << 16) | msg

def pack_i32(low, high):
    return ctypes.c_long((low & 0xFFFF) | (high << 16)).value

def hicon_to_pixmap(hicon):
    if not hicon: return None
    icon_info = wa.ICONINFO()
    if not wa.user32.GetIconInfo(ctypes.wintypes.HICON(hicon), ctypes.byref(icon_info)): return None
    try:
        raw_bmp = icon_info.hbmColor if icon_info.hbmColor else icon_info.hbmMask
        bmp_meta = wa.BITMAP()
        wa.gdi32.GetObjectW(raw_bmp, ctypes.sizeof(bmp_meta), ctypes.byref(bmp_meta))
        if bmp_meta.bmBitsPixel != 32: return None
        width, height = bmp_meta.bmWidth, bmp_meta.bmHeight
        total_bytes = width * height * 4
        buff = ctypes.create_string_buffer(total_bytes)
        if wa.gdi32.GetBitmapBits(raw_bmp, total_bytes, buff) == 0: return None
        img = QImage(buff, width, height, QImage.Format.Format_ARGB32)
        return QPixmap.fromImage(img.copy())
    finally:
        if icon_info.hbmColor: wa.gdi32.DeleteObject(icon_info.hbmColor)
        if icon_info.hbmMask: wa.gdi32.DeleteObject(icon_info.hbmMask)

def get_exe_path_from_hwnd(hwnd):
    process_id = ctypes.c_ulong(0)
    wa.user32.GetWindowThreadProcessId(hwnd, ctypes.byref(process_id))
    if process_id.value == 0: return None
    h_process = wa.kernel32.OpenProcess(wa.PROCESS_QUERY_LIMITED_INFORMATION, False, process_id.value)
    if not h_process: return None
    try:
        buffer_size = ctypes.c_ulong(1024)
        buffer = ctypes.create_unicode_buffer(buffer_size.value)
        if wa.kernel32.QueryFullProcessImageNameW(h_process, 0, buffer, ctypes.byref(buffer_size)):
            return buffer.value
    finally:
        wa.kernel32.CloseHandle(h_process)
    return None

def get_process_name(hwnd):
    path = get_exe_path_from_hwnd(hwnd)
    return path.split("\\")[-1] if path else "Unknown"

# --- Backend Monitor ---
class TraySignals(QObject):
    icon_added = Signal(object)    
    icon_modified = Signal(object)
    icon_removed = Signal(object, object)
    icon_version = Signal(object)

class SystemTrayMonitor:
    def __init__(self):
        self.signals = TraySignals()
        self.hwnd = None
        self.real_tray_hwnd = None
        self.running = False
        self.class_name = "Shell_TrayWnd"
        self.rects: Dict[str, Tuple[int, int]] = {} 

    def start(self):
        self.running = True
        self.msg_thread = threading.Thread(target=self._run_window_loop, daemon=True)
        self.msg_thread.start()
    
    def update_icon_rect(self, key_tuple, x, y):
        self.rects[key_tuple] = (x, y)

    def find_real_tray_hwnd(self, hwnd_ignore=None):
        hwnd = 0
        while True:
            hwnd = wa.user32.FindWindowExW(0, hwnd, "Shell_TrayWnd", None)
            if hwnd == 0: break
            if hwnd == hwnd_ignore: continue
            exe = get_exe_path_from_hwnd(hwnd)
            if exe and "explorer.exe" in exe.lower(): return hwnd
        return 0

    def forward_message(self, hwnd, msg, wparam, lparam):
        if not self.real_tray_hwnd or not wa.user32.IsWindow(self.real_tray_hwnd):
            self.real_tray_hwnd = self.find_real_tray_hwnd(hwnd)
        if self.real_tray_hwnd:
            return wa.user32.SendMessageW(self.real_tray_hwnd, msg, wparam, lparam)
        return wa.user32.DefWindowProcW(hwnd, msg, wparam, lparam)

    def _run_window_loop(self):
        try:
            self.wnd_proc = wa.WNDPROCTYPE(self._window_proc)
            h_inst = wa.kernel32.GetModuleHandleW(None)
            self.wc = wa.WNDCLASS()
            self.wc.lpfnWndProc = self.wnd_proc
            self.wc.hInstance = h_inst
            self.wc.lpszClassName = self.class_name
            wa.user32.RegisterClassW(ctypes.byref(self.wc))
            
            ex_style = wa.WS_EX_TOOLWINDOW | wa.WS_EX_TOPMOST
            style = wa.WS_POPUP | wa.WS_CLIPCHILDREN | wa.WS_CLIPSIBLINGS
            self.hwnd = wa.user32.CreateWindowExW(ex_style, self.class_name, "Yasb Tray", style, 0, 0, 0, 0, None, None, h_inst, None)
            if not self.hwnd: return
            
            wa.user32.SetWindowPos(self.hwnd, wa.HWND_TOPMOST, 0, 0, 0, 0, wa.SWP_NOMOVE | wa.SWP_NOSIZE | wa.SWP_NOACTIVATE)
            wa.user32.SetTimer(self.hwnd, 1, 100, None)
            wa.user32.SetPropW(self.hwnd, "TaskbandHWND", self.hwnd)
            
            WM_TASKBARCREATED = wa.user32.RegisterWindowMessageW("TaskbarCreated")
            if WM_TASKBARCREATED:
                wa.user32.SendNotifyMessageW(0xFFFF, WM_TASKBARCREATED, 0, 0)
            
            msg = ctypes.wintypes.MSG()
            while self.running and wa.user32.GetMessageW(ctypes.byref(msg), 0, 0, 0) > 0:
                wa.user32.TranslateMessage(ctypes.byref(msg))
                wa.user32.DispatchMessageW(ctypes.byref(msg))
            wa.user32.UnregisterClassW(self.class_name, h_inst)
        except Exception as e:
            print(f"Tray Error: {e}")

    def _window_proc(self, hwnd, msg, wparam, lparam):
        if msg == wa.WM_TIMER:
            wa.user32.SetWindowPos(hwnd, wa.HWND_TOPMOST, 0, 0, 0, 0, wa.SWP_NOMOVE | wa.SWP_NOSIZE | wa.SWP_NOACTIVATE)
            return 0
        elif msg == wa.WM_COPYDATA:
            try:
                p_cds = ctypes.cast(lparam, ctypes.POINTER(wa.COPYDATASTRUCT))
                if not p_cds: return 0
                cds = p_cds.contents
                
                if cds.dwData == 1:
                    raw_ptr = ctypes.cast(cds.lpData, ctypes.POINTER(wa.SHELLTRAYDATA)).contents
                    dwMessage = raw_ptr.dwMessage
                    
                    byte_ptr = ctypes.cast(cds.lpData, ctypes.POINTER(ctypes.c_ubyte * 12)).contents
                    cb_size = int.from_bytes(byte_ptr[8:12], byteorder='little')

                    if cb_size == 956:
                        struct_ptr = ctypes.c_void_p(ctypes.addressof(raw_ptr) + 8)
                        nid = wa.NOTIFYICONDATA_32.from_address(struct_ptr.value)
                    else:
                        struct_ptr = ctypes.c_void_p(ctypes.addressof(raw_ptr) + 8)
                        nid = wa.NOTIFYICONDATA_64.from_address(struct_ptr.value)

                    guid_str = None
                    if (nid.uFlags & wa.NIF_GUID): guid_str = str(nid.guidItem)
                    exe_name = get_process_name(nid.hWnd)
                    
                    tip = ""
                    if nid.szTip:
                        try: tip = nid.szTip.split('\0')[0]
                        except: pass

                    data = {
                        'hwnd': nid.hWnd, 'uid': nid.uID, 'guid': guid_str,
                        'callback': nid.uCallbackMessage, 'tip': tip,
                        'hIcon': nid.hIcon, 'flags': nid.uFlags,
                        'version': nid.uVersion.uVersion, 'exe': exe_name
                    }
                    
                    if dwMessage == wa.NIM_ADD:
                        self.signals.icon_added.emit(data)
                    elif dwMessage == wa.NIM_MODIFY:
                        self.signals.icon_modified.emit(data)
                    elif dwMessage == wa.NIM_DELETE:
                        self.signals.icon_removed.emit(nid.hWnd, nid.uID)
                    elif dwMessage == wa.NIM_SETVERSION:
                        self.signals.icon_version.emit(data)
                    
                    return self.forward_message(hwnd, msg, wparam, lparam)

                elif cds.dwData == 3:
                    icon_id = ctypes.cast(cds.lpData, ctypes.POINTER(wa.WINNOTIFYICONIDENTIFIER)).contents
                    guid_key = str(icon_id.guidItem)
                    legacy_key = (icon_id.hWnd, icon_id.uID)
                    
                    # Fallback to cursor pos if we don't know the rect
                    pt = ctypes.wintypes.POINT()
                    wa.user32.GetCursorPos(ctypes.byref(pt))
                    
                    if guid_key in self.rects:
                        x, y = self.rects[guid_key]
                        return pack_i32(x, y)
                    elif legacy_key in self.rects:
                        x, y = self.rects[legacy_key]
                        return pack_i32(x, y)
                    else:
                        return pack_i32(pt.x, pt.y)

            except Exception:
                return 0
        
        elif msg == wa.WM_DESTROY:
            wa.user32.PostQuitMessage(0)
            return 0
        
        return wa.user32.DefWindowProcW(hwnd, msg, wa.WPARAM(wparam), wa.LPARAM(to_signed_64(lparam)))

# --- List Row Widget (Vertical Item) ---
class TrayRowWidget(QPushButton):
    def __init__(self, data, monitor_ref, parent=None):
        super().__init__(parent)
        self.target_hwnd = data['hwnd']
        self.callback_id = data['callback']
        self.uid = data['uid']
        self.guid = data['guid']
        self.exe_name = data['exe']
        self.version = data['version'] if data['version'] > 0 else 0 
        self.monitor_ref = monitor_ref
        
        self.setCursor(Qt.PointingHandCursor)
        self.setFixedHeight(45)
        
        # Internal Layout
        self.layout = QHBoxLayout(self)
        self.layout.setContentsMargins(10, 5, 10, 5)
        self.layout.setSpacing(10)

        # Icon Label
        self.icon_lbl = QLabel()
        self.icon_lbl.setFixedSize(24, 24)
        self.icon_lbl.setStyleSheet("background: transparent; border: none;")
        self.layout.addWidget(self.icon_lbl)
        
        # Text Label
        self.text_lbl = QLabel()
        self.text_lbl.setStyleSheet(f"color: {TEXT_WHITE}; font-family: 'Segoe UI'; font-size: 13px; background: transparent; border: none;")
        self.text_lbl.setWordWrap(False)
        self.layout.addWidget(self.text_lbl)
        self.layout.addStretch()

        self.setStyleSheet(f"""
            QPushButton {{
                background-color: transparent;
                border: none;
                border-radius: 4px;
                text-align: left;
            }}
            QPushButton:hover {{
                background-color: {TILE_HOVER};
            }}
            QPushButton:pressed {{
                background-color: {TILE_INACTIVE};
            }}
        """)
        
        self.update_data(data)
        
    def update_data(self, data):
        # Update Text
        text = data['tip'] if data['tip'] else data['exe']
        self.text_lbl.setText(text)
        self.setToolTip(text)
        
        # Update Icon
        if (data['flags'] & wa.NIF_ICON) and data['hIcon'] != 0:
            pixmap = hicon_to_pixmap(data['hIcon'])
            if pixmap and not pixmap.isNull():
                self.icon_lbl.setPixmap(pixmap.scaled(24, 24, Qt.KeepAspectRatio, Qt.SmoothTransformation))
            else:
                self.icon_lbl.setText("")
        
    def set_version(self, version):
        self.version = version

    def get_physical_cursor_pos(self):
        pos = QCursor.pos()
        dpr = self.devicePixelRatio()
        return int(pos.x() * dpr), int(pos.y() * dpr)

    def mouseReleaseEvent(self, event):
        hwnd_obj = wa.HWND(self.target_hwnd)
        pid = ctypes.c_ulong()
        wa.user32.GetWindowThreadProcessId(hwnd_obj, ctypes.byref(pid))
        wa.user32.AllowSetForegroundWindow(pid)
        wa.user32.SetForegroundWindow(hwnd_obj)

        if event.button() == Qt.MouseButton.LeftButton:
            self._send_click_sequence(wa.WM_LBUTTONDOWN, wa.WM_LBUTTONUP, wa.NIN_SELECT)
        elif event.button() == Qt.MouseButton.RightButton:
            self._send_click_sequence(wa.WM_RBUTTONDOWN, wa.WM_RBUTTONUP, wa.NIN_CONTEXTMENU, send_wm_context=True)
        super().mouseReleaseEvent(event)
    
    def mouseDoubleClickEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
             self._send_click_sequence(wa.WM_LBUTTONDBLCLK, wa.WM_LBUTTONUP, wa.NIN_SELECT)
        super().mouseDoubleClickEvent(event)

    def _send_click_sequence(self, down_msg, up_msg, v3_extra_msg, send_wm_context=False):
        self._send_raw(wa.WM_MOUSEMOVE)
        if self.version >= 3:
            self._send_raw(wa.NIN_POPUPOPEN)
            
        self._send_raw(down_msg)
        self._send_raw(up_msg)
        
        if send_wm_context:
            self._send_raw(wa.WM_CONTEXTMENU)

        if self.version >= 3 and v3_extra_msg:
             self._send_raw(v3_extra_msg)

    def _send_raw(self, msg_id):
        # Notify backend of our location (approximation for popup menus)
        # In a list, we just send cursor pos usually
        if self.monitor_ref:
            pos = self.mapToGlobal(QPoint(0, 0))
            dpr = self.devicePixelRatio()
            phys_x = int((pos.x() + (self.width() / 2)) * dpr)
            phys_y = int((pos.y() + (self.height() / 2)) * dpr)
            
            key = (self.target_hwnd, self.uid)
            self.monitor_ref.update_icon_rect(key, phys_x, phys_y)
            if self.guid:
                self.monitor_ref.update_icon_rect(self.guid, phys_x, phys_y)

        wparam = 0
        lparam = 0
        
        if self.version >= 4:
            px, py = self.get_physical_cursor_pos()
            wparam = pack_wparam_coords(px, py)
            lparam = pack_lparam_v4(msg_id, self.uid)
        else:
            wparam = self.uid
            lparam = msg_id

        wa.user32.SendNotifyMessageW(
            wa.HWND(self.target_hwnd), self.callback_id, 
            wa.WPARAM(wparam), wa.LPARAM(to_signed_64(lparam))
        )

# --- Popup Content Widget ---
class TrayMenuWidget(QWidget):
    def __init__(self, initial_data, monitor):
        super().__init__()
        self.monitor = monitor
        self.row_map = {} # Key -> Widget

        # Layout
        self.layout = QVBoxLayout(self)
        self.layout.setContentsMargins(0, 0, 0, 0)
        self.layout.setSpacing(2)

        # Scroll Area
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        scroll.setStyleSheet(f"""
            QScrollArea {{ background: transparent; border: none; }}
            QWidget {{ background: transparent; }}
            QScrollBar:vertical {{
                background: #2b2b2b;
                width: 8px;
                border-radius: 4px;
            }}
            QScrollBar::handle:vertical {{
                background: #555;
                border-radius: 4px;
            }}
        """)
        
        self.container = QWidget()
        self.cont_layout = QVBoxLayout(self.container)
        self.cont_layout.setContentsMargins(5, 5, 5, 5)
        self.cont_layout.setSpacing(2)
        
        scroll.setWidget(self.container)
        self.layout.addWidget(scroll)

        # Fixed size for popup
        self.setFixedSize(300, 400)

        # Populate initial
        for key, data in initial_data.items():
            self.add_row(key, data)

        # Connect signals for live updates
        self.monitor.signals.icon_added.connect(self.on_added)
        self.monitor.signals.icon_modified.connect(self.on_modified)
        self.monitor.signals.icon_removed.connect(self.on_removed)

    def get_key(self, data):
        if data.get('guid'): return data['guid']
        return (data['hwnd'], data['uid'])

    def add_row(self, key, data):
        if key in self.row_map: return
        row = TrayRowWidget(data, self.monitor)
        self.cont_layout.addWidget(row)
        self.row_map[key] = row

    def on_added(self, data):
        key = self.get_key(data)
        self.add_row(key, data)

    def on_modified(self, data):
        key = self.get_key(data)
        if key in self.row_map:
            self.row_map[key].update_data(data)
        else:
            self.add_row(key, data)

    def on_removed(self, hwnd, uid):
        # Find key
        target_key = None
        for k, row in self.row_map.items():
            if row.target_hwnd == hwnd and row.uid == uid:
                target_key = k
                break
        
        if target_key:
            row = self.row_map.pop(target_key)
            self.cont_layout.removeWidget(row)
            row.deleteLater()

# --- Main Bar Component ---
class SystemTrayComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("", parent, settings)
        self.settings = settings
        self.setIcon("mdi.chevron-up") # Chevron icon
        
        # State Data (Keep alive here, because popup dies on close)
        self.tray_data = {} 
        
        self.monitor = SystemTrayMonitor()
        self.monitor.signals.icon_added.connect(self._store_add)
        self.monitor.signals.icon_modified.connect(self._store_mod)
        self.monitor.signals.icon_removed.connect(self._store_rem)
        self.monitor.signals.icon_version.connect(self._store_ver)
        self.monitor.start()

    def get_key(self, data):
        if data.get('guid'): return data['guid']
        return (data['hwnd'], data['uid'])

    def _store_add(self, data):
        if data['uid'] > 3000000000: return
        if self.settings.tray_ignore_list and data['exe'] in self.settings.tray_ignore_list: return
        self.tray_data[self.get_key(data)] = data

    def _store_mod(self, data):
        key = self.get_key(data)
        if key in self.tray_data:
            self.tray_data[key].update(data)
        else:
            self._store_add(data)

    def _store_rem(self, hwnd, uid):
        # Find and remove
        to_del = []
        for k, v in self.tray_data.items():
            if v['hwnd'] == hwnd and v['uid'] == uid:
                to_del.append(k)
        for k in to_del:
            del self.tray_data[k]

    def _store_ver(self, data):
        key = self.get_key(data)
        if key in self.tray_data:
            self.tray_data[key]['version'] = data['version']

    def get_popup_content(self):
        return "System Tray", TrayMenuWidget(self.tray_data, self.monitor)