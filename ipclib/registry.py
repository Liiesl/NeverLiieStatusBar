# ipclib/registry.py
import json
import os
import sys
import time
import subprocess
from .config import REGISTRY_DIR, REGISTRY_FILE

class RegistryManager:
    def __init__(self, app_name):
        self.app_name = app_name
        self._ensure_dir()

    def _ensure_dir(self):
        if not os.path.exists(REGISTRY_DIR):
            try:
                os.makedirs(REGISTRY_DIR)
            except FileExistsError:
                pass

    def register_self(self):
        """Determines execution mode and saves to registry."""
        entry = self._get_launch_info()
        
        # Retry logic for file locking contention
        for _ in range(5):
            try:
                data = {}
                if os.path.exists(REGISTRY_FILE):
                    with open(REGISTRY_FILE, "r") as f:
                        try:
                            data = json.load(f)
                        except json.JSONDecodeError:
                            pass

                data[self.app_name] = entry

                with open(REGISTRY_FILE, "w") as f:
                    json.dump(data, f, indent=4)
                return
            except PermissionError:
                time.sleep(0.05)
            except Exception as e:
                print(f"[IPC] Registry Write Error: {e}")
                return

    def _get_launch_info(self):
        """Detects Nuitka/PyInstaller vs Raw Python."""
        if getattr(sys, "frozen", False):
            # Running as compiled EXE
            return {
                "type": "binary",
                "cmd": [sys.executable],
                "cwd": os.path.dirname(sys.executable),
            }
        else:
            # Running as Python Script
            return {
                "type": "script",
                "cmd": [sys.executable, os.path.abspath(sys.argv[0])],
                "cwd": os.getcwd(),
            }

    def launch_target(self, target_name):
        """Reads registry and spawns the target process."""
        if not os.path.exists(REGISTRY_FILE):
            return False

        try:
            with open(REGISTRY_FILE, "r") as f:
                data = json.load(f)
        except (json.JSONDecodeError, PermissionError):
            return False

        info = data.get(target_name)
        if not info:
            return False

        cmd = info.get("cmd", [])
        cwd = info.get("cwd", os.getcwd())

        # --- ZOMBIE PRUNING ---
        # Validate executable exists before trying to run
        exe_path = cmd[0] if len(cmd) == 1 else cmd[1]
        if not os.path.exists(exe_path):
            print(
                f"[IPC] Target {target_name} not found at {exe_path}. Pruning registry."
            )
            self._prune_entry(data, target_name)
            return False
        # ----------------------

        try:
            # Launch DETACHED so it survives if this process dies
            subprocess.Popen(
                cmd,
                cwd=cwd,
                creationflags=subprocess.DETACHED_PROCESS | subprocess.CREATE_NEW_PROCESS_GROUP,
                close_fds=True,
                shell=False,
            )
            return True
        except Exception as e:
            print(f"[IPC] Launch failed: {e}")
            return False

    def _prune_entry(self, data, target_name):
        if target_name in data:
            del data[target_name]
            try:
                with open(REGISTRY_FILE, "w") as f:
                    json.dump(data, f, indent=4)
            except:
                pass