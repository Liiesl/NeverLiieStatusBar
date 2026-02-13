# ipclib/client.py
from multiprocessing.connection import Client
from .config import PIPE_PREFIX

class IPCClient:
    """Static helper to manage low-level connections."""
    
    @staticmethod
    def connect(target_name):
        pipe_address = f"{PIPE_PREFIX}{target_name}"
        try:
            return Client(pipe_address)
        except (FileNotFoundError, ConnectionRefusedError, OSError):
            return None

class IPCStream:
    """
    An iterator for streaming data from the server.
    Handles 'next' and 'cancel' logic transparently.
    """

    def __init__(self, ipc_instance, target, task_id, conn):
        self._ipc = ipc_instance
        self._target = target
        self._task_id = task_id
        self._conn = conn
        self._active = True

    def __iter__(self):
        return self

    def __next__(self):
        if not self._active:
            raise StopIteration
        try:
            msg = self._conn.recv()
            
            # Check for protocol signals
            if msg.get("status") == "stream_end":
                self._close()
                raise StopIteration
            
            if msg.get("status") == "error":
                self._active = False
                # Re-raise server-side errors on the client
                raise Exception(f"Stream Error: {msg.get('msg')}")
            
            return msg.get("data")
            
        except (EOFError, OSError):
            self._close()
            raise StopIteration

    def cancel(self):
        """Sends a signal to the server to terminate this specific task."""
        if self._active:
            self._close()
            # We open a NEW connection just to send the kill signal
            # This is because the original pipe is busy iterating
            try:
                self._ipc.call(self._target, "__cancel_task__", task_id=self._task_id)
            except:
                pass

    def _close(self):
        self._active = False
        try:
            self._conn.close()
        except:
            pass