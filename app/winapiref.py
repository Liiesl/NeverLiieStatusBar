import ctypes
import ctypes.wintypes
import sys

# --- 1. CTYPES DEFINITIONS ---------------------------------------------------

# Libs
user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32
gdi32 = ctypes.windll.gdi32
try:
    secur32 = ctypes.windll.secur32
except:
    secur32 = None

# Constants
WM_COPYDATA = 0x004A
WM_DESTROY = 0x0002
WM_MOUSEMOVE = 0x0200
WM_LBUTTONDOWN = 0x0201
WM_LBUTTONUP = 0x0202
WM_LBUTTONDBLCLK = 0x0203
WM_RBUTTONDOWN = 0x0204
WM_RBUTTONUP = 0x0205
WM_MBUTTONDOWN = 0x0207
WM_MBUTTONUP = 0x0208
WM_CONTEXTMENU = 0x007B
WM_TIMER = 0x0113

# Tray Events
NIN_SELECT = 0x400
NIN_KEYSELECT = 0x401
NIN_BALLOONSHOW = 0x402
NIN_BALLOONHIDE = 0x403
NIN_BALLOONTIMEOUT = 0x404
NIN_CONTEXTMENU = 0x405
NIN_POPUPOPEN = 0x406
NIN_POPUPCLOSE = 0x407

# Messages
NIM_ADD = 0x0
NIM_MODIFY = 0x1
NIM_DELETE = 0x2
NIM_SETFOCUS = 0x3
NIM_SETVERSION = 0x4

# Flags
NIF_MESSAGE = 0x1
NIF_ICON = 0x2
NIF_TIP = 0x4
NIF_STATE = 0x8
NIF_INFO = 0x10
NIF_GUID = 0x20

# Styles & Window Pos
WS_POPUP = 0x80000000
WS_CLIPCHILDREN = 0x02000000
WS_CLIPSIBLINGS = 0x04000000
WS_EX_TOOLWINDOW = 0x00000080
WS_EX_TOPMOST = 0x00000008
SWP_NOMOVE = 0x0002
SWP_NOSIZE = 0x0001
SWP_NOACTIVATE = 0x0010
HWND_TOPMOST = -1
PROCESS_QUERY_LIMITED_INFORMATION = 0x1000

# Types
LRESULT = ctypes.c_int64
WPARAM = ctypes.c_uint64
LPARAM = ctypes.c_int64
HWND = ctypes.wintypes.HWND
HBITMAP = ctypes.wintypes.HANDLE
WNDPROCTYPE = ctypes.WINFUNCTYPE(LRESULT, HWND, ctypes.c_uint, WPARAM, LPARAM)

# Structs
class GUID(ctypes.Structure):
    _fields_ = [("Data1", ctypes.c_ulong), ("Data2", ctypes.c_ushort), 
                ("Data3", ctypes.c_ushort), ("Data4", ctypes.c_ubyte * 8)]
    def __str__(self):
        return f"{{{self.Data1:08x}-{self.Data2:04x}-{self.Data3:04x}-{self.Data4[0]:02x}{self.Data4[1]:02x}-{self.Data4[2]:02x}{self.Data4[3]:02x}{self.Data4[4]:02x}{self.Data4[5]:02x}{self.Data4[6]:02x}{self.Data4[7]:02x}}}"

class WNDCLASS(ctypes.Structure):
    _fields_ = [('style', ctypes.c_uint), ('lpfnWndProc', WNDPROCTYPE), 
                ('cbClsExtra', ctypes.c_int), ('cbWndExtra', ctypes.c_int), 
                ('hInstance', ctypes.wintypes.HINSTANCE), ('hIcon', ctypes.wintypes.HICON), 
                ('hCursor', ctypes.wintypes.HANDLE), ('hbrBackground', ctypes.wintypes.HANDLE), 
                ('lpszMenuName', ctypes.wintypes.LPCWSTR), ('lpszClassName', ctypes.wintypes.LPCWSTR)]

class COPYDATASTRUCT(ctypes.Structure):
    _fields_ = [('dwData', ctypes.c_uint64), ('cbData', ctypes.c_uint32), ('lpData', ctypes.c_void_p)]

class _U_TIMEOUT_VERSION(ctypes.Union):
    _fields_ = [("uTimeout", ctypes.wintypes.UINT), ("uVersion", ctypes.wintypes.UINT)]

