import asyncio
import threading
import traceback
import sys
import os
import socket
import time

# --- DEBUGGING IMPORTS (For Focus Spy) ---
import win32gui
import win32process
import psutil

from PySide6.QtCore import Qt, Signal, QObject, QTimer, QThread
from PySide6.QtWidgets import (QWidget, QVBoxLayout, QLabel, QPushButton, 
                               QScrollArea)

# --- CONFIG ---
ENABLE_DEBUG = True

def dprint(msg):
    if ENABLE_DEBUG:
        ts = time.strftime("%H:%M:%S")
        print(f"[{ts}] [NETWORK] {msg}")
        sys.stdout.flush()

# --- MODERN WINRT IMPORTS ---
WINRT_AVAILABLE = False
try:
    dprint("Importing WinRT...")
    from winrt.windows.devices.wifi import WiFiAdapter, WiFiReconnectionKind
    from winrt.windows.security.credentials import PasswordCredential
    import winrt.windows.foundation
    import winrt.windows.foundation.collections # type: ignore
    WINRT_AVAILABLE = True
    dprint("WinRT Import Successful.")
except ImportError as e:
    dprint(f"CRITICAL: WinRT Import Failed: {e}")

from .common import ClickableLabel, WifiListItem

# --- SILENT CONNECTIVITY CHECKER ---
class ConnectivityWorker(QThread):
    status_changed = Signal(bool) 

    def run(self):
        dprint("Connectivity Checker started.")
        while True:
            is_online = self.check_internet()
            self.status_changed.emit(is_online)
            self.sleep(5)

    def check_internet(self):
        try:
            socket.create_connection(("1.1.1.1", 53), timeout=3)
            return True
        except OSError:
            return False

# --- WINRT WORKER (SCANS NETWORKS) ---
class WinRTWorker(QObject):
    scan_finished = Signal(list) 
    status_msg = Signal(str)
    
    def __init__(self):
        super().__init__()
        self.loop = asyncio.new_event_loop()
        self.adapter = None
        self.thread = threading.Thread(target=self._run_loop, daemon=True)
        self.thread.start()

    def _run_loop(self):
        asyncio.set_event_loop(self.loop)
        self.loop.run_forever()

    def start_init(self):
        if WINRT_AVAILABLE:
            # Don't re-init if already done
            if self.adapter: 
                return
            dprint("Initializing Adapter sequence...")
            asyncio.run_coroutine_threadsafe(self._init_adapter(), self.loop)
        else:
            self.status_msg.emit("WinRT Missing")

    async def _init_adapter(self):
        try:
            dprint("Requesting Access Async...")
            await WiFiAdapter.request_access_async()
            
            dprint("Finding Adapters...")
            devs = await WiFiAdapter.find_all_adapters_async()
            
            if devs:
                self.adapter = devs[0]
                dprint(f"Adapter Found: {self.adapter}")
                await self._scan()
            else:
                dprint("No WiFi Adapter found.")
                self.status_msg.emit("No Adapter")
        except Exception as e:
            dprint(f"INIT ERROR: {traceback.format_exc()}")
            self.status_msg.emit("Init Error")

    def request_scan(self):
        dprint("Scan requested by UI.")
        if self.adapter:
            asyncio.run_coroutine_threadsafe(self._scan(), self.loop)
        else:
            # If adapter not ready, try init
            self.start_init()

    async def _scan(self):
        self.status_msg.emit("Scanning...")
        dprint("Starting Scan...")
        try:
            dprint("--- CALLING ADAPTER.SCAN_ASYNC() ---")
            await self.adapter.scan_async()
            dprint("--- SCAN_ASYNC RETURNED ---")
            
            report = self.adapter.network_report 
            dprint(f"Report received. Found {report.available_networks.size} networks.")
            
            results = []
            seen_ssids = set()
            
            connected_ssid = None
            try:
                profile = await self.adapter.network_adapter.get_connected_profile_async()
                if profile:
                    connected_ssid = profile.profile_name
            except: pass

            for net in report.available_networks:
                ssid = net.ssid
                if not ssid: continue
                if ssid in seen_ssids: continue
                
                seen_ssids.add(ssid)
                signal = net.signal_bars
                auth_type = net.security_settings.network_authentication_type
                is_secure = (auth_type > 1) 
                is_connected = (ssid == connected_ssid)

                results.append((ssid, signal, is_secure, is_connected, net))
            
            results.sort(key=lambda x: (not x[3], -x[1]))
            
            dprint(f"Emitting {len(results)} results.")
            self.scan_finished.emit(results)
            self.status_msg.emit("Ready")
            
        except Exception as e:
            dprint(f"SCAN ERROR: {traceback.format_exc()}")
            self.status_msg.emit("Scan Failed")

    def request_connect(self, network_obj, password):
        dprint(f"Connection request for {network_obj.ssid}")
        asyncio.run_coroutine_threadsafe(self._connect(network_obj, password), self.loop)

    async def _connect(self, net, password):
        self.status_msg.emit(f"Connecting to {net.ssid}...")
        
        cred = None
        if password:
            cred = PasswordCredential()
            cred.password = password

        recon = WiFiReconnectionKind.AUTOMATIC
        
        try:
            dprint("Calling connect_async...")
            if cred:
                result = await self.adapter.connect_async(net, recon, cred)
            else:
                result = await self.adapter.connect_async(net, recon)

            dprint(f"Connect result code: {result.connection_status}")

            if result.connection_status == 0: 
                self.status_msg.emit("Connected")
                await asyncio.sleep(2) 
                await self._scan()
            else:
                self.status_msg.emit(f"Failed: {result.connection_status}")
        except Exception as e:
            dprint(f"CONNECT ERROR: {e}")
            self.status_msg.emit("Conn Error")

    def request_disconnect(self, network_obj):
        asyncio.run_coroutine_threadsafe(self._disconnect(), self.loop)
        
    async def _disconnect(self):
        if self.adapter:
            try:
                dprint("Disconnecting...")
                self.adapter.disconnect()
                self.status_msg.emit("Disconnected")
                await asyncio.sleep(1)
                await self._scan()
            except Exception as e:
                dprint(f"DISCONNECT ERROR: {e}")
                self.status_msg.emit("Disc Error")

