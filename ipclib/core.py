# ipclib/core.py
import sys
import time
from .config import PIPE_PREFIX
from .registry import RegistryManager
from .server import IPCServer
from .client import IPCClient, IPCStream

class RemoteExecutionError(Exception):
    """Raised when the remote function fails (logic error on server)."""
    pass

class PeerOfflineError(Exception):
    """Raised when the target process is not running or unreachable."""
    pass

class RemotePeer:
    """
    Magic Proxy.
    Allows calling `peer.function_name()` naturally.
    """
    def __init__(self, ipc, target_name):
        self._ipc = ipc
        self._target = target_name

    def __getattr__(self, name):
        def wrapper(*args, **kwargs):
            # Check for magic kwarg to toggle streaming
            if kwargs.pop('_stream', False):
                return self._ipc.stream(self._target, name, *args, **kwargs)
            return self._ipc.call(self._target, name, *args, **kwargs)
        return wrapper

class NeverLiieIPC:
    def __init__(self, app_name):
        self.app_name = app_name
        self.methods = {}
        
        # 1. Singleton Enforcement (Exit if I already exist)
        if IPCClient.connect(app_name):
            print(f"[IPC] {app_name} is already running. Exiting.")
            sys.exit(0)

        # 2. Initialize Registry Helper
        self.registry = RegistryManager(app_name)
        self.registry.register_self()

        # 3. Start Server Thread
        self.server = IPCServer(app_name, self.methods)
        self.server.start()

    # --- DECORATOR API ---
    def expose(self, name_or_func=None):
        """
        Decorator to register functions.
        Usage:
           @ipc.expose
           def foo(): ...
           
           @ipc.expose("alias")
           def foo(): ...
        """
        # Case 1: Bare decorator @ipc.expose
        if callable(name_or_func):
            func = name_or_func
            self.methods[func.__name__] = func
            return func

        # Case 2: Decorator with args @ipc.expose("alias")
        # Or manual call ipc.expose("alias", func)
        def decorator(func):
            # If called manually as expose('name', func), name_or_func is the name
            name = name_or_func if isinstance(name_or_func, str) else func.__name__
            self.methods[name] = func
            return func
            
        # If user did ipc.expose('name', func) - manual legacy support
        # We return the decorator, expecting them to use it, but if they passed a function...
        # Simpler to just assume decorator usage or standard usage.
        return decorator
        
    def get_peer(self, target_name):
        """Returns a proxy object for the target application."""
        return RemotePeer(self, target_name)

    # --- LIFECYCLE MANAGEMENT ---
    def ping(self, target_name):
        """
        Checks if a target is online.
        Returns: True/False
        """
        conn = IPCClient.connect(target_name)
        if conn:
            try:
                conn.close()
                return True
            except:
                return False
        return False

    def wake(self, target_name):
        """
        Explicitly attempts to launch the target from the registry.
        Fire-and-Forget: Does not wait for the target to come online.
        
        Returns: True if launch command was initiated (or already online).
        Returns: False if registry entry is missing.
        """
        # Optimization: If already running, we consider it "woken"
        if self.ping(target_name):
            return True

        # Attempt Launch
        print(f"[IPC] Waking {target_name} (Fire-and-Forget)...")
        launched = self.registry.launch_target(target_name)
        
        return launched

    # --- EXECUTION LOGIC ---
    def call(self, target, method, *args, **kwargs):
        """
        Calls a remote method. 
        Raises PeerOfflineError if target is down.
        """
        response = self._send(target, method, args, kwargs, stream=False)
        
        if response is None:
             raise PeerOfflineError(f"Target '{target}' is offline.")

        status = response.get("status")
        if status == "ok":
            return response.get("data")
        elif status == "error":
            raise RemoteExecutionError(f"Remote Error in {target}.{method}: {response.get('msg')}")
        else:
            raise RemoteExecutionError(f"Unknown Protocol Response: {response}")

    def stream(self, target, method, *args, **kwargs):
        """Returns an IPCStream iterator."""
        return self._send(target, method, args, kwargs, stream=True)

    def _send(self, target, method, args, kwargs, stream):
        # 1. Try to connect
        conn = self._connect_to_target(target, method, args, kwargs)
        
        # 2. If failed, DO NOT auto-launch. Fail immediately.
        if not conn:
            if stream: raise PeerOfflineError(f"Target '{target}' is offline.")
            return None

        # 3. Process Response
        if stream:
            header = conn.recv()
            if header.get("status") == "stream_start":
                return IPCStream(self, target, header["task_id"], conn)
            else:
                conn.close()
                return []
        else:
            try:
                # Default timeout 5s unless specified
                timeout = kwargs.pop("_timeout", 5.0)
                if conn.poll(timeout):
                    res = conn.recv()
                    conn.close()
                    return res
                else:
                    conn.close()
                    return {"status": "error", "msg": "Request Timeout"}
            except Exception:
                return {"status": "error", "msg": "Connection Dropped"}

    def _connect_to_target(self, target, method, args, kwargs):
        conn = IPCClient.connect(target)
        if conn:
            try:
                conn.send({"method": method, "args": args, "kwargs": kwargs})
                return conn
            except:
                conn.close()
                return None
        return None