class NOTIFYICONDATA_64(ctypes.Structure):
    _fields_ = [('cbSize', ctypes.wintypes.DWORD), ('hWnd', HWND), ('uID', ctypes.wintypes.UINT), 
                ('uFlags', ctypes.wintypes.UINT), ('uCallbackMessage', ctypes.wintypes.UINT), 
                ('hIcon', ctypes.wintypes.HICON), ('szTip', ctypes.c_wchar * 128), 
                ('dwState', ctypes.wintypes.DWORD), ('dwStateMask', ctypes.wintypes.DWORD), 
                ('szInfo', ctypes.c_wchar * 256), ('uVersion', _U_TIMEOUT_VERSION), 
                ('szInfoTitle', ctypes.c_wchar * 64), ('dwInfoFlags', ctypes.wintypes.DWORD), 
                ('guidItem', GUID), ('hBalloonIcon', ctypes.wintypes.HICON)]

class NOTIFYICONDATA_32(ctypes.Structure):
    _pack_ = 1
    _fields_ = [('cbSize', ctypes.wintypes.DWORD), ('hWnd', ctypes.c_uint32), ('uID', ctypes.wintypes.UINT), 
                ('uFlags', ctypes.wintypes.UINT), ('uCallbackMessage', ctypes.wintypes.UINT), 
                ('hIcon', ctypes.c_uint32), ('szTip', ctypes.c_wchar * 128), 
                ('dwState', ctypes.wintypes.DWORD), ('dwStateMask', ctypes.wintypes.DWORD), 
                ('szInfo', ctypes.c_wchar * 256), ('uVersion', _U_TIMEOUT_VERSION), 
                ('szInfoTitle', ctypes.c_wchar * 64), ('dwInfoFlags', ctypes.wintypes.DWORD), 
                ('guidItem', GUID), ('hBalloonIcon', ctypes.c_uint32)]

class SHELLTRAYDATA(ctypes.Structure):
    _pack_ = 1
    _fields_ = [('dwUnknown', ctypes.c_ulong), ('dwMessage', ctypes.c_uint)]

class WINNOTIFYICONIDENTIFIER(ctypes.Structure):
    _fields_ = [('dwMagic', ctypes.wintypes.DWORD), ('dwSize', ctypes.wintypes.DWORD), 
                ('dwMessage', ctypes.c_int), ('hWnd', HWND), ('uID', ctypes.c_uint), ('guidItem', GUID)]

class BITMAP(ctypes.Structure):
    _fields_ = [("bmType", ctypes.c_long), ("bmWidth", ctypes.c_long), ("bmHeight", ctypes.c_long), 
                ("bmWidthBytes", ctypes.c_long), ("bmPlanes", ctypes.c_short), 
                ("bmBitsPixel", ctypes.c_short), ("bmBits", ctypes.c_void_p)]

class ICONINFO(ctypes.Structure):
    _fields_ = [("fIcon", ctypes.wintypes.BOOL), ("xHotspot", ctypes.wintypes.DWORD), 
                ("yHotspot", ctypes.wintypes.DWORD), ("hbmMask", HBITMAP), ("hbmColor", HBITMAP)]

# Function Argtypes
user32.GetIconInfo.argtypes = [ctypes.wintypes.HICON, ctypes.POINTER(ICONINFO)]
gdi32.GetObjectW.argtypes = [ctypes.wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p]
gdi32.GetBitmapBits.argtypes = [HBITMAP, ctypes.c_long, ctypes.c_void_p]
gdi32.DeleteObject.argtypes = [ctypes.wintypes.HANDLE]
user32.SetPropW.argtypes = [HWND, ctypes.wintypes.LPCWSTR, ctypes.wintypes.HANDLE]
user32.SetPropW.restype = ctypes.wintypes.BOOL
user32.IsWindow.argtypes = [HWND]
user32.IsWindow.restype = ctypes.wintypes.BOOL
user32.PostMessageW.argtypes = [HWND, ctypes.c_uint, WPARAM, LPARAM]
user32.DefWindowProcW.argtypes = [HWND, ctypes.c_uint, WPARAM, LPARAM]
user32.GetWindowThreadProcessId.argtypes = [HWND, ctypes.POINTER(ctypes.c_ulong)]
user32.SetWindowPos.argtypes = [HWND, HWND, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_uint]
user32.SetWindowPos.restype = ctypes.c_int
user32.SetTimer.argtypes = [HWND, ctypes.c_uint, ctypes.c_uint, ctypes.c_void_p]
user32.SetTimer.restype = ctypes.c_uint
user32.FindWindowExW.argtypes = [HWND, HWND, ctypes.wintypes.LPCWSTR, ctypes.wintypes.LPCWSTR]
user32.FindWindowExW.restype = HWND
user32.SendMessageW.argtypes = [HWND, ctypes.c_uint, WPARAM, LPARAM]
user32.SendMessageW.restype = ctypes.c_int
user32.AllowSetForegroundWindow.argtypes = [ctypes.c_ulong]
user32.AllowSetForegroundWindow.restype = ctypes.wintypes.BOOL
user32.SendNotifyMessageW.argtypes = [HWND, ctypes.c_uint, WPARAM, LPARAM]
user32.SendNotifyMessageW.restype = ctypes.c_int
kernel32.OpenProcess.argtypes = [ctypes.c_uint, ctypes.wintypes.BOOL, ctypes.c_ulong]
kernel32.OpenProcess.restype = ctypes.wintypes.HANDLE
kernel32.CloseHandle.argtypes = [ctypes.wintypes.HANDLE]
kernel32.CloseHandle.restype = ctypes.wintypes.BOOL
kernel32.QueryFullProcessImageNameW.argtypes = [ctypes.wintypes.HANDLE, ctypes.c_uint, ctypes.wintypes.LPWSTR, ctypes.POINTER(ctypes.c_ulong)]

