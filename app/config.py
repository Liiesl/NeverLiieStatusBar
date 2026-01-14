class Settings:
    def __init__(self):
        # --- DIMENSIONS & POSITION ---
        self.bar_height = 35
        self.mouse_trigger_height = 5
        
        # --- TIMING ---
        self.anim_duration = 250
        self.monitor_interval = 200
        self.auto_hide_delay = 600
        
        # --- UPDATE INTERVALS ---
        self.audio_poll_rate = 500
        self.clock_refresh_rate = 1000
        self.battery_poll_rate = 5000  # Added explicit battery poll
        
        # --- VISUALS / THEME ---
        self.bg_color = "rgba(25, 25, 25, 230)"
        self.border_color = "rgba(255, 255, 255, 30)"
        self.text_color = "#e0e0e0"
        self.hover_bg = "rgba(255, 255, 255, 20)"
        self.font_family = "Segoe UI"
        self.font_size = "13px"
        self.border_radius = 10