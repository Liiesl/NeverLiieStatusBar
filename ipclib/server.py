# ipclib/server.py
import threading
import types
import uuid
import time
from multiprocessing.connection import Listener
from .config import PIPE_PREFIX

class IPCServer(threading.Thread):
    def __init__(self, app_name, methods_dict):
        super().__init__(daemon=True)
        self.pipe_address = f"{PIPE_PREFIX}{app_name}"
        self.methods = methods_dict
        self.running = True
        
        # Tracking active streams for cancellation
        self.active_tasks = {} 
        self.active_tasks_lock = threading.Lock()

    def run(self):
        while self.running:
            try:
                # Create the named pipe listener
                with Listener(self.pipe_address) as listener:
                    while self.running:
                        try:
                            conn = listener.accept()
                            # Handle every connection in a separate thread
                            t = threading.Thread(
                                target=self._handle_client,
                                args=(conn,),
                                daemon=True
                            )
                            t.start()
                        except (EOFError, OSError):
                            pass
            except Exception:
                # If pipe creation fails (rare race condition), wait and retry
                time.sleep(1)

    def cancel_task(self, task_id):
        """Thread-safe cancellation of a generator task."""
        with self.active_tasks_lock:
            if task_id in self.active_tasks:
                self.active_tasks[task_id].set()

    def _handle_client(self, conn):
        task_id = None
        try:
            msg = conn.recv()
            method_name = msg.get("method")

            # 1. Internal: Cancel Request
            if method_name == "__cancel_task__":
                tid = msg.get("kwargs", {}).get("task_id")
                self.cancel_task(tid)
                conn.send({"status": "ok"})
                return

            # 2. Ping
            if method_name == "__ping__":
                conn.send(True)
                return

            # 3. Lookup User Function
            func = self.methods.get(method_name)
            if not func:
                conn.send({"status": "error", "msg": f"Method '{method_name}' not found"})
                return

            # 4. Execute
            args = msg.get("args", [])
            kwargs = msg.get("kwargs", {})
            res = func(*args, **kwargs)

            # 5. Handle Result Type
            if isinstance(res, types.GeneratorType):
                # STREAMING RESPONSE
                task_id = str(uuid.uuid4())
                stop_event = threading.Event()
                
                with self.active_tasks_lock:
                    self.active_tasks[task_id] = stop_event

                conn.send({"status": "stream_start", "task_id": task_id})

                try:
                    for item in res:
                        if stop_event.is_set():
                            break
                        conn.send({"status": "progress", "data": item})
                    conn.send({"status": "stream_end"})
                finally:
                    with self.active_tasks_lock:
                        if task_id in self.active_tasks:
                            del self.active_tasks[task_id]
            else:
                # STANDARD RESPONSE
                conn.send({"status": "ok", "data": res})

        except (BrokenPipeError, EOFError):
            pass
        except Exception as e:
            # Catch application errors and send back to client
            try:
                conn.send({"status": "error", "msg": str(e)})
            except:
                pass
        finally:
            try:
                conn.close()
            except:
                pass