# Additional Profile/Lock functions
if secur32:
    secur32.GetUserNameExW.argtypes = [ctypes.c_int, ctypes.c_wchar_p, ctypes.POINTER(ctypes.c_ulong)]
    secur32.GetUserNameExW.restype = ctypes.c_ubyte

# --- 2. COM / PYCAW (Audio) --------------------------------------------------
COM_AVAILABLE = False
AudioUtilities = None
IAudioEndpointVolume = None
CLSCTX_ALL = None

class ERoleStub:
    eConsole = 0
    eMultimedia = 1
    eCommunications = 2

ERole = ERoleStub

try:
    import comtypes
    from comtypes import CLSCTX_ALL
    from pycaw.pycaw import AudioUtilities, IAudioEndpointVolume
    try:
        from pycaw.constants import ERole
    except ImportError:
        pass
    COM_AVAILABLE = True
except ImportError:
    pass


# --- 3. WINRT (General, Media, Network, User) --------------------------------
WINRT_AVAILABLE = False

# Placeholders for WinRT classes
WiFiAdapter = None
WiFiReconnectionKind = None
PasswordCredential = None
GlobalSystemMediaTransportControlsSessionManager = None
GlobalSystemMediaTransportControlsSessionPlaybackStatus = None
DataReader = None
InputStreamOptions = None
User = None
UserType = None
UserPictureSize = None
KnownUserProperties = None
IPropertyValue = None
winrt_foundation = None

try:
    # Requires: pip install winrt-Windows.Foundation winrt-Windows.Devices.Wifi etc.
    import winrt.windows.foundation as winrt_foundation
    import winrt.windows.foundation.collections 
    from winrt.windows.foundation import IPropertyValue

    # Network
    from winrt.windows.devices.wifi import WiFiAdapter, WiFiReconnectionKind
    from winrt.windows.security.credentials import PasswordCredential

    # Media
    from winrt.windows.media.control import (
        GlobalSystemMediaTransportControlsSessionManager, 
        GlobalSystemMediaTransportControlsSessionPlaybackStatus
    )
    from winrt.windows.storage.streams import DataReader, InputStreamOptions

    # User Profile
    from winrt.windows.system import User, UserType, UserPictureSize, KnownUserProperties
    
    WINRT_AVAILABLE = True
except ImportError:
    pass

# Helper to unbox IPropertyValue (Used in Profile)
def unbox_winrt_str(winrt_obj):
    if not WINRT_AVAILABLE: return ""
    if winrt_obj is None: return ""
    if isinstance(winrt_obj, str): return winrt_obj
    try:
        prop_val = IPropertyValue._from(winrt_obj)
        if prop_val.type == winrt_foundation.PropertyType.STRING:
            return prop_val.get_string()
    except:
        pass
    return ""

# --- 4. WLAN API (WiFi Profiles) -------------------------------------------------
WLAN_AVAILABLE = False
wlanapi = None

try:
    wlanapi = ctypes.windll.wlanapi
    WLAN_AVAILABLE = True
except:
    pass

