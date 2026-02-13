class Settings:
    def __init__(self):
        # --- DIMENSIONS & POSITION ---
        self.bar_height = 40          # Visual height of the bar itself
        self.mouse_trigger_height = 5
        
        # --- FLOATING STYLE SETTINGS ---
        self.floating_margin_x = 20   # Distance from Left/Right edges
        self.floating_margin_y = 5   # Distance from Top edge
        self.border_radius = 20       # Fully rounded corners
        
        # --- TIMING ---
        self.anim_duration = 300      # Slightly slower for smoother float
        self.monitor_interval = 200
        self.auto_hide_delay = 600
        self.trigger_dwell_time = 500
        
        # --- UPDATE INTERVALS ---
        self.audio_poll_rate = 500
        self.clock_refresh_rate = 1000
        self.battery_poll_rate = 5000
        
        # --- VISUALS / THEME ---
        self.bg_color = "rgba(25, 25, 25, 240)" # Slightly more opaque
        self.border_color = "rgba(255, 255, 255, 40)"
        self.text_color = "#e0e0e0"
        self.hover_bg = "rgba(255, 255, 255, 20)"
        self.font_family = "Segoe UI"
        self.font_size = "13px"

        self.tray_ignore_list = ["SearchApp.exe", "ShellExperienceHost.exe"]