# ipclib/config.py
import os

# Base directory for configuration
REGISTRY_DIR = os.path.join(os.path.expanduser("~"), ".neverliie")
REGISTRY_FILE = os.path.join(REGISTRY_DIR, "registry.json")

# Named Pipe Prefix (Windows specific)
PIPE_PREFIX = r"\\.\pipe\NeverLiie_"