if WLAN_AVAILABLE:
    WLAN_API_VERSION = (2, 0)
    
    class _GUID(ctypes.Structure):
        _fields_ = [
            ("Data1", ctypes.c_ulong),
            ("Data2", ctypes.c_ushort),
            ("Data3", ctypes.c_ushort),
            ("Data4", ctypes.c_ubyte * 8)
        ]

    class WLAN_INTERFACE_INFO(ctypes.Structure):
        _fields_ = [
            ("InterfaceGuid", _GUID),
            ("strInterfaceDescription", ctypes.wintypes.WCHAR * 256),
            ("State", ctypes.wintypes.DWORD)
        ]

    class WLAN_INTERFACE_INFO_LIST(ctypes.Structure):
        _fields_ = [
            ("dwNumberOfItems", ctypes.wintypes.DWORD),
            ("dwIndex", ctypes.wintypes.DWORD),
            ("InterfaceInfo", WLAN_INTERFACE_INFO * 1)
        ]

    class WLAN_PROFILE_INFO(ctypes.Structure):
        _fields_ = [
            ("strProfileName", ctypes.wintypes.WCHAR * 256),
            ("dwFlags", ctypes.wintypes.DWORD)
        ]

    class WLAN_PROFILE_INFO_LIST(ctypes.Structure):
        _fields_ = [
            ("dwNumberOfItems", ctypes.wintypes.DWORD),
            ("dwIndex", ctypes.wintypes.DWORD),
            ("ProfileInfo", WLAN_PROFILE_INFO * 1)
        ]

    wlanapi.WlanOpenHandle.argtypes = [
        ctypes.wintypes.DWORD, ctypes.c_void_p,
        ctypes.POINTER(ctypes.wintypes.DWORD), ctypes.POINTER(ctypes.wintypes.HANDLE)
    ]
    wlanapi.WlanOpenHandle.restype = ctypes.wintypes.DWORD

    wlanapi.WlanCloseHandle.argtypes = [ctypes.wintypes.HANDLE, ctypes.c_void_p]
    wlanapi.WlanCloseHandle.restype = ctypes.wintypes.DWORD

    wlanapi.WlanEnumInterfaces.argtypes = [
        ctypes.wintypes.HANDLE, ctypes.c_void_p,
        ctypes.POINTER(ctypes.POINTER(WLAN_INTERFACE_INFO_LIST))
    ]
    wlanapi.WlanEnumInterfaces.restype = ctypes.wintypes.DWORD

    wlanapi.WlanGetProfileList.argtypes = [
        ctypes.wintypes.HANDLE, ctypes.POINTER(_GUID), ctypes.c_void_p,
        ctypes.POINTER(ctypes.POINTER(WLAN_PROFILE_INFO_LIST))
    ]
    wlanapi.WlanGetProfileList.restype = ctypes.wintypes.DWORD

    wlanapi.WlanFreeMemory.argtypes = [ctypes.c_void_p]
    wlanapi.WlanFreeMemory.restype = None

    _wlan_handle = None

    def _get_wlan_handle():
        global _wlan_handle
        if _wlan_handle is not None:
            return _wlan_handle
        handle = ctypes.wintypes.HANDLE()
        version = ctypes.wintypes.DWORD()
        result = wlanapi.WlanOpenHandle(
            WLAN_API_VERSION[0], None, ctypes.byref(version), ctypes.byref(handle)
        )
        if result == 0:
            _wlan_handle = handle
            return handle
        return None

    def get_saved_wifi_profiles():
        handle = _get_wlan_handle()
        if not handle:
            return set()
        
        p_list = ctypes.POINTER(WLAN_INTERFACE_INFO_LIST)()
        result = wlanapi.WlanEnumInterfaces(handle, None, ctypes.byref(p_list))
        if result != 0:
            return set()
        
        if p_list.contents.dwNumberOfItems == 0:
            wlanapi.WlanFreeMemory(p_list)
            return set()
        
        iface_guid = p_list.contents.InterfaceInfo[0].InterfaceGuid
        wlanapi.WlanFreeMemory(p_list)
        
        p_profiles = ctypes.POINTER(WLAN_PROFILE_INFO_LIST)()
        result = wlanapi.WlanGetProfileList(handle, ctypes.byref(iface_guid), None, ctypes.byref(p_profiles))
        if result != 0:
            return set()
        
        num = p_profiles.contents.dwNumberOfItems
        profile_array = ctypes.cast(
            ctypes.addressof(p_profiles.contents.ProfileInfo),
            ctypes.POINTER(WLAN_PROFILE_INFO)
        )
        
        profiles = set()
        for i in range(num):
            name = profile_array[i].strProfileName
            profiles.add(name)
        
        wlanapi.WlanFreeMemory(p_profiles)
        return profiles
else:
    def get_saved_wifi_profiles():
        return set()