# --- UI COMPONENTS ---
class WifiPopupWidget(QWidget):
    def __init__(self, worker, cached_data=None):
        super().__init__()
        self.worker = worker
        self.setFixedSize(300, 400)
        
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0,0,0,0)
        
        top_layout = QVBoxLayout()
        top_layout.setContentsMargins(5, 5, 5, 5)
        
        self.lbl_status = QLabel("Initializing...")
        self.lbl_status.setStyleSheet("color: #888; font-size: 11px;")
        self.lbl_status.setAlignment(Qt.AlignCenter)
        
        btn_refresh = QPushButton("Refresh Networks")
        btn_refresh.setCursor(Qt.PointingHandCursor)
        btn_refresh.setStyleSheet(f"""
            QPushButton {{ background: #333; color: white; border: none; border-radius: 4px; padding: 4px; }}
            QPushButton:hover {{ background: #444; }}
        """)
        btn_refresh.clicked.connect(self.worker.request_scan)
        
        top_layout.addWidget(self.lbl_status)
        top_layout.addWidget(btn_refresh)
        layout.addLayout(top_layout)

        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setStyleSheet("background: transparent; border: none;")
        self.scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        
        self.scroll_content = QWidget()
        self.vbox_networks = QVBoxLayout(self.scroll_content)
        self.vbox_networks.setSpacing(2)
        self.vbox_networks.setContentsMargins(0, 0, 0, 0)
        self.vbox_networks.addStretch()
        
        self.scroll.setWidget(self.scroll_content)
        layout.addWidget(self.scroll)

        # Connect Worker Signals to UI
        self.worker.scan_finished.connect(self.update_list)
        self.worker.status_msg.connect(self.lbl_status.setText)
        
        # 1. Load Cache Immediately (Instant UI)
        if cached_data:
            dprint("Loading cached networks...")
            self.lbl_status.setText("Cached Data")
            self.update_list(cached_data)
        
        # 2. Trigger fresh init/scan
        QTimer.singleShot(100, self.worker.start_init)
        if not cached_data:
             QTimer.singleShot(200, self.worker.request_scan)
        else:
             # Even if we have cache, refresh in background silently
             QTimer.singleShot(500, self.worker.request_scan)

    def update_list(self, networks):
        dprint("Updating UI list... (Start)")
        try:
            dprint(f"Clearing {self.vbox_networks.count()} existing items...")
            while self.vbox_networks.count() > 1:
                child = self.vbox_networks.takeAt(0)
                if child.widget():
                    child.widget().deleteLater()
            
            dprint("Existing items cleared.")

            if not networks:
                dprint("No networks to add.")
                lbl = QLabel("No networks found")
                lbl.setAlignment(Qt.AlignCenter)
                lbl.setStyleSheet("color: #666; padding: 20px;")
                self.vbox_networks.insertWidget(0, lbl)
                return

            dprint(f"Looping through {len(networks)} networks to create widgets...")
            
            for i, (ssid, signal, secure, connected, net_obj) in enumerate(networks):
                # Check start time of creation for debug
                t_start = time.perf_counter()
                
                # Passing parent=self.scroll_content helps ensure ownership
                item = WifiListItem(ssid, signal, secure, connected, net_obj)
                
                t_end = time.perf_counter()
                if i < 5: dprint(f"[{i}] Item created in {(t_end - t_start):.4f}s")

                item.connect_requested.connect(self.worker.request_connect)
                item.disconnect_requested.connect(self.worker.request_disconnect)
                self.vbox_networks.insertWidget(self.vbox_networks.count()-1, item)
            
            dprint("All network items added.")
        
        except Exception as e:
            dprint(f"ERROR in update_list: {e}")
            traceback.print_exc()

class NetworkComponent(ClickableLabel):
    def __init__(self, settings, parent=None):
        super().__init__("WiFi: --", parent, settings=settings)
        
        self.cached_networks = [] # Cache storage
        
        self.worker = WinRTWorker()
        # Ensure component always listens to worker to update cache,
        # even if popup isn't open
        self.worker.scan_finished.connect(self._on_background_scan_finished)
        
        self.conn_checker = ConnectivityWorker()
        self.conn_checker.status_changed.connect(self.update_icon_status)
        self.conn_checker.start()
        
        # Trigger an initial scan shortly after startup so cache is ready
        QTimer.singleShot(2000, self.worker.start_init)

    def _on_background_scan_finished(self, networks):
        dprint(f"Network Component received {len(networks)} networks (Background Update)")
        self.cached_networks = networks

    def update_icon_status(self, is_online):
        if is_online:
            self.setIcon("mdi.wifi")
            self.setText("Online")
        else:
            self.setIcon("mdi.wifi-off")
            self.setText("Offline")

    def get_popup_content(self):
        if not WINRT_AVAILABLE:
            return "Error", "WinRT modules missing.\nSee console logs."
        
        # Pass the cache to the widget so it opens instantly
        widget = WifiPopupWidget(self.worker, self.cached_networks)
        return "Wi-Fi", widget